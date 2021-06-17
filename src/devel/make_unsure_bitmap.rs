use std::io::prelude::*;
use std::fs::File;
use roaring::*;
use roaring_landmask::RoaringMask;

fn main() -> std::io::Result<()> {
    println!("opening ocean_0.25_nm.mm");
    let mut fd = File::open("ocean_0.25_nm.mm")?;

    println!("reading into vec..");
    let mut mask: Vec<u8> = Vec::new();
    fd.read_to_end(&mut mask)?;

    let ny = 43200;
    let nx = 86400;

    assert!(mask.len() == ny * nx);

    println!("opening landmask map..");
    let land = RoaringMask::new()?;

    println!("setting up roaring treemap..");
    let mut tmap = RoaringTreemap::new();

    println!("filling up treemap..");
    for (y, row) in mask.chunks_exact(nx).enumerate() {
        for (x, inocean) in row.into_iter().enumerate() {
            if *inocean == 1 {
                let idx = y * nx + x;
                let idx = idx as u64;

                if land.tmap.contains(idx) {
                    tmap.insert(idx);
                }
            }
        }

        if y % 1000 == 0 {
            println!("on row: {}", y);
        }
    }

    println!("serialized length: {}, size: {} mb", tmap.len(), tmap.serialized_size() / 1024 / 1024);
    println!("serializing bitmap to file: unsure_mask.tbmap..");
    {
        let ofd = File::create("unsure_mask.tbmap")?;
        tmap.serialize_into(ofd)?;
    }

    Ok(())
}

