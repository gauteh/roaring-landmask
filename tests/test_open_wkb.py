import pytest
from roaring_landmask import Shapes, LandmaskProvider
import shapely.wkb as wkb


@pytest.mark.parametrize("provider",
                         [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_read_wkb(provider):
    w = Shapes.wkb(provider)


@pytest.mark.parametrize("provider",
                         [LandmaskProvider.Gshhg, LandmaskProvider.Osm])
def test_load_wkb(provider):
    w = Shapes.wkb(provider)
    polys = wkb.loads(w)
