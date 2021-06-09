#![feature(test)]
extern crate test;

#[macro_use]
extern crate lazy_static;

use pyo3::prelude::*;
use std::io;
use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn};

pub mod shapes;
pub mod mask;
pub use mask::{Affine, RoaringLandmask};

#[pymodule]
fn roaring_landmask(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "im_alive")]
    fn im_alive(_py: Python) -> PyResult<bool> {
        Ok(true)
    }

    m.add_class::<MaskShape>()?;

    Ok(())
}

#[pyclass]
pub struct MaskShape {
    mask: RoaringLandmask,
    shapes: shapes::Gshhg
}

#[pymethods]
impl MaskShape {
    #[staticmethod]
    pub fn new() -> io::Result<MaskShape> {
        let mask = RoaringLandmask::from_compressed("mask.tbmap.xz")?;
        let shapes = shapes::Gshhg::from_compressed(&shapes::GSHHS_F)?;

        Ok(MaskShape { mask, shapes })
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        self.mask.contains(x,y) && self.shapes.contains(x, y)
    }

    pub fn contains_many(&self, x: PyReadonlyArrayDyn<f64>, y: PyReadonlyArrayDyn<f64>) -> Vec<bool> {
        let x = x.as_array();
        let y = y.as_array();

        x.iter().zip(y.iter()).map(|(x, y)| self.contains(*x, *y)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn load_ms() {
        let _ms = MaskShape::new().unwrap();
    }

    #[bench]
    fn test_contains_on_land(b: &mut Bencher) {
        let mask = MaskShape::new().unwrap();

        assert!(mask.contains(15., 65.6));
        assert!(mask.contains(10., 60.0));

        b.iter(|| mask.contains(15., 65.6))
    }

    #[bench]
    fn test_contains_in_ocean(b: &mut Bencher) {
        let mask = MaskShape::new().unwrap();

        assert!(!mask.contains(5., 65.6));

        b.iter(|| mask.contains(5., 65.6))
    }

    // #[bench]
    // fn test_contains_many(b: &mut Bencher) {
    //     let mask = MaskShape::new().unwrap();

    //     let pts = (0..360*2)
    //         .map(|v| v as f64 * 0.5 - 180.)
    //         .map(|x| (0..180*2).map(|y| y as f64 * 0.5 - 90.).map(move |y| (x, y)))
    //         .flatten()
    //         .collect::<Vec<(f64, f64)>>();

    //     println!("testing {} points..", pts.len());

    //     b.iter(|| {
    //         let _onland = pts.iter().map(|(x, y)| mask.contains(*x, *y)).collect::<Vec<bool>>();
    //     })
    // }
}
