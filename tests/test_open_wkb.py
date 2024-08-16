from roaring_landmask import Gshhg
import shapely.wkb as wkb
import pytest

@pytest.mark.skip
def test_read_wkb():
    w = Gshhg.wkb()

@pytest.mark.skip
def test_load_wkb():
    w = Gshhg.wkb()
    polys = wkb.loads(w)



