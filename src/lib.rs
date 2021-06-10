#![feature(test)]
extern crate test;

#[macro_use]
extern crate lazy_static;

use pyo3::prelude::*;
use std::io;
use numpy::{PyArray, PyReadonlyArrayDyn};

pub mod shapes;
pub mod mask;

pub use mask::RoaringMask;
pub use shapes::Gshhg;

#[pymodule]
fn roaring_landmask(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<mask::Affine>()?;
    m.add_class::<RoaringMask>()?;
    m.add_class::<Gshhg>()?;
    m.add_class::<RoaringLandmask>()?;

    Ok(())
}

#[pyclass]
pub struct RoaringLandmask {
    #[pyo3(get)]
    pub mask: RoaringMask,
    #[pyo3(get)]
    pub shapes: shapes::Gshhg
}

#[pymethods]
impl RoaringLandmask {
    #[staticmethod]
    pub fn new() -> io::Result<RoaringLandmask> {
        let mask = RoaringMask::from_compressed("mask.tbmap.xz")?;
        let shapes = shapes::Gshhg::from_compressed(&shapes::GSHHS_F)?;

        Ok(RoaringLandmask { mask, shapes })
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        self.mask.contains(x,y) && self.shapes.contains(x, y)
    }

    pub fn contains_many(&self, py: Python, x: PyReadonlyArrayDyn<f64>, y: PyReadonlyArrayDyn<f64>) -> Py<PyArray<bool, numpy::Ix1>> {
        let x = x.as_array();
        let y = y.as_array();

        PyArray::from_exact_iter(
            py,
            x.iter().zip(y.iter()).map(|(x, y)| self.contains(*x, *y))
        ).to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn load_ms() {
        let _ms = RoaringLandmask::new().unwrap();
    }

    #[bench]
    fn test_contains_on_land(b: &mut Bencher) {
        let mask = RoaringLandmask::new().unwrap();

        assert!(mask.contains(15., 65.6));
        assert!(mask.contains(10., 60.0));

        b.iter(|| mask.contains(15., 65.6))
    }

    #[bench]
    fn test_contains_in_ocean(b: &mut Bencher) {
        let mask = RoaringLandmask::new().unwrap();

        assert!(!mask.contains(5., 65.6));

        b.iter(|| mask.contains(5., 65.6))
    }
}
