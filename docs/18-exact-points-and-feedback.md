# Exact points, reusable dots, and quiet feedback

## 1. Which version are we learning?

This chapter continues the archived tutorials through main commit
`5e4d5b7d022a580db32a61228f28452ca3d8eedb`. The code on `learn` deliberately
remains the older teaching snapshot: this is a documentation update, not a merge
of application code. Source links in chapters 18–23 therefore name that main
commit explicitly. Earlier annotated source remains useful historical material.

The relevant changes are `63102a9`, `a35fa24`, and `448460d`. They replace
tile-center selection with continuous selection, retain computed results as
clickable dots, and make the decorative grid disappear after zooming stops.
The distinction matters scientifically: a small move can put an observer on the
other side of a crater rim. A display grid must not silently move that observer.

Read [the picking implementation](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/pick.rs),
[application integration](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/app/src/lib.rs),
and [custom jobs](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/app/src/custom.rs)
alongside this chapter. The numerical horizon still belongs to `lunar_core`,
not to the fragment shader.

## 2. Undo the camera before interpreting a click

The renderer uses east-positive longitude, north-positive latitude, and the
Moon-fixed direction

$$
\mathbf p=(\cos\phi\sin\lambda,\ \sin\phi,\ \cos\phi\cos\lambda).
$$

Thus north is $+y$, longitude zero is $+z$, and $90^\circ$ east is $+x$.
To recover coordinates from a unit direction, use

$$
\phi=\arcsin(p_y),\qquad
\lambda=\mathrm{atan2}(p_x,p_z).
$$

The argument order is intentional. Swapping the arguments of `atan2` would
rotate/reflect the longitude convention rather than merely change notation.
Clamping $p_y$ to $[-1,1]$ protects `asin` from roundoff at a pole.

A pointer position is initially in UI coordinates: right is positive x and down
is positive y. `surface_pick` converts it to normalized device coordinates:

```rust
let ndc = [
    (p.x - rect.center().x) / (rect.width() * 0.5),
    -(p.y - rect.center().y) / (rect.height() * 0.5),
];
```

The minus sign flips UI y into graphics y. `View::unproject` constructs the
perspective ray, intersects the analytic sphere, and undoes the complete model
rotation, including the dated Moon rotation. Merely applying the inverse mouse
drag would be wrong in Earth-following and inertial camera modes.

The returned vector is Moon-fixed, so converting it to latitude/longitude does
not depend on which side of the display currently faces the camera. A missed
sphere produces `None`, not coordinates on an infinite background plane.

## 3. Selection priority is part of the interaction contract

The event is accepted only for a primary single click. Double-click recentering
must not accidentally launch a terrain download. Existing computed dots are
tested before a new surface intersection; prepared-site marker handling also
prevents an ordinary marker click from becoming a new custom point.

The dot test projects every stored coordinate using the same `View`. It chooses
the nearest visible candidate within ten UI points of the pointer, using squared
distance. This is a screen-sized hit target, not a ten-meter circle on the Moon.
It remains usable at different zooms and device pixel ratios.

The essential new-point conversion is:

```rust
if let Some(point) = view.unproject(ndc) {
    let lat = f64::from(point[1].clamp(-1.0, 1.0).asin().to_degrees());
    let lon = f64::from(point[0].atan2(point[2]).to_degrees())
        .rem_euclid(360.0);
    self.pick = Some((lat, lon));
}
```

These are continuous coordinates within the renderer's floating-point picking
precision, not arbitrary-precision survey coordinates. Storing them as `f64`
does not recover precision already lost in the `f32` intersection.

`take_pick` transfers the pending coordinate to the application and clears it.
This is an event, not a value that should re-trigger downloads every frame.
`Custom::start` passes the chosen latitude and longitude directly to
`terrain_requests` and later `HorizonJob::new`. `tile_of` still exists in core,
but it no longer chooses the observer location in this UI path.

## 4. The scientific job does not sample the screenshot

The globe's sphere is not a displaced terrain mesh. Its bright bumps are
imagery or normal-map shading. The numerical observer elevation instead comes
from interpolating the downloaded DEM at the exact selected coordinate and
adding a two-meter mast height.

Core computes 720 azimuth bins. The application advances the job in four-bin
chunks under a twelve-millisecond frame budget, then derives hourly topocentric
Sun/Earth tracks for 2027-01-01 through 2030-01-01 inclusive. A work budget is
not a hard frame-time guarantee: one chunk can overrun before control returns.

