use rayon::prelude::*;
use roaring_landmask::Gshhg;
use std::fs::File;
use std::io::{BufWriter, prelude::*};

fn main() -> std::io::Result<()> {
    let ny = 43200;
    let nx = 86400;
    // let ny = 100;
    // let nx = 100;

    let (y0, y1) = (-90.0f64, 90.0f64);
    let (x0, x1) = (-180.0f64, 180.0f64);

    let dy = (y1 - y0) / ny as f64;
    let dx = (x1 - x0) / nx as f64;

    println!("ny, nx = {}, {}", ny, nx);

    println!("opening pixelmap..");
    let pixels = std::fs::read("pixels.map")?;
    println!("pixels: {}", pixels.len());

    assert!(pixels.len() == (nx+1) * (ny+1));


    Ok(())
}

