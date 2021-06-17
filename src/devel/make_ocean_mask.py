import numpy as np
import rasterio
from rasterio.features import rasterize, geometry_mask
from rasterio import Affine
import shapely.wkb as wkb

from opendrift_landmask_data.gshhs import get_gshhs_f
from opendrift_landmask_data.mask import Landmask


def mask_rasterize(inwkb, outnp, ocean):
    dnm = Landmask.dnm
    nx = Landmask.nx
    ny = Landmask.ny
    x = [-180, 180]
    y = [-90, 90]

    print('nx =', nx, 'ny =', ny)

    resx = float(x[1] - x[0]) / nx
    resy = float(y[1] - y[0]) / ny
    transform = Landmask.get_transform()
    print("transform = ", transform)

    land = wkb.load(inwkb)

    img = geometry_mask(land,
        invert=(not ocean),
        out_shape=(ny, nx),
        all_touched=True,
        transform=transform).astype('uint8')

    print("img shape:", img.shape)

    print('writing:', outnp)
    with open(outnp, 'wb') as fd:
      fd.write(img.tobytes())

    return img


if __name__ == '__main__':
    print("resolution, m =", Landmask.dm)
    # mask_rasterize(get_gshhs_f(), 'mask_%.2f_nm.mm' % Landmask.dnm, False)
    mask_rasterize(get_gshhs_f(), 'ocean_%.2f_nm.mm' % Landmask.dnm, True)
