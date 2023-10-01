use numpy::{PyArray, PyReadonlyArrayDyn};
use pyo3::prelude::*;
use roaring::RoaringTreemap;
use std::borrow::Borrow;
use std::fs::File;
use std::io;
use std::path::Path;

pub const NY: u64 = 43200;
pub const NX: u64 = 86400;

lazy_static! {
    static ref TRANSFORM: Affine = Affine::make();
}

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct RoaringMask {
    tmap: RoaringTreemap,
}

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct Affine {
    #[pyo3(get)]
    sa: f64,
    #[pyo3(get)]
    sb: f64,
    #[pyo3(get)]
    sc: f64,
    #[pyo3(get)]
    sd: f64,
    #[pyo3(get)]
    se: f64,
    #[pyo3(get)]
    sf: f64,
}

#[pymethods]
impl Affine {
    /// Makes the inverse transform for the landmask image. Goes from latitude, longitude
    /// coordinates to index in mask.
    #[staticmethod]
    pub fn make() -> Affine {
        // Forward transformation is declared as follows:
        //
        // let resx: f64 = (180f64 - (-180f64)) / (NX as f64);
        // let resy: f64 = (90f64 - (-90f64)) / (NY as f64);

        // let tx: f64 = -180f64 - resx / 2.;
        // let ty: f64 = -90f64 - resy / 2.;

        Affine {
            sa: 240.0,
            sb: -0.0,
            sc: 43200.5,
            sd: -0.0,
            se: 240.0,
            sf: 21600.5,
        }
    }

    /// Transform longitude and latitude to index in landmask.
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (
            x * self.sa + y * self.sb + self.sc,
            x * self.sd + y * self.se + self.sf,
        )
    }
}

impl RoaringMask {
    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let fd = xz2::bufread::XzDecoder::new(fd);
        let tmap = RoaringTreemap::deserialize_from(fd)?;

        Ok(RoaringMask { tmap })
    }
}

#[pymethods]
impl RoaringMask {
    #[staticmethod]
    /// Make a new mask.
    pub fn new() -> io::Result<Self> {
        use crate::GsshgData;

        let buf = GsshgData::get("mask.tbmap.xz")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find mask"))?;
        let buf: &[u8] = buf.data.borrow();

        let fd = xz2::read::XzDecoder::new(buf);
        let tmap = RoaringTreemap::deserialize_unchecked_from(fd)?;

        Ok(RoaringMask { tmap })
    }

    #[getter]
    pub fn dx(&self) -> f64 {
        (180f64 - (-180f64)) / (NX as f64)
    }

    #[getter]
    pub fn dy(&self) -> f64 {
        (180f64 - (-180f64)) / (NX as f64)
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] east
    /// `y` is latitude,  [- 90,  90] north
    ///
    /// The check is _optimistic_, it will yield `true` for points that are closer to the shore
    /// than the resolution of the landmask. The positive points should be checked against the
    /// vectorized land shapes.
    ///
    /// Returns `true` if the point is on land or close to the shore.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let x = super::modulate_longitude(x);
        debug_assert!(x >= -180. && x <= 180.);
        assert!(y >= -90.);

        let (x, y) = TRANSFORM.apply(x, y);
        let x = x as u64;
        let y = y as u64;

        // Special case where we are in northernmost cell. North Pole is always in ocean anyway.
        if y == NY {
            return false;
        }

        debug_assert!(x < NX);
        assert!(y < NY);

        self.tmap.contains(y * NX + x)
    }

    /// Same as `contains`, but does not check for bounds.
    pub(crate) fn contains_unchecked(&self, x: f64, y: f64) -> bool {
        let (x, y) = TRANSFORM.apply(x, y);
        let x = x as u64;
        let y = y as u64;
        self.tmap.contains(y * NX + x)
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
    fn required_size() {
        println!("upper bound coordinate system: {}", NY * NX);

        let mask = RoaringMask::new().unwrap();
        println!("maximum in tree: {:?}", mask.tmap.max());

        assert!(mask.tmap.max().unwrap() <= std::u32::MAX as u64);
    }

    #[test]
    fn test_np() {
        let mask = RoaringMask::new().unwrap();
        assert!(!mask.contains(5., 90.));
    }

    #[test]
    fn test_sp() {
        let mask = RoaringMask::new().unwrap();
        assert!(mask.contains(5., -90.));
    }

    #[cfg(feature = "nightly")]
    mod benches {
        use super::*;
        use test::Bencher;

        #[bench]
        fn load_tmap(b: &mut Bencher) {
            b.iter(|| {
                let _mask = RoaringMask::new().unwrap();
            })
        }

        #[bench]
        fn load_tmap_compressed(b: &mut Bencher) {
            b.iter(|| {
                let _mask = RoaringMask::from_compressed("gshhs/mask.tbmap.xz").unwrap();
            })
        }

        #[bench]
        fn test_inv_transform(be: &mut Bencher) {
            // let a = Affine::make();
            let a = &TRANSFORM;

            let b = a.apply(40.5, 87.);
            println!("{:?}", b);
            assert_eq!(b.0, 52920.5);
            assert_eq!(b.1, 42480.5);

            let c = a.apply(0., 30.);
            assert_eq!(c.0, 43200.5);
            assert_eq!(c.1, 28800.5);

            be.iter(|| a.apply(40.5, 87.))
        }

        #[bench]
        fn test_contains_on_land(b: &mut Bencher) {
            let mask = RoaringMask::new().unwrap();

            assert!(mask.contains(15., 65.6));
            assert!(mask.contains(10., 60.0));

            b.iter(|| mask.contains(15., 65.6))
        }

        #[bench]
        fn test_contains_in_ocean(b: &mut Bencher) {
            let mask = RoaringMask::new().unwrap();

            assert!(!mask.contains(5., 65.6));

            b.iter(|| mask.contains(5., 65.6))
        }

        #[bench]
        fn test_contains_many(b: &mut Bencher) {
            let mask = RoaringMask::new().unwrap();

            let (x, y): (Vec<f64>, Vec<f64>) = (0..360 * 2)
                .map(|v| v as f64 * 0.5 - 180.)
                .map(|x| {
                    (0..180 * 2)
                        .map(|y| y as f64 * 0.5 - 90.)
                        .map(move |y| (x, y))
                })
                .flatten()
                .unzip();

            println!("testing {} points..", x.len());

            b.iter(|| {
                let _onland = x
                    .iter()
                    .zip(y.iter())
                    .map(|(x, y)| mask.contains(*x, *y))
                    .collect::<Vec<bool>>();
            })
        }
    }
}
