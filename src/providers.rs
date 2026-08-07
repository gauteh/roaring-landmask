use pyo3::prelude::*;

#[pyclass(from_py_object)]
#[derive(Debug, Clone, Copy)]
pub enum LandmaskProvider {
    Gshhg,
    Osm,
}
