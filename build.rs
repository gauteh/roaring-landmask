use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use path_slash::PathExt;

pub static GSHHS_F: &str = "gshhs_f_-180.000000E-90.000000N180.000000E90.000000N.wkb.xz";
pub static GSHHS_F_CS: &str = "05bdf3089407b9829a7a5be7ee43f1e4205f2bbc641e4778af77e4814be216da";

pub static MASK: &str = "mask.tbmap.xz";
pub static MASK_CS: &str = "5ea0e772ffc6ca8ad10c5de02be50670cbaedcff20b3541df6b78d3e1fdf48a1";

fn main() {
    println!("hello");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    println!("outdir: {:?}", out_dir);

    let assets_dir = Path::new(&out_dir).join("gshhs");

    // write assets script
    let assets = Path::new(&out_dir).join("gshhs.rs");
    {
        let mut fd = fs::File::create(assets).unwrap();
        write!(
            fd,
            "
use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = \"{}\"]
pub struct GsshgData;

        ",
            assets_dir.to_slash().unwrap()
        )
        .unwrap();
    }

    let gshhs = Path::new(&out_dir).join("gshhs");
    if !gshhs.exists() {
        fs::create_dir(gshhs).unwrap();
    }

    // copy or download files
    if env::var("DOCS_RS").is_err() {
        copy_or_download(GSHHS_F, GSHHS_F_CS);
        copy_or_download(MASK, MASK_CS);
    } else {
        println!("not downloading anything when on docs.rs.");
    }
}

fn copy_or_download(from: impl AsRef<Path>, csum: &str) {
    let from = from.as_ref();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let full_from = Path::new("gshhs").join(&from);
    let full_to = Path::new(&out_dir).join("gshhs").join(&from);

    if !full_to.exists() {
        if full_from.exists() {
            println!("copying {:?}..", &from);
            fs::copy(&full_from, &full_to).unwrap();
        } else {
            let url = format!(
                "https://github.com/gauteh/roaring-landmask/raw/main/gshhs/{}",
                from.to_str().unwrap()
            );
            println!("downloading {:?} from {:?}..", &from, &url);

            let resp = reqwest::blocking::get(&url).unwrap();
            let mut fd = fs::File::create(&full_to).unwrap();
            fd.write_all(&resp.bytes().unwrap()).unwrap();
        }
    } else {
        println!("{:?} already exists..", &from);
    }

    // Check check-sum
    use ring::{digest, test};
    let expected: Vec<u8> = test::from_hex(csum).unwrap();
    let actual = digest::digest(&digest::SHA256, &fs::read(&full_to).unwrap());
    if &expected != &actual.as_ref() {
        // Delete erronous file
        fs::remove_file(&full_to).unwrap();
        panic!("Checksum mismatched for {:?}, downloaded file deleted..", &from);
    }
}
