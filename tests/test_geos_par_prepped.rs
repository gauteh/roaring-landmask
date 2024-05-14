use roaring_landmask::Gshhg;
use geos::{CoordSeq, Geom, Geometry};

#[ignore]
#[test]
fn test_par_prepped_no_warmup() {
    use rayon::prelude::*;

    // let s = Gshhg::new().unwrap();
    // let prepped = &s.prepped;

    let g = Gshhg::get_geometry_from_compressed(
        "gshhs/gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz",
    ).unwrap();
    let prepped = g.to_prepared_geom().unwrap();

    (0..10000).into_par_iter().for_each(|k| {

        let x = k % 180;
        let y = (k / 180) % 90;


        let point = CoordSeq::new_from_vec(&[&[x as f64, y as f64]]).unwrap();
        let point = Geometry::create_point(point).unwrap();
        prepped.contains(&point).unwrap();
    });
}

#[ignore]
#[test]
fn test_par_prepped_with_warmup() {
    use rayon::prelude::*;

    // let s = Gshhg::new().unwrap();
    // let prepped = &s.prepped;

    let g = Gshhg::get_geometry_from_compressed(
        "gshhs/gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz",
    ).unwrap();
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
