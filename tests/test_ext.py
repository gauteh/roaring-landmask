import roaring_landmask

def test_ext_is_working():
    assert roaring_landmask.im_alive() == True
