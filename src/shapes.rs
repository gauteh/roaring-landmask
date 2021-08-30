use pyo3::prelude::*;
use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;

use geos::{CoordSeq, Geom, Geometry, PreparedGeometry};
use numpy::{PyArray, PyReadonlyArrayDyn};

pub static GSHHS_F: &str = "gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz";

#[pyclass]
pub struct Gshhg {
    geom: *mut Geometry<'static>,

    // prepped requires `geom` above to be around, and is valid as long as geom is alive.
    prepped: PreparedGeometry<'static>,
}

impl Drop for Gshhg {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.geom)) }
    }
}

// PreparedGeometry is Send, Geometry is Send. *mut Geometry is never modified.
unsafe impl Send for Gshhg {}

impl Clone for Gshhg {
    fn clone(&self) -> Self {
        let gptr = self.geom.clone();
        debug_assert!(gptr != self.geom);
        let prepped = unsafe { (&*gptr).to_prepared_geom().unwrap() };

        Gshhg {
            geom: gptr,
            prepped,
        }
    }
}

impl Gshhg {
    pub fn from_geom(geom: Geometry<'static>) -> io::Result<Gshhg> {
        let bxd = Box::new(geom);
        let gptr = Box::into_raw(bxd);
        let prepped = unsafe { (&*gptr).to_prepared_geom() }
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "cannot prepare geomtry"))?;

        Ok(Gshhg {
            geom: gptr,
            prepped,
        })
    }

    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Gshhg> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let mut fd = xz2::bufread::XzDecoder::new(fd);
        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;

        let g = geos::Geometry::new_from_wkb(&buf).unwrap();

        Gshhg::from_geom(g)
    }
}

#[pymethods]
impl Gshhg {
    #[staticmethod]
    /// Make a new Gshhg shapes instance.
    pub fn new() -> io::Result<Self> {
        use crate::GsshgData;

        let buf = GsshgData::get(&GSHHS_F)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find shapes"))?;
        let buf: &[u8] = buf.borrow();
        let mut fd = xz2::read::XzDecoder::new(buf);

        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;

        let g = geos::Geometry::new_from_wkb(&buf).unwrap();
        Gshhg::from_geom(g)
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] north
    /// `y` is latitude,  [- 90,  90] east
    ///
    /// Returns `true` if the point is on land.
    pub fn contains(&self, x: f64, y: f64) -> bool {
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

        PyArray::from_exact_iter(
            py,
            x.iter().zip(y.iter()).map(|(x, y)| self.contains(*x, *y)),
        )
        .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_load_compressed() {
        let _s = Gshhg::from_compressed(
            "gshhs/gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz",
        )
        .unwrap();
    }

    #[test]
    fn test_load() {
        let _s = Gshhg::new().unwrap();
    }

    #[bench]
    fn test_contains_on_land(b: &mut Bencher) {
        let s = Gshhg::new().unwrap();

        assert!(s.contains(15., 65.6));
        assert!(s.contains(10., 60.0));

        b.iter(|| s.contains(15., 65.6))
    }

    #[bench]
    fn test_contains_in_ocean(b: &mut Bencher) {
        let s = Gshhg::new().unwrap();

        assert!(!s.contains(5., 65.6));

        b.iter(|| s.contains(5., 65.6))
    }
}
