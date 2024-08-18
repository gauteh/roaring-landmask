use pyo3::prelude::*;
use std::fs::File;
use std::io;
use std::path::Path;
use std::{borrow::Borrow, convert::TryInto};

use geo::{point, Contains, Geometry, MultiPolygon, Point, Polygon, PreparedGeometry, Relate};
use numpy::{PyArray, PyReadonlyArrayDyn};
use rstar::{Envelope, PointDistance, RTree, RTreeObject, AABB};

pub static GSHHS_F: &str = "gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz";

#[pyclass]
#[derive(Clone)]
pub struct Gshhg {
    geom: RTree<PolW>,
}

#[derive(Clone)]
struct PolW {
    p: Polygon,
    e: AABB<Point<f64>>,
}

impl PolW {
    pub fn from(p: Polygon) -> PolW {
        PolW {
            e: p.envelope(),
            p: p,
        }
    }
}

impl RTreeObject for PolW {
    type Envelope = AABB<Point<f64>>;

    fn envelope(&self) -> Self::Envelope {
        self.e
    }
}

impl PointDistance for PolW {
    fn distance_2(&self, _point: &Point<f64>) -> f64 {
        panic!("this should only be used for contains, the distance will give the wrong answer");
    }

    fn contains_point(&self, point: &Point<f64>) -> bool {
        // fast contains from libgeos
        // https://github.com/libgeos/geos/blob/main/src/geom/prep/PreparedPolygonContainsProperly.cpp
        if !self.e.contains_point(point) {
            return false;
        }

        // self.p.relate(point).is_covers()
        self.p.contains(point)
    }

    fn distance_2_if_less_or_equal(&self, _point: &Point<f64>, _max_distance: f64) -> Option<f64> {
        panic!("this should only be used for contains, the distance will give the wrong answer");
    }
}

impl Gshhg {
    pub fn from_geom(geom: Geometry) -> io::Result<Gshhg> {
        let geom: MultiPolygon = geom.try_into().unwrap();
        assert!(geom.0.len() > 10);
        let geoms = geom.0.into_iter().map(|p| PolW::from(p)).collect();

        let geom = RTree::bulk_load(geoms);
        Ok(Gshhg { geom })
    }

    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Gshhg> {
        let g = Gshhg::get_geometry_from_compressed(path)?;

        Gshhg::from_geom(g)
    }

    pub fn get_geometry_from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Geometry> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let mut fd = xz2::bufread::XzDecoder::new(fd);
        let geom = wkb::wkb_to_geom(&mut fd).unwrap();
        Ok(geom)
    }

    pub fn geom_from_embedded() -> io::Result<Geometry> {
        use crate::GsshgData;

        let buf = GsshgData::get(&GSHHS_F)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find shapes"))?;
        let buf: &[u8] = buf.data.borrow();
        let mut fd = xz2::read::XzDecoder::new(buf);
        let geom = wkb::wkb_to_geom(&mut fd).unwrap();

        Ok(geom)
    }
}

#[pymethods]
impl Gshhg {
    /// Make a new Gshhg shapes instance.
    #[staticmethod]
    pub fn new(_py: Python) -> io::Result<Self> {
        Gshhg::from_geom(Gshhg::geom_from_embedded()?)
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] east
    /// `y` is latitude,  [- 90,  90] north
    ///
    /// Returns `true` if the point is on land.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let x = super::modulate_longitude(x);
        debug_assert!(x >= -180. && x <= 180.);
        assert!(y > -90. && y <= 90.);
        self.contains_unchecked(x, y)
    }

    /// Same as `contains`, but does not check for bounds.
    pub(crate) fn contains_unchecked(&self, x: f64, y: f64) -> bool {
        let p = point!(x: x, y: y);
        self.geom.locate_at_point(&p).is_some()
    }

    pub fn contains_many(
        &self,
        py: Python,
        x: PyReadonlyArrayDyn<f64>,
        y: PyReadonlyArrayDyn<f64>,
    ) -> Py<PyArray<bool, numpy::Ix1>> {
        let x = x.as_array();
        let y = y.as_array();

        PyArray::from_iter_bound(
            py,
            x.iter().zip(y.iter()).map(|(x, y)| self.contains(*x, *y)),
        )
        .to_owned()
        .into()
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
        PyArray::from_owned_array_bound(py, contains)
            .unbind()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_compressed() {
        let _s = Gshhg::from_compressed(
            "gshhs/gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz",
        )
        .unwrap();
    }

    #[test]
    fn test_load_embedded() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| Gshhg::new(py)).unwrap();
    }

    #[test]
    fn test_np() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mask = Gshhg::new(py).unwrap();
            assert!(!mask.contains(5., 90.));
        })
    }

    #[test]
    fn test_sp() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mask = Gshhg::new(py).unwrap();
            assert!(mask.contains(5., -89.99));
        })
    }

    #[test]
    fn prepare_geometry() {
        let geom = Gshhg::geom_from_embedded().unwrap();
        let prep = PreparedGeometry::from(geom);
    }

    #[cfg(feature = "nightly")]
    mod benches {
        use super::*;
        use test::Bencher;

        #[bench]
        fn test_contains_on_land(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let s = Gshhg::new(py).unwrap();

                assert!(s.contains(15., 65.6));
                assert!(s.contains(10., 60.0));

                b.iter(|| s.contains(15., 65.6))
            })
        }

        #[bench]
        fn test_contains_in_ocean(b: &mut Bencher) {
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let s = Gshhg::new(py).unwrap();

                assert!(!s.contains(5., 65.6));

                b.iter(|| s.contains(5., 65.6))
            })
        }
    }
}
