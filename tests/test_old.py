import numpy as np
from roaring_landmask import RoaringLandmask
from opendrift_landmask_data import Landmask

def test_landmask_vs_o_l_d():
  l = RoaringLandmask.new()
  ol = Landmask()

  x = np.arange(-180, 180, .5)
  y = np.arange(-90, 90, .5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))

  rland = l.contains_many(xx.ravel(), yy.ravel())
  oland = ol.contains(xx.ravel(), yy.ravel())

  assert (rland == oland).all()

