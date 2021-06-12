use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use std::pin::Pin;

use geos::{CoordSeq, Geom, Geometry, PreparedGeometry};

pub static GSHHS_F: &str = "gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz";

pub struct Gshhg<'a> {
    // this needs to stay at the same location in memory for
    // PreparedGeometry's references to it to be valid.
    #[allow(dead_code)]
    geom: Pin<Box<Geometry<'a>>>,
    prepped: PreparedGeometry<'a>,
}

unsafe impl<'a> Send for Gshhg<'a> {}

impl<'a> Clone for Gshhg<'a> {
    fn clone(&self) -> Self {
        let geom = Clone::clone(&self.geom);
        let prepped = geom.to_prepared_geom().unwrap();
        let prepped = unsafe { std::mem::transmute(prepped) };

        Gshhg { geom, prepped }
    }
}

impl<'a> Gshhg<'a> {
    pub fn from_geom(g: Geometry<'a>) -> io::Result<Gshhg<'a>> {
        let geom = Box::pin(g);
        let prepped = geom.to_prepared_geom().unwrap();
        let prepped = unsafe { std::mem::transmute(prepped) };

        Ok(Gshhg { geom, prepped })
    }

    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Gshhg<'a>> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let mut fd = xz2::bufread::XzDecoder::new(fd);
        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;

        let g = geos::Geometry::new_from_wkb(&buf).unwrap();

        Gshhg::from_geom(g)
    }
}

impl<'a> Gshhg<'a> {
    /// Make a new Gshhg shapes instance.
    pub fn new() -> io::Result<Self> {
        use crate::GsshgData;

        let buf = GsshgData::get(&GSHHS_F)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot find shapes"))?;
        let buf: &[u8] = buf.borrow();
        let mut fd = xz2::read::XzDecoder::new(buf);

        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;

        let g = geos::Geometry::new_from_wkb(&buf).unwrap();

        Gshhg::from_geom(g)
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] north
    /// `y` is latitude,  [- 90,  90] east
    ///
    /// Returns `true` if the point is on land.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let point = CoordSeq::new_from_vec(&[&[x, y]]).unwrap();
        let point = Geometry::create_point(point).unwrap();
        self.prepped.contains(&point).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_load_compressed() {
        let _s = Gshhg::from_compressed(
            "gshhs/gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz",
        )
        .unwrap();
    }

    #[test]
    fn test_load() {
        let _s = Gshhg::new().unwrap();
    }

    #[bench]
    fn test_contains_on_land(b: &mut Bencher) {
        let s = Gshhg::new().unwrap();

        assert!(s.contains(15., 65.6));
        assert!(s.contains(10., 60.0));

        b.iter(|| s.contains(15., 65.6))
    }

    #[bench]
    fn test_contains_in_ocean(b: &mut Bencher) {
        let s = Gshhg::new().unwrap();

        assert!(!s.contains(5., 65.6));

        b.iter(|| s.contains(5., 65.6))
    }
}
