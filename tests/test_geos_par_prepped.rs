use geo::{Geometry, Point, PreparedGeometry, Relate};
use roaring_landmask::Shapes;
use std::sync::Arc;

pub static GSHHG: &str = "assets/gshhg.wkb.xz";
pub static OSM: &str = "assets/osm.wkb.xz";

/// Thin wrapper that asserts Sync for PreparedGeometry.
/// Safe because `relate()` only takes shared references and performs no mutation.
struct SyncPrepped(PreparedGeometry<'static, Geometry>);
unsafe impl Sync for SyncPrepped {}

/// Test that PreparedGeometry can be shared read-only across threads via Arc.
/// With geo's PreparedGeometry (Send via PR georust/geo#1571), no warmup is needed.
#[ignore]
#[test]
fn test_par_prepped() {
    use rayon::prelude::*;

    for landmask_path in [GSHHG, OSM] {
        let g = Shapes::get_geometry_from_compressed(landmask_path).unwrap();
        let prepped = Arc::new(SyncPrepped(PreparedGeometry::from(g)));

        (0..10000).into_par_iter().for_each(|k| {
            let x = (k % 180) as f64;
            let y = ((k / 180) % 89 + 1) as f64;
            let point = Point::new(x, y);
            prepped.0.relate(&point).is_contains();
        });
    }
}

