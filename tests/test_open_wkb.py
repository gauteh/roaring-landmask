from roaring_landmask import Gshhg
import shapely.wkb as wkb

def test_read_wkb():
    w = Gshhg.wkb()

def test_load_wkb():
    w = Gshhg.wkb()
    polys = wkb.loads(w)