The raw terrain inputs are ordered F32 TIFF responses. Fine terrain is preferred
over coarse terrain; longitude-seam splits can produce more than two requests.
Scientific rays stop at 250 km or a coverage gap. Do not confuse that algorithm
with the roughly 600 km visual shadow marcher in [chapter 20](20-terrain-shadows.md).

The request dimensions are also intentionally different from rendering detail:
global/coarse scientific exports are 512²; polar fine scientific windows are
1024². The rendering detail windows in chapter 23 are always 512² DEM exports.

## 5. Reusing a point means reusing its data, not its label

`App::open_custom` caches a completed old panorama when another coordinate is
selected. An exactly matching cached coordinate restores that panorama instead
of starting the same work again. `Custom::can_reuse` rejects failed jobs, so
clicking a failed point can retry.

The equality is coordinate equality, not equality of the displayed two-decimal
name. Two different positions can round to the same label. Replacing coordinate
identity with the label would be a subtle cache collision and a scientific bug.

`set_points` receives coordinates and each panorama's percentage of time with
any Sun in the chosen period. The marker colors interpolate red → yellow → green:

$$
p=\mathrm{clamp}(q/100,0,1),\quad
r=\min(2(1-p),1),\quad g=\min(2p,1).
$$

The implementation scales these channels to 230 and 200 and keeps blue at 60.
The legend describes a period statistic, not instantaneous brightness, power,
or the fraction of the solar disk visible at the selected instant. Statistics
remain duration-weighted through the existing panorama/core path.

## 6. Signed degrees are presentation, not a new reference frame

Internally a longitude may be in $[0,360)$. A common display formatter uses

$$
\lambda_{\mathrm{display}}
=((\lambda+180)\bmod 360)-180.
$$

Rust's `rem_euclid` implements the nonnegative remainder required here. The
actual formatter is small enough to inspect in full:

```rust
fn coordinates(lat: f64, lon: f64) -> String {
    let lon = (lon + 180.0).rem_euclid(360.0) - 180.0;
    format!("{lat:.2}°, {lon:.2}°")
}
```

For example, east longitude $271.99^\circ$ displays as $-88.01^\circ$.
This does not move the point or change Trek's request convention. Notice also
that $180^\circ$ displays as $-180^\circ$, which is the same meridian.

## 7. A decorative grid must not pretend to be a computation grid

`GridFade` remembers the last zoom and the time it changed. At zoom below 3×,
the grid is hidden. At or above 3× it holds alpha 18 for one second, fades during
the next half second, and then stops requesting animation frames.

For normalized fade time $t$ the opacity multiplier is

$$
f(t)=1-t^2(3-2t),\qquad 0\le t\le1.
$$

The derivative is zero at both ends, avoiding a sharp velocity change in the
fade. During the hold the next repaint is scheduled at the hold boundary;
during the fade the code requests about 60 Hz. Invalid time/zoom values return
an invisible, non-repainting result.

The old two-degree-band construction still supplies decorative lines, but
there is no selected-tile outline and no snapping. Rotation alone does not
restart the timer: the feedback is specifically tied to zoom changes.

## 8. Timeout means lack of progress

The old thirty-second download deadline was misleading for slow but progressing
terrain requests. `Stage::Fetching` now records `received` and `last_response`.
When another response completes, the inactivity clock resets. The job fails
after ninety seconds without another terrain response while terrain is missing.

Individual ehttp requests use `timeout = None`; the job owns this policy. An
HTTP/error response is still reported immediately. Once all terrain is ready,
waiting for shared ephemeris data is a different state of progress and is not
mistaken for a terrain timeout. This is not a ninety-second maximum total job
duration and not a ninety-second byte-by-byte streaming-progress timer.

## 9. Read tests as executable interaction specifications

In the current main checkout, run `cargo test --workspace` to cover the pure
coordinate, grid, dot-picking and inactivity-clock tests. In particular inspect
`dot_picking_is_exact_nearest_and_screen_sized`,
`grid_holds_fades_and_stops_requesting_frames`,
`signed_degree_coordinates_wrap_longitude`, and
`terrain_timeout_resets_on_each_response`.

For a browser exercise, pick two nearby points that round to the same printed
coordinates, then reopen each dot. Confirm their cached identities are distinct.
Also click empty space and double-click the sphere: neither should start a new
single-click terrain job. A screenshot alone cannot establish that no duplicate
network requests occurred; inspect the requests as well.
