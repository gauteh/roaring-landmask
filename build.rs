use path_slash::PathExt;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

pub static GSHHG: &str = "gshhg.wkb.xz";
pub static GSHHG_SHA256: &str = "05bdf3089407b9829a7a5be7ee43f1e4205f2bbc641e4778af77e4814be216da";

pub static GSHHG_MASK: &str = "gshhg_mask.tbmap.xz";
pub static GSHHG_MASK_SHA256: &str =
    "5ea0e772ffc6ca8ad10c5de02be50670cbaedcff20b3541df6b78d3e1fdf48a1";

pub static OSM: &str = "osm.wkb.xz";
pub static OSM_SHA256: &str = "7cbbbb56dc8f6a339d837e57aac4c50c9f54e7ac1118803274725cf61226b727";

pub static OSM_MASK: &str = "osm_mask.tbmap.xz";
pub static OSM_MASK_SHA256: &str =
    "e60dd30737ad8480619d727bb246a1107d30a66563b73628337dc3f92255b684";

fn main() {
    println!("hello");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    println!("outdir: {:?}", out_dir);

    let assets_dir = Path::new(&out_dir).join("assets");

    // write assets script
    let mut fd = fs::File::create(Path::new(&out_dir).join("source_data.rs")).unwrap();
    write!(
        fd,
        "
use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = \"{}\"]
pub struct GsshgData;

#[derive(RustEmbed)]
#[folder = \"{}\"]
pub struct OsmData;
    ",
        assets_dir.to_slash().unwrap(),
        assets_dir.to_slash().unwrap()
    )
    .unwrap();

    if !assets_dir.exists() {
        fs::create_dir(assets_dir).unwrap();
    }

    // copy or download files
    if env::var("DOCS_RS").is_err() {
        copy_or_download(GSHHG, GSHHG_SHA256);
        copy_or_download(GSHHG_MASK, GSHHG_MASK_SHA256);
        copy_or_download(OSM, OSM_SHA256);
        copy_or_download(OSM_MASK, OSM_MASK_SHA256);
    } else {
        println!("not downloading anything when on docs.rs.");
    }
}

fn copy_or_download(from: impl AsRef<Path>, csum: &str) {
    let from = from.as_ref();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let full_from = Path::new("assets").join(&from);
    let full_to = Path::new(&out_dir).join("assets").join(&from);

    if !full_to.exists() {
        if full_from.exists() {
            println!("copying {:?}..", &from);
            fs::copy(&full_from, &full_to).unwrap();
        } else {
            let url = format!(
                "https://github.com/gauteh/roaring-landmask/raw/main/assets/{}",
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
        panic!(
            "Checksum mismatched for {:?}, downloaded file deleted..",
            &from
        );
    }
}
