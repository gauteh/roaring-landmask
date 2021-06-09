use roaring::RoaringTreemap;
use std::fs::File;
use std::io;
use std::path::Path;

pub const NY: u64 = 43200;
pub const NX: u64 = 86400;

lazy_static! {
    static ref TRANSFORM: Affine<f64> = Affine::make();
}

pub struct RoaringLandmask {
    tmap: RoaringTreemap,
}

pub struct Affine<T> {
    sa: T,
    sb: T,
    sc: T,
    sd: T,
    se: T,
    sf: T,
}

impl Affine<f64> {
    /// Makes the inverse transform for the landmask image. Goes from latitude, longitude
    /// coordinates to index in mask.
    pub fn make() -> Affine<f64> {
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

impl RoaringLandmask {
    pub fn from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let tmap = RoaringTreemap::deserialize_from(fd)?;

        Ok(RoaringLandmask { tmap })
    }

    pub fn from_compressed<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let fd = File::open(path)?;
        let fd = io::BufReader::new(fd);
        let fd = xz2::bufread::XzDecoder::new(fd);
        let tmap = RoaringTreemap::deserialize_from(fd)?;

        Ok(RoaringLandmask { tmap })
    }

    /// Check if point (x, y) is on land.
    ///
    /// `x` is longitude, [-180, 180] north
    /// `y` is latitude,  [- 90,  90] east
    ///
    /// The check is _optimistic_, it will yield `true` for points that are closer to the shore
    /// than the resolution of the landmask. The positive points should be checked against the
    /// vectorized land shapes.
    ///
    /// Returns `true` if the point is on land or close to the shore.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        debug_assert!(x > -180.);
        debug_assert!(y > -90.);

        let (x, y) = TRANSFORM.apply(x, y);
        let x = x as u64;
        let y = y as u64;

        debug_assert!(x < NX);
        debug_assert!(y < NY);

        self.tmap.contains(y * NX + x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn load_tmap(b: &mut Bencher) {
        b.iter(|| {
            let _mask = RoaringLandmask::from("mask.tbmap").unwrap();
        })
    }

    #[bench]
    fn load_tmap_compressed(b: &mut Bencher) {
        b.iter(|| {
            let _mask = RoaringLandmask::from_compressed("mask.tbmap.xz").unwrap();
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
        let mask = RoaringLandmask::from_compressed("mask.tbmap.xz").unwrap();

        assert!(mask.contains(15., 65.6));
        assert!(mask.contains(10., 60.0));

        b.iter(|| mask.contains(15., 65.6))
    }

    #[bench]
    fn test_contains_in_ocean(b: &mut Bencher) {
        let mask = RoaringLandmask::from_compressed("mask.tbmap.xz").unwrap();

        assert!(!mask.contains(5., 65.6));

        b.iter(|| mask.contains(5., 65.6))
    }

    #[bench]
    fn test_contains_many(b: &mut Bencher) {
        let mask = RoaringLandmask::from_compressed("mask.tbmap.xz").unwrap();

        let pts = (0..360*2)
            .map(|v| v as f64 * 0.5 - 180.)
            .map(|x| (0..180*2).map(|y| y as f64 * 0.5 - 90.).map(move |y| (x, y)))
            .flatten()
            .collect::<Vec<(f64, f64)>>();

        println!("testing {} points..", pts.len());

        b.iter(|| {
            let _onland = pts.iter().map(|(x, y)| mask.contains(*x, *y)).collect::<Vec<bool>>();
        })
    }
}
