use geo::{Geometry, MultiPolygon, indexed::IntervalTreeMultiPolygon};
use geozero::ToGeo;
use std::io::Read;
use std::time::Instant;

fn time_load(path: &str) {
    println!("=== {path} ===");

    let t = Instant::now();
    let fd = std::fs::File::open(path).unwrap();
    let fd = std::io::BufReader::new(fd);
    let mut fd = xz2::bufread::XzDecoder::new(fd);
    let mut buf = Vec::new();
    fd.read_to_end(&mut buf).unwrap();
    println!("  decompress:                    {:>8.1?}  ({} bytes)", t.elapsed(), buf.len());

    let t = Instant::now();
    let geom = geozero::wkb::Wkb(buf).to_geo().unwrap();
    println!("  geozero WKB parse:             {:>8.1?}", t.elapsed());

    let mp = match geom {
        Geometry::MultiPolygon(mp) => mp,
        _ => panic!("expected MultiPolygon"),
    };

    let t = Instant::now();
    let _tree = IntervalTreeMultiPolygon::<f64>::new(&mp);
    println!("  IntervalTreeMultiPolygon::new: {:>8.1?}", t.elapsed());
}

#[test]
fn time_load_steps() {
    time_load("assets/gshhg.wkb.xz");
    time_load("assets/osm.wkb.xz");
}
