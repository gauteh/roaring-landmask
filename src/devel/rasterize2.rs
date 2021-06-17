use geos::{CoordSeq, Geom, Geometry};
use rayon::prelude::*;
use roaring::*;
use roaring_landmask::Gshhg;
use std::fs::File;
use std::io::{BufWriter, prelude::*};

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
    const THDS: usize = 8;
    rayon::ThreadPoolBuilder::new().num_threads(THDS).build_global().unwrap();

    println!("opening shapes.. x {}", THDS);
    let shp: Vec<Gshhg> = (0..THDS).map(|_| Gshhg::new().unwrap()).collect();
    // let shp = Gshhg::new()?;

    let ny = 43200;
    let nx = 86400;
    // let ny = 100;
    // let nx = 100;

    let (y0, y1) = (-90.0f64, 90.0f64);
    let (x0, x1) = (-180.0f64, 180.0f64);

    let dy = (y1 - y0) / ny as f64;
    let dx = (x1 - x0) / nx as f64;

    println!("ny, nx = {}, {}", ny, nx);

    // 1. check y0, x0 of all boxes
    println!("checking all points..");
    let pixels = (0..ny + 1)
        .into_par_iter()
        .map(|iy| {
            let thd = rayon::current_thread_index().unwrap();

            let shp = &shp[thd];

            (0..nx + 1).map(move |ix| {
                let y = y0 + dy * iy as f64;
                let x = x0 + dx * ix as f64;
                assert!(y <= y1);
                assert!(x <= x1);

                if ix == 0 && iy % 100 == 0 {
                    println!("[{}] row: {}", thd, iy);
                }

                shp.contains(x, y)
            })
        });

    let pixels = pixels.flatten_iter().collect::<Vec<bool>>();
    let pixels: &[u8] = unsafe { &*(pixels.as_slice() as *const [bool] as *const [u8]) };
    println!("pixels: {}", pixels.len());

    println!("writing pixels to disk..");
    let mut fd = BufWriter::new(File::create("pixels.map")?);
    fd.write_all(pixels)?;


    Ok(())
}
