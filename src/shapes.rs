use pyo3::{prelude::*, types::PyBytes};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, prelude::*, Cursor};
use std::path::Path;

use geo::{point, Contains, Geometry};
use numpy::{PyArray, PyReadonlyArrayDyn};

pub static GSHHS_F: &str = "gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz";

#[pyclass]
#[derive(Clone)]
pub struct Gshhg {
    geom: Geometry,
}

impl Gshhg {
    pub fn from_geom(geom: Geometry) -> io::Result<Gshhg> {
        Ok(Gshhg { geom })
    }

    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Gshhg> {
        let g = Gshhg::get_geometry_from_compressed(path)?;

        Gshhg::from_geom(g)
    }

    pub fn get_geometry_from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Geometry> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let mut fd = xz2::bufread::XzDecoder::new(fd);
        let geom = wkb::wkb_to_geom(&mut fd).unwrap();
        Ok(geom)
    }
}

#[pymethods]
impl Gshhg {
    /// Make a new Gshhg shapes instance.
    #[staticmethod]
    pub fn new(py: Python) -> io::Result<Self> {
        use crate::GsshgData;

        let buf = GsshgData::get(&GSHHS_F)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find shapes"))?;
        let buf: &[u8] = buf.data.borrow();
        let mut fd = xz2::read::XzDecoder::new(buf);
        let geom = wkb::wkb_to_geom(&mut fd).unwrap();
        Gshhg::from_geom(geom)
    }

    /// Get the WKB for the GSHHG shapes (full resolution).
    #[staticmethod]
    pub fn wkb(py: Python) -> io::Result<&PyBytes> {
        use crate::GsshgData;

        let buf = GsshgData::get(&GSHHS_F)
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

        let p = point!(x: x, y: y);
        self.geom.contains(&p)
    }

    /// Same as `contains`, but does not check for bounds.
    pub(crate) fn contains_unchecked(&self, x: f64, y: f64) -> bool {
        let p = point!(x: x, y: y);
        self.geom.contains(&p)
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
        let _s = Gshhg::from_compressed(
            "gshhs/gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz",
        )
        .unwrap();
    }

    #[test]
    fn test_load() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| Gshhg::new(py)).unwrap();
    }

    #[test]
    fn test_np() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mask = Gshhg::new(py).unwrap();
            assert!(!mask.contains(5., 90.));
        })
    }

    #[test]
    fn test_sp() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mask = Gshhg::new(py).unwrap();
            assert!(mask.contains(5., -89.99));
        })
    }

    #[cfg(feature = "nightly")]
    mod benches {
        use super::*;
        use test::Bencher;

        #[bench]
        fn test_contains_on_land(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let s = Gshhg::new(py).unwrap();

                assert!(s.contains(15., 65.6));
                assert!(s.contains(10., 60.0));

                b.iter(|| s.contains(15., 65.6))
            })
        }

        #[bench]
        fn test_contains_in_ocean(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let s = Gshhg::new(py).unwrap();

                assert!(!s.contains(5., 65.6));

                b.iter(|| s.contains(5., 65.6))
            })
        }
    }
}
