//! Rasterize the OSM vector coastline (`assets/osm.wkb.xz`) onto the same
//! global 86400x43200 grid used by `RoaringMask`, producing a `mask.bin`
//! raw byte buffer compatible with `make_bitmap.rs`.
//!
//! This exists to fix a correctness bug: the shipped `osm_mask.tbmap.xz`
//! raster is not a superset of `osm.wkb`'s vector polygons (~40% of real
//! land points fall in raster cells marked water), which breaks the
//! `raster AND shapes` fast-path in `RoaringLandmask::contains()` for the
//! Osm provider. Rasterizing `osm.wkb` ourselves, at the same resolution
//! and affine transform, guarantees the raster is a true superset of the
//! data it will be paired with (mirroring how the Gshhg raster already
//! behaves), so the fast AND path can be restored for both providers.

use geo::Geometry;
use geo_rasterize::BinaryBuilder;
use roaring_landmask::shapes::Shapes;
use std::fs;
use std::io::Write;
use std::time::Instant;

const NX: usize = 86400;
const NY: usize = 43200;

/// Same affine transform as `mask::Affine::make()`: lon/lat -> pixel col/row.
///
/// Clamped (not wrapped) into the valid pixel domain: some OSM polygons
/// have vertices at/slightly beyond +-180 longitude (antimeridian-crossing
/// geometry), which maps just outside the raster grid and crashes
/// geo-rasterize's internal indexing. Wrapping longitude per-vertex would
/// corrupt such polygons' topology (teleporting some vertices across the
/// grid while leaving others in place); clamping instead just pins
/// out-of-range vertices to the nearest valid edge pixel, which can only
/// keep the raster a safe superset (never drops real coverage, at worst
/// slightly over-covers at the poles/antimeridian seam).
fn to_pixel(x: f64, y: f64) -> (f64, f64) {
    let px = (x * 240.0 + 43200.5).clamp(0.0, NX as f64);
    let py = (y * 240.0 + 21600.5).clamp(0.0, NY as f64);
    (px, py)
}

fn convert_polygon(poly: &geo::Polygon<f64>) -> geo018::Polygon<f64> {
    let conv_ring = |ring: &geo::LineString<f64>| -> geo018::LineString<f64> {
        geo018::LineString::new(
            ring.coords()
                .map(|c| {
                    let (px, py) = to_pixel(c.x, c.y);
                    geo018::Coordinate { x: px, y: py }
                })
                .collect(),
        )
    };

    let exterior = conv_ring(poly.exterior());
    let interiors: Vec<geo018::LineString<f64>> =
        poly.interiors().iter().map(conv_ring).collect();

    geo018::Polygon::new(exterior, interiors)
}

fn main() {
    let t0 = Instant::now();

    println!("loading osm.wkb.xz...");
    let geom = Shapes::get_geometry_from_compressed("assets/osm.wkb.xz")
        .expect("failed to load osm.wkb.xz");

    let mp = match geom {
        Geometry::MultiPolygon(mp) => mp,
        _ => panic!("expected MultiPolygon geometry"),
    };

    let n_polygons = mp.0.len();
    let n_vertices: usize = mp
        .0
        .iter()
        .map(|p| p.exterior().coords().count() + p.interiors().iter().map(|r| r.coords().count()).sum::<usize>())
        .sum();

    println!(
        "loaded {} polygons, {} vertices in {:.1}s",
        n_polygons,
        n_vertices,
        t0.elapsed().as_secs_f64()
    );

    println!("rasterizing onto {}x{} grid...", NX, NY);
    let mut rasterizer = BinaryBuilder::new()
        .width(NX)
        .height(NY)
        .build()
        .expect("failed to build rasterizer");

    let t1 = Instant::now();

    for (i, poly) in mp.0.iter().enumerate() {
        let converted = convert_polygon(poly);
        rasterizer
            .rasterize(&converted)
            .expect("rasterize failed");

        if (i + 1) % 2000 == 0 || i + 1 == n_polygons {
            let elapsed = t1.elapsed().as_secs_f64();
            let rate = (i + 1) as f64 / elapsed;
            let eta = (n_polygons - (i + 1)) as f64 / rate;
            println!(
                "  {}/{} polygons ({:.1}/s, elapsed {:.0}s, eta {:.0}s)",
                i + 1,
                n_polygons,
                rate,
                elapsed,
                eta
            );
        }
    }

    println!("rasterization done in {:.1}s", t1.elapsed().as_secs_f64());

    let pixels = rasterizer.finish();
    let raw: Vec<u8> = pixels.into_raw_vec().into_iter().map(|b| b as u8).collect();

    assert_eq!(raw.len(), NX * NY, "unexpected raster buffer size");

    let n_land: u64 = raw.iter().map(|&b| b as u64).sum();
    println!(
        "land fraction: {:.4}",
        n_land as f64 / (NX * NY) as f64
    );

    println!("writing mask.bin...");
    let mut fd = fs::File::create("mask.bin").expect("failed to create mask.bin");
    fd.write_all(&raw).expect("failed to write mask.bin");

    println!("total time: {:.1}s", t0.elapsed().as_secs_f64());
}
