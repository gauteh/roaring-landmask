use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Copy)]
pub enum LandmaskProvider {
    Gshhg,
    Osm
}
