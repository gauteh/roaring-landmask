use pyo3::{prelude::*, types::PyBytes};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;

use geos::{CoordSeq, Geom, Geometry, PreparedGeometry};
use numpy::{PyArray, PyReadonlyArrayDyn};

pub use crate::providers::LandmaskProvider;

#[pyclass]
pub struct Shapes {
    // prepped requires `geom` above to be around, and is valid as long as geom is alive.
    geom: Geometry,
    prepped: PreparedGeometry,
}

// impl Drop for Shapes {
//     fn drop(&mut self) {
//         unsafe { drop(Box::from_raw(self.geom)) }
//     }
// }

// PreparedGeometry is Send+Sync, Geometry is Send+Sync. *mut Geometry is never modified.
// unsafe impl Send for Shapes {}
// unsafe impl Sync for Shapes {}

// `PreparededGeometry::contains` needs a call to `contains` before it is thread-safe:
// https://github.com/georust/geos/issues/95
fn warmup_prepped(prepped: &PreparedGeometry) {
    let point = CoordSeq::new_from_vec(&[&[0.0, 0.0]]).unwrap();
    let point = Geometry::create_point(point).unwrap();
    prepped.contains(&point).unwrap();
}

impl Clone for Shapes {
    fn clone(&self) -> Self {
        let geom = Clone::clone(&self.geom);
        let prepped = geom.to_prepared_geom().unwrap();
        warmup_prepped(&prepped);

        Shapes { geom, prepped }
    }
}

impl Shapes {
    pub fn from_geom(geom: Geometry) -> io::Result<Shapes> {
        // let bxd = Box::new(geom);
        // let gptr = Box::into_raw(bxd);
        // let prepped = unsafe { (&*gptr).to_prepared_geom() }
        //     .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "cannot prepare geomtry"))?;

        let prepped = geom
            .to_prepared_geom()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "cannot prepare geomtry"))?;
        warmup_prepped(&prepped);

        Ok(Shapes { geom, prepped })
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

        Ok(geos::Geometry::new_from_wkb(&buf).unwrap())
    }
}

#[pymethods]
impl Shapes {
    /// Make a new Gshhg shapes instance.
    #[staticmethod]
    pub fn new(py: Python, provider: LandmaskProvider) -> io::Result<Self> {
        let buf = Shapes::wkb(py, provider)?;
        let g = geos::Geometry::new_from_wkb(buf.as_bytes()).unwrap();
        Shapes::from_geom(g)
    }

    /// Get the WKB for the GSHHG shapes (full resolution).
    #[staticmethod]
    pub fn wkb(py: Python, provider: LandmaskProvider) -> io::Result<&PyBytes> {
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

        let point = CoordSeq::new_from_vec(&[&[x as f64, y as f64]]).unwrap();
        let point = Geometry::create_point(point).unwrap();
        self.prepped.contains(&point).unwrap()
    }

    /// Same as `contains`, but does not check for bounds.
    pub(crate) fn contains_unchecked(&self, x: f64, y: f64) -> bool {
        let point = CoordSeq::new_from_vec(&[&[x, y]]).unwrap();
        let point = Geometry::create_point(point).unwrap();
        self.prepped.contains(&point).unwrap()
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
        .to_owned()
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
        let contains = Zip::from(&x)
            .and(&y)
            .par_map_collect(|x, y| self.contains(*x, *y));
        PyArray::from_owned_array(py, contains).to_owned()
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
