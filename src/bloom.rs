use pyo3::prelude::*;
use roaring_bloom_filter::BloomFilter;

use crate::mask::{NY, NX, TRANSFORM, RoaringMask};

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct RoaringBloom {
    filter: BloomFilter,
}

#[pymethods]
impl RoaringBloom {
    #[staticmethod]
    pub fn new() -> io::Result<Self> {
        todo!()
    }

    #[getter]
    pub fn dx(&self) -> f64 {
        (180f64 - (-180f64)) / (NX as f64)
    }

    #[getter]
    pub fn dy(&self) -> f64 {
        (180f64 - (-180f64)) / (NX as f64)
    }

}
