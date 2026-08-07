use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use std::sync::Arc;

use geo::{Contains, Geometry, MultiPolygon, Point, indexed::IntervalTreeMultiPolygon};
use geozero::ToGeo;
use numpy::{PyArray, PyReadonlyArrayDyn};
use pyo3::{prelude::*, types::PyBytes};

pub use crate::providers::LandmaskProvider;

#[derive(Clone)]
#[pyclass(from_py_object)]
pub struct Shapes {
    tree: Arc<IntervalTreeMultiPolygon<f64>>,
}

impl Shapes {
    pub fn from_multipolygon(mp: MultiPolygon) -> Shapes {
        Shapes {
            tree: Arc::new(IntervalTreeMultiPolygon::new(&mp)),
        }
    }

    pub fn from_geom(geom: Geometry) -> io::Result<Shapes> {
        match geom {
            Geometry::MultiPolygon(mp) => Ok(Shapes::from_multipolygon(mp)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected MultiPolygon geometry",
            )),
        }
    }

    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Shapes> {
        let g = Shapes::get_geometry_from_compressed(path)?;
        Shapes::from_geom(g)
    }

    pub fn get_geometry_from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Geometry> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let mut fd = xz2::bufread::XzDecoder::new(fd);
        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;

        geozero::wkb::Wkb(buf)
            .to_geo()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}

#[pymethods]
impl Shapes {
    /// Make a new Gshhg shapes instance.
    #[staticmethod]
    pub fn new(py: Python, provider: LandmaskProvider) -> io::Result<Self> {
        let buf = Shapes::wkb(py, provider)?;
        let geom = geozero::wkb::Wkb(buf.as_bytes().to_vec())
            .to_geo()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        Shapes::from_geom(geom)
    }

    /// Get the WKB for the GSHHG shapes (full resolution).
    #[staticmethod]
    pub fn wkb(py: Python<'_>, provider: LandmaskProvider) -> io::Result<Bound<'_, PyBytes>> {
        use crate::GsshgData;
        use crate::OsmData;

        let buf = match provider {
            LandmaskProvider::Gshhg => GsshgData::get("gshhg.wkb.xz"),
            LandmaskProvider::Osm => OsmData::get("osm.wkb.xz"),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find shapes"))?;

        let buf: &[u8] = buf.data.borrow();
        let mut fd = xz2::read::XzDecoder::new(buf);

        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;

        Ok(PyBytes::new(py, &buf))
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] east
    /// `y` is latitude,  [- 90,  90] north
    ///
    /// Returns `true` if the point is on land.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let x = super::modulate_longitude(x);
        debug_assert!(x >= -180. && x <= 180.);
        assert!(y > -90. && y <= 90.);

        self.tree.contains(&Point::new(x, y))
    }

    /// Same as `contains`, but does not check for bounds.
    pub(crate) fn contains_unchecked(&self, x: f64, y: f64) -> bool {
        self.tree.contains(&Point::new(x, y))
    }

    pub fn contains_many(
        &self,
        py: Python,
        x: PyReadonlyArrayDyn<f64>,
        y: PyReadonlyArrayDyn<f64>,
    ) -> Py<PyArray<bool, numpy::Ix1>> {
        let x = x.as_array();
        let y = y.as_array();

        PyArray::from_iter(
            py,
            x.iter().zip(y.iter()).map(|(x, y)| self.contains(*x, *y)),
        )
        .unbind()
    }

    pub fn contains_many_par(
        &self,
        py: Python,
        x: PyReadonlyArrayDyn<f64>,
        y: PyReadonlyArrayDyn<f64>,
    ) -> Py<PyArray<bool, numpy::IxDyn>> {
        let x = x.as_array();
        let y = y.as_array();

        use ndarray::Zip;
        let contains = Zip::from(x.view())
            .and(y.view())
            .par_map_collect(|x, y| self.contains(*x, *y));
        PyArray::from_owned_array(py, contains).unbind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_compressed() {
        let _s = Shapes::from_compressed("assets/gshhg.wkb.xz").unwrap();
        let _s = Shapes::from_compressed("assets/osm.wkb.xz").unwrap();
    }

    #[test]
    fn test_load() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            Shapes::new(py, LandmaskProvider::Gshhg).unwrap();
            Shapes::new(py, LandmaskProvider::Osm).unwrap();
        });
    }

    #[test]
    fn test_np() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mask = Shapes::new(py, LandmaskProvider::Gshhg).unwrap();
            assert!(!mask.contains(5., 90.));

            let mask = Shapes::new(py, LandmaskProvider::Osm).unwrap();
            assert!(!mask.contains(5., 90.));
        });
    }

    #[test]
    fn test_sp() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mask = Shapes::new(py, LandmaskProvider::Gshhg).unwrap();
            assert!(mask.contains(5., -89.99));

            let mask = Shapes::new(py, LandmaskProvider::Osm).unwrap();
            assert!(mask.contains(5., -89.99));
        });
    }

    #[cfg(feature = "nightly")]
    mod benches {
        use super::*;
        use test::Bencher;

        #[bench]
        fn test_contains_on_land(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                for provider in [LandmaskProvider::Gshhg, LandmaskProvider::Osm] {
                    let s = Shapes::new(py, provider).unwrap();
                    assert!(s.contains(15., 65.6));
                    assert!(s.contains(10., 60.0));
                    b.iter(|| s.contains(15., 65.6));
                }
            })
        }

        #[bench]
        fn test_contains_in_ocean(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                for provider in [LandmaskProvider::Gshhg, LandmaskProvider::Osm] {
                    let s = Shapes::new(py, provider).unwrap();
                    assert!(!s.contains(5., 65.6));
                    b.iter(|| s.contains(5., 65.6));
                }
            })
        }
    }
}

