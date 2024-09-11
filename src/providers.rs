use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, Copy)]
pub enum LandmaskProvider {
    Gshhg,
    Osm,
}
