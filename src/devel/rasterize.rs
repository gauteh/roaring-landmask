use geos::{CoordSeq, Geom, Geometry};
use rayon::prelude::*;
use roaring::*;
use roaring_landmask::Gshhg;
use std::fs::File;
use std::io::prelude::*;

fn make_box(x0: f64, y0: f64, dx: f64, dy: f64) -> Geometry<'static> {
    let coords = CoordSeq::new_from_vec(&[
        &[y0, x0],
        &[y0 + dy, x0],
        &[y0 + dy, x0 + dx],
        &[y0, x0 + dx],
        &[y0, x0],
    ])
    .unwrap();

    let ext = Geometry::create_linear_ring(coords).unwrap();

    Geometry::create_polygon(ext, Vec::new()).unwrap()
}

fn main() -> std::io::Result<()> {
    println!("opening shapes..");
    let shp = Gshhg::new()?;

    let ny = 43200;
    let nx = 86400;

    let (y0, y1) = (-90.0f64, 90.0f64);
    let (x0, x1) = (-180.0f64, 180.0f64);

    let dy = (y1 - y0) / ny as f64;
    let dx = (x1 - x0) / nx as f64;

    println!("ny, nx = {}, {}", ny, nx);

    // 1. grid into boxes
    // 2. find boxes that touch land (intersects)
    // 3. find boxes that are _not_ fully within land
    // 4. find the boxes that are in both 2. and 3.
    // 5. store the boxes from 2. and 4.

    println!("making boxes..");
    let pixels = (0..ny).map(|iy| (0..nx).map(move |ix| {
        let y = y0 + dy * iy as f64;
        let x = x0 + dx * ix as f64;
        assert!(y < y1);
        assert!(x < x1);

        let idx = iy * nx + ix;

        (idx, make_box(x, y, dx, dy))
    })).flatten();

    let prepped = &shp.prepped;

    println!("checking pixels..");
    let (land, unsure): (Vec<_>, Vec<_>) = pixels.map(|(idx, px)| {
        let touches = prepped.intersects(&px).unwrap();
        let inside  = prepped.covers(&px).unwrap();

        let unsure  = if touches && !inside { Some(idx) } else { None };
        let touches = if touches { Some(idx) } else { None };

        if idx % nx == 0 {
            println!("row: {}", idx % nx);
        }

        (touches, unsure)
    }).unzip();

    println!("setting up roaring treemap..");
    let land = land.into_iter().filter_map(|x| x).collect::<RoaringTreemap>();
    let unsure = unsure.into_iter().filter_map(|x| x).collect::<RoaringTreemap>();

    println!("unsure: serialized len: {},  size: {} mb", unsure.len(), unsure.serialized_size() / 1024 / 1024);
    println!("land: serialized len: {},  size: {} mb", land.len(), land.serialized_size() / 1024 / 1024);

    Ok(())
}
