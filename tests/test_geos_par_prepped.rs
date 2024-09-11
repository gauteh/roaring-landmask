use geos::{CoordSeq, Geom, Geometry};
use roaring_landmask::Shapes;

pub static GSHHG: &str = "assets/gshhg.wkb.xz";
pub static OSM: &str = "assets/osm.wkb.xz";

#[ignore]
#[test]
fn test_par_prepped_no_warmup() {
    use rayon::prelude::*;

    for landmask_path in [GSHHG, OSM] {
        let g = Shapes::get_geometry_from_compressed(landmask_path).unwrap();
        let prepped = g.to_prepared_geom().unwrap();
        (0..10000).into_par_iter().for_each(|k| {
            let x = k % 180;
            let y = (k / 180) % 90;

            let point = CoordSeq::new_from_vec(&[&[x as f64, y as f64]]).unwrap();
            let point = Geometry::create_point(point).unwrap();
            prepped.contains(&point).unwrap();
        });
    }
}

#[ignore]
#[test]
fn test_par_prepped_with_warmup() {
    use rayon::prelude::*;

    for landmask_path in [GSHHG, OSM] {
        let g = Shapes::get_geometry_from_compressed(landmask_path).unwrap();
        let prepped = g.to_prepared_geom().unwrap();

        let point = CoordSeq::new_from_vec(&[&[10., 50.]]).unwrap();
        let point = Geometry::create_point(point).unwrap();
        prepped.contains(&point).unwrap();

        (0..10000).into_par_iter().for_each(|k| {
            let x = k % 180;
            let y = (k / 180) % 90;

            let point = CoordSeq::new_from_vec(&[&[x as f64, y as f64]]).unwrap();
            let point = Geometry::create_point(point).unwrap();
            prepped.contains(&point).unwrap();
        });
    }
}
