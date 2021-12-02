import numpy as np
from roaring_landmask import RoaringLandmask

def test_dateline():
    mask = RoaringLandmask.new()

    x = np.linspace(-180, 180, 100)
    y = np.linspace(-90, 90, 100)

    xx, yy = np.meshgrid(x, y)
    xx, yy = xx.ravel(), yy.ravel()
    mm = mask.contains_many(xx, yy)

    # Offset
    x2 = np.linspace(180, 540, 100)
    y2 = np.linspace(-90, 90, 100)


    xx, yy = np.meshgrid(x2, y2)
    xx, yy = xx.ravel(), yy.ravel()
    MM = mask.contains_many(xx, yy)

    np.testing.assert_array_equal(mm, MM)

