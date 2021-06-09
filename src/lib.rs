#![feature(test)]
extern crate test;

#[macro_use]
extern crate lazy_static;

use pyo3::prelude::*;

pub mod mask;
pub use mask::{Affine, RoaringLandmask};

#[pymodule]
fn roaring_landmask(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "im_alive")]
    fn im_alive(_py: Python) -> PyResult<bool> {
        Ok(true)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
