# Continuous custom points

Click anywhere on the Moon away from a prepared-site marker to compute that exact point.
There is no geographic snapping, tile outline or zoomed grid.
The label is `Custom point (lat, lon)`, with two decimals and signed longitude.
Only the display label is rounded. Terrain and sky calculations retain the picked coordinates.

`globe/src/pick.rs` unprojects the click onto the lunar surface.
The app fetches `lunar_core::anypoint::terrain_requests` for those coordinates.
`HorizonJob` computes 720 horizon bearings incrementally, using the existing terrain sources.
The observer is 2 m above the point's terrain surface.
The shared hourly ephemeris supplies Sun and Earth tracks from 2027 through 2030.
The existing panorama, timeline and statistics display the result.
The seven prepared sites and the lit Moon while custom terrain loads are unchanged.

Computed points remain as small colored dots on the globe.
Red through yellow to green represents increasing Sun visibility in the selected period.
The bottom-left legend explains the color scale. The current point retains its normal marker.
Click within about 10 screen pixels of a visible dot to reopen its exact cached point instantly.
Otherwise a click starts a new computation at the clicked position.
Failed current points can be clicked again to retry; terrain downloads still time out after 30 s.
The cache lasts for the current app session, not across reloads.

Terrain still depends on NASA Moon Trek availability, coverage and browser CORS access.
Nominal DEM resolution does not guarantee agreement with the prepared reference terrain.
The core tile-grid API remains available for other callers, but the app no longer uses it.
