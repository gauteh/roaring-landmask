import numpy as np
from roaring_landmask import MaskShape

def test_make_shape():
    m = MaskShape.new()

def test_landmask_onland(benchmark):
    l = MaskShape.new()

    onland = (np.array([15.]), np.array([65.6]))
    c = benchmark(l.contains, onland[0], onland[1])
    assert c

def test_landmask_many(benchmark):
  l = MaskShape.new()

  x = np.arange(-180, 180, .5)
  y = np.arange(-90, 90, .5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))
  benchmark(l.contains_many, xx.ravel(), yy.ravel())
