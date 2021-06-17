use std::io::prelude::*;
use std::fs::File;
use roaring::*;

fn main() -> std::io::Result<()> {
    println!("opening mask.bin..");
    let mut fd = File::open("mask.bin")?;

    println!("reading into vec..");
    let mut mask: Vec<u8> = Vec::new();
    fd.read_to_end(&mut mask)?;

    let ny = 43200;
    let nx = 86400;

    assert!(mask.len() == ny * nx);

    println!("setting up roaring treemap..");
    let mut tmap = RoaringTreemap::new();

    println!("filling up treemap..");
    for (y, row) in mask.chunks_exact(nx).enumerate() {
        for (x, onland) in row.into_iter().enumerate() {
            if *onland == 1 {
                let idx = y * nx + x;
                let idx = idx as u64;
                tmap.insert(idx);
            }
        }

        if y % 1000 == 0 {
            println!("on row: {}", y);
        }
    }

    println!("serialized size: {} mb", tmap.serialized_size() / 1024 / 1024);

    println!("serializing bitmap to file: mask.tbmap..");
    {
        let ofd = File::create("mask.tbmap")?;
        tmap.serialize_into(ofd)?;
    }

    Ok(())
}
