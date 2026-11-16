# Continuous custom points

Click anywhere on the Moon away from a prepared-site marker to compute that exact point.
There is no geographic snapping or selected tile outline.
A faint grid appears only when zoomed in and is purely decorative.
The label uses signed decimal degrees, for example `Custom point (-79.24°, -88.01°)`.
Coordinates have two decimals; negative latitude is south and negative longitude is west.
Displayed longitude is normalized to -180° through 180°.
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
Failed current points can be clicked again to retry.
Terrain downloads time out after 90 s without another response, not while waiting for shared sky data.
The download status shows the response count and total elapsed seconds.
The cache lasts for the current app session, not across reloads.

Terrain still depends on NASA Moon Trek availability, coverage and browser CORS access.
Nominal DEM resolution does not guarantee agreement with the prepared reference terrain.
The core tile-grid API remains available for other callers, but the app no longer uses it.
