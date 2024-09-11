import pytest
import numpy as np
from roaring_landmask import RoaringLandmask
from roaring_landmask import LandmaskProvider

@pytest.mark.parametrize("provider", [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_make_landmask(provider):
    m = RoaringLandmask.new_with_provider(provider)

@pytest.mark.parametrize("provider", [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_landmask_onland(benchmark, provider):
    l = RoaringLandmask.new_with_provider(provider)

    onland = (np.array([15.]), np.array([65.6]))
    c = benchmark(l.contains, onland[0], onland[1])
    assert c

@pytest.mark.parametrize("provider", [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_landmask_onland_single(provider):
    l = RoaringLandmask.new_with_provider(provider)

    c = l.contains(15., 65.6)
    assert c

@pytest.mark.parametrize("provider", [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_landmask_many(benchmark, provider):
  l = RoaringLandmask.new_with_provider(provider)

  x = np.arange(-180, 180, .5)
  y = np.arange(-90, 90, .5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))
  benchmark(l.contains_many, xx.ravel(), yy.ravel())

@pytest.mark.parametrize("provider", [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_landmask_many_few(benchmark, provider):
  l = RoaringLandmask.new_with_provider(provider)

  x = np.linspace(-180, 180, 10)
  y = np.linspace(-90, 90, 5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))
  benchmark(l.contains_many, xx.ravel(), yy.ravel())

@pytest.mark.skip
def test_landmask_many_par(benchmark):
  l = RoaringLandmask.new()

  x = np.arange(-180, 180, .5)
  y = np.arange(-90, 90, .5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))
  benchmark(l.contains_many_par, xx.ravel(), yy.ravel())

@pytest.mark.skip
def test_landmask_many_par_few(benchmark):
  l = RoaringLandmask.new()

  x = np.linspace(-180, 180, 10)
  y = np.linspace(-90, 90, 5)

  xx, yy = np.meshgrid(x,y)

  print ("points:", len(xx.ravel()))
  benchmark(l.contains_many_par, xx.ravel(), yy.ravel())

