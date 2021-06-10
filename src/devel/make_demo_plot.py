import matplotlib.pyplot as plt
import matplotlib.colors
import numpy as np
import cartopy.crs as ccrs
from roaring_landmask import RoaringLandmask


fig = plt.figure(dpi = 150, figsize = (20, 10))
ax = fig.add_subplot(1, 1, 1, projection=ccrs.PlateCarree())

ax.coastlines()
ax.gridlines(draw_labels = True)

x = np.arange(-180, 180, .5)
y = np.arange(-90, 90, .5)

xx, yy = np.meshgrid(x,y)

l = RoaringLandmask.new()
land = l.contains_many(xx.ravel(), yy.ravel())
land = land.reshape(xx.shape)

cmap = matplotlib.colors.ListedColormap(['b', 'g'])
plt.pcolormesh(xx, yy, land, cmap = cmap, alpha = .7, transform = ccrs.PlateCarree())

plt.title('The Earth')

plt.savefig('the_earth.png')
# plt.show()
