//! # The Roaring Landmask
//!
//! Have you ever needed to know whether you are in the ocean or on land? And you
//! need to know it fast? And you need to know it without using too much memory or
//! too much disk? Then try the _Roaring Landmask_!
//!
//! The _roaring landmask_ is a Rust + Python package for quickly determining
//! whether a point given in latitude and longitude is on land or not. A landmask
//! is stored in a tree of [Roaring Bitmaps](https://roaringbitmap.org/). Points
//! close to the shore might still be in the ocean, so a positive
//! value is then checked against the vector shapes of the coastline.
//!
//! <img src="https://raw.githubusercontent.com/gauteh/roaring-landmask/main/the_earth.png" width="50%" />
//!
//! ([source](https://github.com/gauteh/roaring-landmask/blob/main/src/devel/make_demo_plot.py))
//!
//! The landmask is generated from the [GSHHG shoreline database](https://www.soest.hawaii.edu/pwessel/gshhg/) (Wessel, P., and W. H. F. Smith, A Global Self-consistent, Hierarchical, High-resolution Shoreline Database, J. Geophys. Res., 101, 8741-8743, 1996).
//!
//! ## Usage
//!
//! ```
//! # use std::io;
//! # fn main() -> io::Result<()> {
//! #
//! # pyo3::prepare_freethreaded_python();
//! # pyo3::Python::with_gil(|py| {
//! use roaring_landmask::RoaringLandmask;
//!
//! let mask = RoaringLandmask::new(py).unwrap();
//!
//! // Check some points on land
//! assert!(mask.contains(15., 65.6));
//! assert!(mask.contains(10., 60.0));
//!
//! // Check a point in the ocean
//! assert!(!mask.contains(5., 65.6));
//! # });
//! #
//! # Ok(())
//! # }
//! ```
//!
//! or in Python:
//!
//! ```python
//! from roaring_landmask import RoaringLandmask
//!
//! l = RoaringLandmask.new()
//! x = np.arange(-180, 180, .5)
//! y = np.arange(-90, 90, .5)
//!
//! xx, yy = np.meshgrid(x,y)
//!
//! print ("points:", len(xx.ravel()))
//! on_land = l.contains_many(xx.ravel(), yy.ravel())
//! ```

#![cfg_attr(feature = "nightly", feature(test))]
#[cfg(feature = "nightly")]
extern crate test;

#[macro_use]
extern crate lazy_static;

// geos-sys needs libc++, probably libstdc++. On Windows Conda builds this hopefully adds the
// correct flags to the linker.
// extern crate link_cplusplus;

use numpy::{PyArray, PyReadonlyArrayDyn};
use pyo3::prelude::*;
use std::io;

pub mod mask;
pub mod shapes;

pub use mask::RoaringMask;
pub use shapes::Gshhg;

include!(concat!(env!("OUT_DIR"), "/gshhs.rs"));

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
    pub shapes: shapes::Gshhg,
}

#[pymethods]
impl RoaringLandmask {
    #[staticmethod]
    pub fn new(py: Python) -> io::Result<RoaringLandmask> {
        let mask = RoaringMask::new()?;
        let shapes = Gshhg::new(py)?;

        Ok(RoaringLandmask { mask, shapes })
    }

    #[getter]
    pub fn dx(&self) -> f64 {
        self.mask.dx()
    }

    #[getter]
    pub fn dy(&self) -> f64 {
        self.mask.dy()
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] east
    /// `y` is latitude,  [- 90,  90] north
    ///
    ///
    /// Returns `true` if the point is on land or close to the shore.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        assert!(y >= -90. && y <= 90.);

        let x = modulate_longitude(x);

