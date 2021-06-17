# Write landmask in a binary format readable by Rust: this is _NOT_ portable,
# run both things on the same machine.

import numpy as np
import opendrift_landmask_data as old

print('setting up landmask..')
l = old.Landmask()

print('ny =', l.ny)
print('nx =', l.nx)
print('shape = (', l.ny, ',', l.nx, ')')

print('writing to file: mask.bin..')
with open('mask.bin', 'wb') as fd:
  fd.write(l.mask.tobytes())

print('done')

