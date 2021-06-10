import numpy as np
from roaring_landmask import RoaringLandmask

def test_make_landmask():
    m = RoaringLandmask.new()

def test_landmask_onland(benchmark):
    l = RoaringLandmask.new()

    onland = (np.array([15.]), np.array([65.6]))
    c = benchmark(l.contains, onland[0], onland[1])
    assert c

def test_landmask_many(benchmark):
  l = RoaringLandmask.new()

  x = np.arange(-180, 180, .5)
  y = np.arange(-90, 90, .5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))
  benchmark(l.contains_many, xx.ravel(), yy.ravel())
