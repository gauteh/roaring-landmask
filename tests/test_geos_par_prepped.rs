use roaring_landmask::Shapes;

pub static GSHHG: &str = "assets/gshhg.wkb.xz";
pub static OSM: &str = "assets/osm.wkb.xz";

/// Test that Shapes can be queried from multiple threads in parallel.
/// IntervalTreeMultiPolygon is Send+Sync, so no unsafe or Arc wrapper needed.
#[ignore]
#[test]
fn test_par_prepped() {
    use rayon::prelude::*;

    for landmask_path in [GSHHG, OSM] {
        let shapes = Shapes::from_compressed(landmask_path).unwrap();

        (0..10000).into_par_iter().for_each(|k| {
            let x = (k % 180) as f64;
            let y = ((k / 180) % 89 + 1) as f64;
            shapes.contains_unchecked(x, y);
        });
    }
}