        self.mask.contains_unchecked(x, y) && self.shapes.contains_unchecked(x, y)
    }

    fn contains_many(
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

/// Move longitude into -180 to 180 domain.
fn modulate_longitude(lon: f64) -> f64 {
    ((lon + 180.) % 360.) - 180.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_ms() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let _ms = RoaringLandmask::new(py).unwrap();
        })
    }

    #[test]
    fn test_np() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let mask = RoaringLandmask::new(py).unwrap();
            assert!(!mask.contains(5., 90.));
        })
    }

    #[test]
    fn test_sp() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let mask = RoaringLandmask::new(py).unwrap();
            assert!(mask.contains(5., -89.99));
        })
    }

    #[test]
    #[should_panic]
    fn test_sp_oob() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let mask = RoaringLandmask::new(py).unwrap();
            assert!(mask.contains(5., -90.));
        })
    }

    #[test]
    fn test_dateline_wrap() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let mask = RoaringLandmask::new(py).unwrap();

            // Close to NP
            assert!(!mask.contains(5., 89.));

            // Close to SP
            assert!(mask.contains(5., -89.));

            // Within bounds
            let x = (-180..180).map(f64::from).collect::<Vec<_>>();
            let m = x.iter().map(|x| mask.contains(*x, 65.)).collect::<Vec<_>>();

            // Wrapped bounds
            let x = (180..540).map(f64::from).collect::<Vec<_>>();
            let mm = x.iter().map(|x| mask.contains(*x, 65.)).collect::<Vec<_>>();

            assert_eq!(m, mm);
        })
    }

    #[test]
    #[should_panic]
    fn test_not_on_earth_north() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let mask = RoaringLandmask::new(py).unwrap();
            assert!(!mask.contains(5., 95.));
        })
    }

    #[test]
    #[should_panic]
    fn test_not_on_earth_south() {
        pyo3::prepare_freethreaded_python();
        pyo3::Python::with_gil(|py| {
            let mask = RoaringLandmask::new(py).unwrap();
            assert!(!mask.contains(5., -95.));
        })
    }

    #[cfg(feature = "nightly")]
    mod benches {
        use super::*;
        use test::Bencher;

        #[bench]
        fn test_contains_on_land(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            pyo3::Python::with_gil(|py| {
                let mask = RoaringLandmask::new(py).unwrap();

                assert!(mask.contains(15., 65.6));
                assert!(mask.contains(10., 60.0));

                b.iter(|| mask.contains(15., 65.6))
            })
        }

        #[bench]
        fn test_contains_in_ocean(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            pyo3::Python::with_gil(|py| {
                let mask = RoaringLandmask::new(py).unwrap();

                assert!(!mask.contains(5., 65.6));

                b.iter(|| mask.contains(5., 65.6));
            });
        }

        #[bench]
        fn test_contains_many(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            pyo3::Python::with_gil(|py| {
                let mask = RoaringLandmask::new(py).unwrap();

                let (x, y): (Vec<f64>, Vec<f64>) = (0..360 * 2)
                    .map(|v| v as f64 * 0.5 - 180.)
                    .map(|x| {
                        (0..180 * 2)
                            .map(|y| y as f64 * 0.5 - 90.)
                            .map(move |y| (x, y))
                    })
                    .flatten()
                    .unzip();

                let mask = RoaringLandmask::new(py).unwrap();

                let (x, y): (Vec<f64>, Vec<f64>) = (0..360 * 2)
                    .map(|v| v as f64 * 0.5 - 180.)
                    .map(|x| {
                        (0..180 * 2)
                            .map(|y| y as f64 * 0.5 - 90.)
                            .map(move |y| (x, y))
                    })
                    .flatten()
                    .unzip();

                let x = PyArray::from_vec(py, x);
                let y = PyArray::from_vec(py, y);

                println!("testing {} points..", x.len());

                b.iter(|| {
                    let len = x.len();

                    let x = x.to_dyn().readonly();
                    let y = y.to_dyn().readonly();

                    let onland = mask.contains_many(py, x, y);
                    assert!(onland.as_ref(py).len() == len);
                })
            })
        }

        #[bench]
        #[ignore]
        fn test_contains_many_par(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            pyo3::Python::with_gil(|py| {
                let mask = RoaringLandmask::new(py).unwrap();

                let (x, y): (Vec<f64>, Vec<f64>) = (0..360 * 2)
                    .map(|v| v as f64 * 0.5 - 180.)
                    .map(|x| {
                        (0..180 * 2)
                            .map(|y| y as f64 * 0.5 - 90.)
                            .map(move |y| (x, y))
                    })
                    .flatten()
                    .unzip();

                let mask = RoaringLandmask::new(py).unwrap();

                let (x, y): (Vec<f64>, Vec<f64>) = (0..360 * 2)
                    .map(|v| v as f64 * 0.5 - 180.)
                    .map(|x| {
                        (0..180 * 2)
                            .map(|y| y as f64 * 0.5 - 90.)
                            .map(move |y| (x, y))
                    })
                    .flatten()
                    .unzip();

                let x = PyArray::from_vec(py, x);
                let y = PyArray::from_vec(py, y);

                println!("testing {} points..", x.len());

                b.iter(|| {
                    let len = x.len();

                    let x = x.to_dyn().readonly();
                    let y = y.to_dyn().readonly();

                    let onland = mask.contains_many_par(py, x, y);
                    assert!(onland.as_ref(py).len() == len);
                })
            })
        }
    }
}
