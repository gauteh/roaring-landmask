import matplotlib.pyplot as plt
import matplotlib.colors
import numpy as np
import cartopy.crs as ccrs
from roaring_landmask import RoaringLandmask
from roaring_landmask import LandmaskProvider


x = np.arange(-180, 180, .1)
y = np.arange(-90, 90, .1)

xx, yy = np.meshgrid(x,y)

gshhg_landmask = RoaringLandmask.new_with_provider(LandmaskProvider.Gshhg)
land = gshhg_landmask.contains_many(xx.ravel(), yy.ravel())
land = land.reshape(xx.shape)

cmap = matplotlib.colors.ListedColormap(['b', 'g'])

fig = plt.figure(dpi = 600, figsize = (20, 10))
ax = fig.add_subplot(1, 1, 1, projection=ccrs.PlateCarree())


ax.pcolormesh(xx, yy, land, cmap = cmap, alpha = .7, transform = ccrs.PlateCarree())
ax.gridlines(draw_labels = True)
ax.set_title('The Earth GSHHG', pad=20)
fig.savefig('the_earth_gshhg.png')

osm_landmask = RoaringLandmask.new_with_provider(LandmaskProvider.Osm)
land = osm_landmask.contains_many(xx.ravel(), yy.ravel())
land = land.reshape(xx.shape)

fig = plt.figure(dpi = 600, figsize = (20, 10))
ax = fig.add_subplot(1, 1, 1, projection=ccrs.PlateCarree())
ax.pcolormesh(xx, yy, land, cmap = cmap, alpha = .7, transform = ccrs.PlateCarree())
ax.gridlines(draw_labels = True)
ax.set_title('The Earth OSM', pad=20)
fig.savefig('the_earth_osm.png')
