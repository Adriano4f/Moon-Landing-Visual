# Dated Moon rotation

The default globe camera is fixed in Moon-centered J2000 equatorial space. Advancing UTC turns the Moon at its physical sidereal rate instead of moving only its light. Dragging changes that inertial camera; the optional one-minute auto-orbit remains an explicitly artificial inspection aid.

`globe/src/orientation.rs` implements all E1–E13 IAU lunar pole/prime-meridian terms from the WGCCRE model, as published in NASA NAIF's [pck00011.tpc](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/pck00011.tpc), BODY301 and BODY3_NUT_PREC_ANGLES. The mean W rate is 13.17635815 degrees/day, or 27.32166 days per turn. UTC approximates TDB for this display. This is not a high-precision DE-series lunar attitude solution or an independently validated inertial ephemeris; the prepared mean-Earth directions remain authoritative for site calculations.

Renderer axes are `(Y, Z, X)` relative to conventional J2000 axes, placing inertial north upward. One model matrix composes the camera and dated lunar rotation. Mesh positions, mapped normals, polar imagery, Sun/Earth directions, rays, markers, tile outlines, grid and computed-tile colors use that same transform. The shader retains Moon-fixed texture coordinates. Picking uses the matrix transpose; Earth-follow and go-to-site transform their target directions first.

No scientific horizon, local azimuth/elevation, visibility predicate or period statistic is rotated or recomputed by this display feature. Shading uses the same relative normal/light geometry. Earth-follow remains available but is no longer the default.
