import geopandas as gpd
import shapely
from shapely.geometry import MultiPolygon
from shapely.ops import unary_union
import numpy as np
import time
import regionmask
import xarray as xr


def wkb_to_wkt(wkb_file_path, wkt_file_path):
    # Read the WKB file
    with open(wkb_file_path, "rb") as wkb_file:
        wkb_data = wkb_file.read()

    # Parse WKB and create a shapely geometry object
    geometry = shapely.wkb.loads(wkb_data)

    # Convert geometry to WKT
    wkt_data = shapely.wkt.dumps(geometry)

    # Write WKT to file
    with open(wkt_file_path, "w") as wkt_file:
        wkt_file.write(wkt_data)

    print(f"Conversion complete. WKT file saved as {wkt_file_path}")


# wkb_to_wkt('output.wkb', 'output.wkt')


def shapefile_to_wkb(shp_path, wkb_path, tolerance=0.0005, chunk_size=1000):
    print("Reading shapefile...")
    start_time = time.time()
    gdf = gpd.read_file(shp_path)
    print(f"Shapefile read in {time.time() - start_time:.2f} seconds.")
    print(f"Number of geometries: {len(gdf)}")

    simplified_geometries = []
    total_geometries = len(gdf)

    for i in range(0, total_geometries, chunk_size):
        chunk = gdf.iloc[i : i + chunk_size]
        print(
            f"Processing chunk {i//chunk_size + 1} of {(total_geometries-1)//chunk_size + 1}..."
        )

        # Simplify each geometry individually
        simplified_chunk = chunk.geometry.simplify(tolerance, preserve_topology=True)
        simplified_geometries.extend(list(simplified_chunk))

        print(
            f"Processed {min(i+chunk_size, total_geometries)} out of {total_geometries} geometries."
        )

    print("Combining all simplified geometries...")
    # Use unary_union to combine all geometries efficiently
    combined_geometry = unary_union(simplified_geometries)

    # Ensure the result is a MultiPolygon
    if combined_geometry.geom_type == "Polygon":
        final_multi_polygon = MultiPolygon([combined_geometry])
    elif combined_geometry.geom_type == "MultiPolygon":
        final_multi_polygon = combined_geometry
    else:
        raise ValueError(f"Unexpected geometry type: {combined_geometry.geom_type}")

    print("Getting WKB representation...")
    wkb = final_multi_polygon.wkb

    print("Writing WKB to file...")
    with open(wkb_path, "wb") as wkb_file:
        wkb_file.write(wkb)

    print(f"Process completed. Output written to: {wkb_path}")


# shapefile_to_wkb('land-polygons-complete-4326/land_polygons.shp', 'output.wkb')


def wkb_to_mask(wkb_file_path, chunk_size=1000):
    # Read the WKB file as binary
    with open(wkb_file_path, "rb") as file:
        wkb_data = file.read()

    # Convert WKB to Shapely geometry
    geometry = shapely.wkb.loads(wkb_data)

    # Create a GeoDataFrame
    gdf = gpd.GeoDataFrame({"geometry": [geometry]})

    lon = np.linspace(-180, 180, 86400, endpoint=False)
    lat = np.linspace(-90, 90, 43200, endpoint=False)

    total_rows = len(lat)

    with open("mask.bin", "wb") as f:
        for start_row in range(0, total_rows, chunk_size):
            end_row = min(start_row + chunk_size, total_rows)
            lat_chunk = lat[start_row:end_row]

            mask_chunk = regionmask.mask_geopandas(gdf, lon, lat_chunk)
            mask_chunk = xr.where(mask_chunk == 0.0, 1.0, mask_chunk)
            mask_chunk = mask_chunk.fillna(0.0)
            mask_chunk = mask_chunk.astype(np.uint8)

            mask_chunk.values.tofile(f)
            print(
                f"{end_row / total_rows * 100.0}% - ({start_row} -> {end_row}) / {total_rows}"
            )


# wkb_to_mask("output.wkb")
