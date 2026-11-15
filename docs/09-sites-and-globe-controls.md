# Published sites and interactive globe controls

## 15. Fixed, published coordinates

The catalog contains seven locations, not seven arbitrarily chosen pixels. All four south-polar rows from Glaser et al. (2017), Table 2 are retained, including both distinct Shackleton rows. S1 and S2 are application identifiers for those rows, not new published names. Three additional coordinates are reproduced in that peer-reviewed paper's Table 1, attributed to Mazarico et al. (2011) and Speyerer and Robinson (2013). Their exact printed precision is retained. Those three coordinates and values are verified against this published secondary source; no claim of primary-paper table recovery is made.

The source per entry is annotated in sites.json and available on the site selector's tooltip. [Glaser et al. (2017)](https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf) is the accessible verification source. The original Speyerer and Robinson paper is [Icarus 222, 122 to 136](https://doi.org/10.1016/j.icarus.2012.10.010). A published comparison is not a reproduction when terrain resolution, observer height, dates or illumination definitions differ. In particular, an image-derived illuminated fraction need not equal a circular solar-area fraction.

| Catalog name | Latitude, degrees | East longitude, degrees | Coordinate verification |
|:--|--:|--:|:--|
| Connecting Ridge C1 | -89.4398 | 222.83 | Table 2 |
| Connecting Ridge C2 | -89.3130 | 245.98 | Table 2 |
| Shackleton S1 | -89.6866 | 197.19 | Table 2, first Shackleton row |
| Shackleton S2 | -89.7815 | 204.70 | Table 2, second Shackleton row |
| Shackleton Mazarico | -89.78 | 204.27 | Table 1, Mazarico row |
| Connecting Ridge Speyerer | -89.418 | 221.3 | Table 1, Speyerer row |
| Shackleton Speyerer | -89.740 | 201.2 | Table 1, Speyerer row |

These points span the Connecting Ridge and Shackleton region. They remain geographically clustered near the pole because that is where these publications identify their candidates. Coordinates are not displaced for a prettier globe. The existing downloaded PGDA surfaces cover all seven observers, as checked by prep's actual surface lookup and full horizon run. No extra DEM files or changed sampling parameters are introduced. The older rounded baseline JSON is not overwritten or used as a new catalog site.

## 16. Unchanged preparation and comparison

Run the same command for the expanded catalog:

```sh
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
```

Each site has 26,305 samples per body and 1,440 terrain bearings. Each Horizons query stays below 90,000 rows. The observer is 2 m above the selected DEM surface. The circular solar-area model, full coverage tracing, exact spherical geometry and 0.5 degree default radio margin are unchanged. Site01 is used where valid; the existing regional PGDA surfaces supply fallback and far terrain. Different sites need not use the same highest-resolution local tile.

Table 2 publication values use a 2 m observer and twenty years from 2017 through 2037. Table 1 compares surface-level calculations and observations; Mazarico uses a coarser LOLA model and Speyerer uses roughly one year of images. Their observer heights, periods and metrics must not be silently substituted into our uniform 2 m, 2027 through 2030 run. Original Mazarico exact simulation dates remain unverified. Earth and joint availability were not supplied by these comparison tables.

| Site | Mean solar area, % | Any Sun, % | Earth 0.5 deg, % | Both, % | Longest night, h |
|:--|--:|--:|--:|--:|--:|
| Connecting Ridge C1 | 86.7065 | 91.4462 | 52.6080 | 49.7567 | 111 |
| Connecting Ridge C2 | 78.7481 | 82.6680 | 53.0148 | 45.0882 | 164 |
| Shackleton S1 | 84.3611 | 89.1385 | 48.4831 | 44.7156 | 67 |
| Shackleton S2 | 81.6761 | 88.4466 | 51.2622 | 45.0654 | 65 |
| Shackleton Mazarico | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 26304 |
| Connecting Ridge Speyerer | 56.2341 | 57.5578 | 16.6781 | 9.4852 | 356 |
| Shackleton Speyerer | 74.2901 | 78.5850 | 36.8157 | 29.7787 | 140 |

The first table is our uniform 2027 through 2030 run. The next table is each publication's own comparison, verified in Glaser Table 1 or Table 2 rather than relabeled as a same-period reproduction.

| Published comparison | Mean illumination, % | Solar visibility, % | Maximum darkness, h |
|:--|--:|--:|--:|
| Connecting Ridge C1, Table 2 | 88.0 | 92.1 | 112 |
| Connecting Ridge C2, Table 2 | 78.1 | 82.0 | 165 |
| Shackleton S1, Table 2 | 85.5 | 89.4 | 66 |
| Shackleton S2, Table 2 | 82.1 | 89.8 | 65 |
| Shackleton Mazarico, Table 1 | 86.7 | 90.5 | Not given |
| Connecting Ridge Speyerer, Table 1 | 64.2 | Not given | 308 |
| Shackleton Speyerer, Table 1 | 71.6 | Not given | 145 |

| Site | Minimum horizon, deg | Maximum horizon, deg | Seconds |
|:--|--:|--:|--:|
| Connecting Ridge C1 | -2.738297 | -0.478074 | 120.559 |
| Connecting Ridge C2 | -3.195148 | 1.648819 | 119.719 |
| Shackleton S1 | -3.337964 | 1.335023 | 112.457 |
| Shackleton S2 | -2.656814 | 1.046622 | 116.391 |
| Shackleton Mazarico | 2.294499 | 10.465001 | 115.858 |
| Connecting Ridge Speyerer | -3.899722 | 5.033395 | 115.865 |
| Shackleton Speyerer | -2.639292 | 2.296859 | 111.558 |

Important non-reproduction: the printed Mazarico Shackleton coordinate yields zero Sun and Earth availability in our fine-terrain run, not the published 86.7% illumination. Its lowest horizon is +2.294499 degrees, while its maximum sampled solar-center elevation is +1.738698 degrees; even adding the 0.267 degree solar radius leaves the entire disk below every horizon bearing. This directly explains our computed zero without retuning. Printed coordinate precision, local terrain sampling, observer height and simulation periods differ from the source, but their individual contributions have not been isolated for this site. We do not move the coordinate onto a nearby crest or claim the old study was wrong. Connecting Ridge Speyerer also differs substantially; its printed position is retained unchanged.

All output files contain 26,305 samples per track and 1,440 horizon bins. Sizes in catalog order are 2,718,617; 2,726,966; 2,718,530; 2,718,350; 2,716,224; 2,717,960; 2,718,383 bytes. The preexisting C1 file regenerates identically. The older rounded-coordinate baseline remains byte-for-byte unchanged.

No parameter is fitted to these published percentages. Differences remain scientific results to report, not targets for retuning.

## 17. Lazy loading and one shared UTC time

Only the small metadata catalog is embedded with include_str!, the Rust compile-time equivalent of a C++ string literal generated from a file. The large per-site JSON files remain external resources. The app owns a vector of optional panorama models. None means no request has been started. Selecting a site initializes its model exactly once; switching back reuses the cached model. Each load has its own synchronized completion slot, so an old request cannot replace another site's state.

Native loads use the existing disk-directory search. Browser loads use assets/data followed by the same prep-derived ASCII filename. Data validation is unchanged: both tracks must be nonempty, ordered, finite and exactly time-aligned. Failures remain visible. All-sites ray mode starts the other requests on demand and shows a ready count plus individual load errors. Selected-only mode is the default and starts just C1 on initial launch.

The selected panorama owns the time slider, while the app remembers its UTC epoch. Switching a site restores that epoch before displaying the cached view. Every background model resolves its own track at that same epoch with partition_point: find the first timestamp greater than the epoch, then use the previous actual sample. Out-of-range epochs produce no ray rather than invented extrapolation. All generated grids are identical hourly grids, so lookups hit exactly. The Earth clearance is copied to all-site ray predicates; a pending model restores the requested clearance when its data arrives.

The selector is a list of published names. Globe picking uses the same stable catalog identifiers. Changing either selection updates the panorama, statuses, summary cards and highlighted marker. There is no arbitrary-coordinate landing-site creation, new region-selection workflow or online DEM loading.

## 18. Camera from zero

A camera converts a three-dimensional point into a two-dimensional image. The globe still uses a perspective projection from (0,0,3.3), but the model orientation is now controlled by yaw alpha and tilt theta:

$$M=R_x(\theta)R_y(\alpha).$$

The current default follows the actual Earth line of sight, with no fixed presentation tilt or automatic spin. Dragging changes the two angles and enters free orbit. Optional Auto-rotate continues smoothly from that orientation without changing the scientific date. View from Earth restores Earth-following mode. The [lighting guide](10-real-lighting-and-orbit.md) derives the camera and physical light directions. The time increment is bounded after an inactive frame to prevent a large jump. Wheel or pinch changes a zoom factor zeta, clamped from 1 to 1,024. The plus/minus buttons provide the same capability without a wheel or a multitouch device. Globe wheel input is consumed before the enclosing page can scroll.

Zoom multiplies the projection's horizontal and vertical scales by zeta. This narrows the field of view instead of moving the camera through the Moon. It is a presentation magnification, not additional texture or DEM detail. Drag sensitivity decreases with zoom so a magnified site remains controllable.

Go to site rotates the selected radial vector to the center of the image. For site latitude phi and east longitude lambda, choose alpha=-lambda and theta=phi. Substituting into the radial-vector formula gives M U=(0,0,1). Therefore the projected center is exactly zero in both axes. The transition follows the shortest yaw difference, uses a cubic easing function t squared times (3 minus 2t), and interpolates logarithmic zoom over 0.8 seconds to 128 times. It pauses auto-rotation. A new selection or manual interaction cancels the transition. The matrix and projection are shared by Moon vertices, marker projection and ray billboards.

Markers are six logical points in radius, with a two-point white outline; the selected marker is nine points and red rather than green. The selected marker is drawn last so a nearby unselected marker cannot cover it. Its published name is drawn alongside. Sphere-backside markers are not drawn through the Moon. Click/tap picking chooses the closest projected marker within fourteen logical points; the list remains usable when nearby published points overlap at overview scale. Zoom separates them without inventing offsets.

## 19. Thick rays and endpoint disks

WebGL implementations commonly support only one-pixel GL_LINES, so glLineWidth is not a portable solution. Each ray is now a pair of triangles, a camera-facing ribbon. Its endpoint carries a twenty-triangle circular billboard disk. Position uses three floats and the clip-space offset uses two more, matching a twenty-byte vertex stride. This reuses the existing vertex-buffer ownership and constant-color material branch.

The shared u_ray_mode integer uniform explicitly uses highp in both shader stages. WebGL's linker rejects the default mismatch between vertex high precision and fragment medium precision; native linking alone does not catch it. Browser verification exposed this mismatch and the explicit qualifier fixes it. A source-contract regression guards both declarations.

Let projected endpoints be a and b in normalized device coordinates. Convert b-a into pixel space and normalize its perpendicular to obtain a two-dimensional screen normal n. For a desired logical width w and device scale s, the half-offset in normalized device coordinates is:

$$\delta_x=n_x\frac{ws}{W},\qquad \delta_y=n_y\frac{ws}{H}.$$

W and H are the physical viewport dimensions. Vertices receive plus/minus delta. The vertex shader adds offset times homogeneous clip w to clip x/y. Perspective division then cancels that w, leaving a constant screen width rather than a ribbon that collapses with depth. Width is 2.8 logical points; endpoint radius is 5.5 points. A degenerate camera-aligned ray is guarded against division by zero and still receives an endpoint disk.

The direction remains the ENU-derived Moon-fixed unit vector from the preceding guide. For legibility at high zoom, illustrative ray length becomes 0.6 divided by zeta lunar radii. This preserves roughly similar on-screen reach and keeps endpoint disks in useful proximity to the magnified marker. It is explicitly not a distance scale.

Blocked bodies use 0.35 times their display RGB. Unlike the earlier thin physical-depth lines, the new rays are direction diagrams over the visible surface: they do not depth-test against the smooth sphere. Otherwise a blocked direction could disappear below that sphere entirely at high zoom, defeating its purpose. Backside origins are still rejected. Real terrain and the radio margin, not the globe's illustrative photometric light, determine the styling. Sun is yellow and Earth is blue. All-sites mode uses each marker's own current sample; selected-only mode submits just that marker's pair.

## 20. Checks and API index

Tests cover catalog identifiers/filenames, shared-epoch lookup with out-of-range rejection, radio-margin changes, exact go-to centering, finite ribbon vertices and pixel width. Existing core and loader tests remain applicable. Visual checks must exercise the list, marker picking, go-to, drag/pause/resume, all-site loading and phone zoom buttons, not only page loading. The browser title is Karaya Explorer. No step-two menu, help or logo redesign is included.

- `abs`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `acos`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `active_texture`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `add`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `add_enabled`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `all`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `allocate_exact_size`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `allocate_ui`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `allow_double_click_reset`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `allow_drag`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `allow_scroll`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `allow_zoom`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `and_then`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `any`: [annotated app/src/data.rs](#212-appsrcdatars)
- `Arc::clone`: [annotated app/src/data.rs](#212-appsrcdatars)
- `Arc::new`: [annotated app/src/data.rs](#212-appsrcdatars)
- `as_array`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `as_chunks`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `as_deref`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `as_f64`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `as_mut`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `as_raw`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `as_ref`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `asin`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `aspect_ratio`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `assert!`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `assert_eq!`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `assets::load`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `assets::preferences`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `assets::select`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `at`: [annotated app/src/data.rs](#212-appsrcdatars)
- `atan2`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `attach_shader`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `availability`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `available_size`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `available_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `bind_attrib_location`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `bind_buffer`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `bind_texture`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `bind_vertex_array`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `BodyDirection`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `buffer_data_u8_slice`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `bundled_three_years_are_aligned_hourly_and_have_terrain`: [annotated app/src/data.rs](#212-appsrcdatars)
- `button`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `bytemuck::cast_slice`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `catalog_and_lazy_filenames`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `center`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `centered_site_and_billboard_width`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `chain`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `changed`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `checkbox`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `circle_filled`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `circle_stroke`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `clamp`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `clear`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `clear_depth_f32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `clicked`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `clone`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `collect`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `color`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Color32::from_gray`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `colored_label`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `columns`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `compile_shader`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `contains`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `copied`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `cos`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `count`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `crate::sky::show`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `create_buffer`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `create_program`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `create_shader`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `create_texture`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `create_vertex_array`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `ctx`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `cull_face`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `data::load`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `default_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `default_x_bounds`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `delete_buffer`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `delete_program`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `delete_shader`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `delete_texture`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `delete_vertex_array`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `delta`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `depth_func`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `depth_mask`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `destroy`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `detach_shader`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `direction`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `directions`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `directions_at`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `disable`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `display`: [annotated app/src/data.rs](#212-appsrcdatars)
- `distance_sq`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `dragged`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `drain`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `draw_arrays`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `draw_elements`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `earthshine_phase_and_scale`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `earthshine_ratio`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `eframe::egui::CentralPanel::default`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::ComboBox::from_id_salt`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::Layout::top_down`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::ScrollArea::vertical`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::vec2`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `egui::Button::new`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `egui::Color32::from_rgb`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `egui::ComboBox::from_id_salt`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::FontId::proportional`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `egui::Frame::group`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Grid::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::RichText::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Sense::click_and_drag`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `egui::Slider::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Stroke::new`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `egui::vec2`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Window::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui_glow::CallbackFn::new`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `egui_glow::ShaderVersion::get`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `ehttp::fetch`: [annotated app/src/data.rs](#212-appsrcdatars)
- `ehttp::Request::get`: [annotated app/src/data.rs](#212-appsrcdatars)
- `enable`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `enable_vertex_attrib_array`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `end_row`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `enumerate`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `env!`: [annotated app/src/data.rs](#212-appsrcdatars)
- `eprintln!`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `exp`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `expand`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `extend_from_slice`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `f64::from`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `filename`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `fill`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `filter`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `filter_map`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `find`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `fixed_decimals`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `fixture`: [annotated app/src/data.rs](#212-appsrcdatars)
- `flat_and_interpolated_terrain_visibility`: [annotated app/src/data.rs](#212-appsrcdatars)
- `flat_map`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `flatten`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `fold`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `format`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `format!`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `frame`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `front_face`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `generate_mipmap`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_error`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_parameter_i32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_program_info_log`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_program_link_status`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_shader_compile_status`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_shader_info_log`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `get_uniform_location`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `gl`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `Globe`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `globe::Globe::new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `globe::moon_fixed`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `glow::PixelUnpackData::Slice`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `heading`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `height`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `hline`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `HLine::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `horizon_at`: [annotated app/src/data.rs](#212-appsrcdatars)
- `horizontal`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `horizontal_wrapped`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `hovered`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `hypot`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `include_bytes!`: [annotated app/src/data.rs](#212-appsrcdatars)
- `include_str!`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `include_y`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `input`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `input_mut`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `interact_pointer_pos`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `into`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `into_iter`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `is_ascii_alphanumeric`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `is_dir`: [annotated app/src/data.rs](#212-appsrcdatars)
- `is_empty`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `is_err`: [annotated app/src/data.rs](#212-appsrcdatars)
- `is_finite`: [annotated app/src/data.rs](#212-appsrcdatars)
- `is_new_shader_interface`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `is_none`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `is_none_or`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `is_positive`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `is_some`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `iter`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `join`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `json!`: [annotated app/src/data.rs](#212-appsrcdatars)
- `label`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `last`: [annotated app/src/data.rs](#212-appsrcdatars)
- `last_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `left_top`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `legend`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Legend::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `len`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `line`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Line::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `link_program`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `ln`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `load`: [annotated app/src/data.rs](#212-appsrcdatars)
- `load_error`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `local_direction`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `local_sky_frame`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `lock`: [annotated app/src/data.rs](#212-appsrcdatars)
- `lunar_core::stats`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `lunar_core::Stats::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `lunar_core::sun_fraction`: [annotated app/src/data.rs](#212-appsrcdatars)
- `map`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `map_err`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `map_or`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `map_or_else`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `margin`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `Marker`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `marker`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `math::direction`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `math::earthshine_ratio`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `math::local_direction`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `math::sphere`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `max`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `mesh_orientation_and_seam`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `min`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `model`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `moon_fixed`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `moon_fixed_ephemeris_libration_ranges`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `MoonApp`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `Mutex::new`: [annotated app/src/data.rs](#212-appsrcdatars)
- `new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `num_columns`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `ok`: [annotated app/src/data.rs](#212-appsrcdatars)
- `ok_or`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `on_exit`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `on_hover_text`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `open`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `or_else`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `paint`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `painter`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `painter_at`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `panorama::Panorama::new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `parent`: [annotated app/src/data.rs](#212-appsrcdatars)
- `parse`: [annotated app/src/data.rs](#212-appsrcdatars)
- `partition_point`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Pending::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `perspective`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `pixel_store_i32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `pixels_per_point`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `Plot::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `pointer_mut`: [annotated app/src/data.rs](#212-appsrcdatars)
- `points`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Points::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `powi`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `project`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `projection`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `projection_depth_and_aspect`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `push`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `radio_value`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `radius`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `rejects_invalid_inputs`: [annotated app/src/data.rs](#212-appsrcdatars)
- `rem_euclid`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Renderer::new`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `replace`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `request_repaint`: [annotated app/src/data.rs](#212-appsrcdatars)
- `ribbon`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `rotation`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `saturating_sub`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `scene_identity_empty_and_invalid`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `segments`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `selectable_value`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `selected_text`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `serde_json::from_slice`: [annotated app/src/data.rs](#212-appsrcdatars)
- `serde_json::from_str`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `serde_json::to_vec`: [annotated app/src/data.rs](#212-appsrcdatars)
- `set_margin_fraction`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `set_max_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `set_scene`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `set_theme`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `set_time`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `set_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `shader_source`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `show`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `show_panorama`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `show_ui`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `show_value`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sin`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `sin_cos`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `site`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `site_projection_and_south_tilt`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `SiteData::parse`: [annotated app/src/data.rs](#212-appsrcdatars)
- `size`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `small`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `spacing`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `spacing_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sphere`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `split`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `sqrt`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `std::array::from_fn`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `std::env::current_exe`: [annotated app/src/data.rs](#212-appsrcdatars)
- `std::env::var`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `std::env::var_os`: [annotated app/src/data.rs](#212-appsrcdatars)
- `std::fs::read`: [annotated app/src/data.rs](#212-appsrcdatars)
- `std::path::Path::new`: [annotated app/src/data.rs](#212-appsrcdatars)
- `std::thread::spawn`: [annotated app/src/data.rs](#212-appsrcdatars)
- `strong`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `style`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `style_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `suffix`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sum`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `summary_uses_full_duration_and_current_clearance`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `take`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `tan`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `technical`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `tex_image_2d`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `tex_parameter_i32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `text`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `then`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `time`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `to_degrees`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `to_radians`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `to_string`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `transpose`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `trim`: [annotated app/src/data.rs](#212-appsrcdatars)
- `ui`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `uniform_1_f32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `uniform_1_i32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `uniform_3_f32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `unwrap`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `unwrap_or`: [annotated app/src/data.rs](#212-appsrcdatars)
- `unwrap_or_else`: [annotated app/src/data.rs](#212-appsrcdatars)
- `update`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `update_earth_margin`: [annotated app/src/data.rs](#212-appsrcdatars)
- `upload_texture`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `use_program`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `utc_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `vec!`: [annotated app/src/data.rs](#212-appsrcdatars)
- `Vec::new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `Vec::with_capacity`: [annotated globe/src/math.rs](08-rays.md#141-globesrcmathrs)
- `version_declaration`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `vertex_attrib_pointer_f32`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `vertical`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `viewport_in_pixels`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `vscroll`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `web_sys::console::log_1`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)
- `width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `window`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `window_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `windows`: [annotated app/src/data.rs](#212-appsrcdatars)
- `windows_are_utc_intervals_not_sample_counts`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `with_layout`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `wraps_do_not_draw_chords`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_axis_formatter`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_axis_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_grid_spacer`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `y_axis_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `zip`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `zoom_delta`: [annotated globe/src/lib.rs](08-rays.md#142-globesrclibrs)

## 21. Exact additional source quotations

### 21.1. sites.json

```json
[
  {
    "name": "Connecting Ridge C1",
    "lat_deg": -89.4398,
    "lon_deg": 222.83,
    "source": "Glaser et al. (2017), Table 2, 2 m observer; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  },
  {
    "name": "Connecting Ridge C2",
    "lat_deg": -89.313,
    "lon_deg": 245.98,
    "source": "Glaser et al. (2017), Table 2, 2 m; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  },
  {
    "name": "Shackleton S1",
    "lat_deg": -89.6866,
    "lon_deg": 197.19,
    "source": "Glaser et al. (2017), Table 2 first Shackleton row, 2 m; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  },
  {
    "name": "Shackleton S2",
    "lat_deg": -89.7815,
    "lon_deg": 204.7,
    "source": "Glaser et al. (2017), Table 2 second Shackleton row, 2 m; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  },
  {
    "name": "Shackleton Mazarico",
    "lat_deg": -89.78,
    "lon_deg": 204.27,
    "source": "Mazarico et al. (2011), coordinates and results reproduced in peer-reviewed Glaser et al. (2017), Table 1; secondary verification, not primary-paper recovery; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  },
  {
    "name": "Connecting Ridge Speyerer",
    "lat_deg": -89.418,
    "lon_deg": 221.3,
    "source": "Speyerer and Robinson (2013), coordinates and results reproduced in peer-reviewed Glaser et al. (2017), Table 1; secondary verification; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  },
  {
    "name": "Shackleton Speyerer",
    "lat_deg": -89.74,
    "lon_deg": 201.2,
    "source": "Speyerer and Robinson (2013), coordinates and results reproduced in peer-reviewed Glaser et al. (2017), Table 1; secondary verification; https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf"
  }
]
```

### 21.2. app/src/data.rs

```rust
//! Validated Horizons data and asynchronous disk/HTTP loading, with no DEM policy.

// Import UTC conversion; C++ chrono provides calendar/time-point types.
use chrono::{DateTime, Utc};
// Import shared completion storage; C++ shared_ptr and a mutex-protected value.
use eframe::egui::{self, mutex::Mutex};
// Import the scientific contracts rather than duplicating visibility math; C++ includes their header.
use lunar_core::{EARTH_MARGIN_DEG, Horizon, Site, Track, visible};
// Import the derive trait used by the JSON decoder; C++ needs explicit serialization adapters.
use serde::Deserialize;
// Import reference-counted ownership; C++ std::shared_ptr.
use std::sync::Arc;

// Keep one filename for native disk and web HTTP paths; C++ constexpr string_view.
pub(crate) const FILE: &str = "connecting-ridge-c1.json";

// Generate JSON deserialization for the prep schema; C++ supplies field adapters manually.
#[derive(Deserialize)]
// Own a validated, aligned pair of tracks; C++ aggregate with vectors and optional horizon.
pub(crate) struct SiteData
{
	// Retain geographic latitude and east longitude; C++ a Site value member.
	pub site: Site,
	// Own the Sun's sample arrays; C++ a Track value member.
	pub sun: Track,
	// Own the Earth's sample arrays on the same UTC grid; C++ a Track value member.
	pub earth: Track,
	// Preserve null as absent terrain, not as an invented DEM; C++ optional<Horizon>.
	pub horizon: Option<Horizon>,
	// This cache is computed after validation, not read from JSON; C++ excludes a derived member from serialization.
	#[serde(skip)]
	// Cache the core's Earth center visibility once per load; C++ vector<bool>.
	pub earth_above: Vec<bool>,
}

// Attach helpers to this owned model; C++ member definitions outside the class.
impl SiteData
{
	// Decode bytes and enforce preconditions before any indexing; C++ expected<SiteData,string>.
	pub fn parse(bytes: &[u8]) -> Result<Self, String>
	{
		// Borrow bytes without copying and propagate JSON errors; C++ span plus expected error propagation.
		let mut data: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
		// Reject unusable site metadata; C++ checks finite doubles and numeric intervals.
		if data.site.name.trim().is_empty()
			|| !data.site.lat_deg.is_finite()
			|| !data.site.lon_deg.is_finite()
			|| !(-90.0..=90.0).contains(&data.site.lat_deg)
		{
			// Return a diagnostic instead of creating a misleading marker; C++ unexpected<string>.
			return Err("Invalid site name or coordinates".into());
		}
		// Validate both borrowed tracks identically; C++ range-for over two const pointers.
		for track in [&data.sun, &data.earth]
		{
			// Refuse empty/misaligned arrays, unordered times and invalid directions; C++ short-circuit predicates.
			if track.t.is_empty()
				|| track.t.len() != track.az_deg.len()
				|| track.t.len() != track.el_deg.len()
				|| track.t.windows(2).any(|t| t[0] >= t[1])
				|| track
					.t
					.iter()
					.any(|&t| DateTime::<Utc>::from_timestamp(t, 0).is_none())
				|| track
					.az_deg
					.iter()
					.any(|a| !a.is_finite() || !(0.0..=360.0).contains(a))
				|| track
					.el_deg
					.iter()
					.any(|e| !e.is_finite() || !(-90.0..=90.0).contains(e))
			{
				// Avoid indexing invalid data or bridging missing arrays; C++ unexpected<string>.
				return Err("Invalid or empty track arrays, timestamps, or angles".into());
			}
		}
		// Require exact timestamp correspondence, not merely equal lengths; C++ vector equality.
		if data.sun.t != data.earth.t
		{
			// Report incompatible tracks; C++ returns an error alternative.
			return Err("Sun and Earth timestamps differ".into());
		}
		// Check supplied terrain only; no DEM is read or selected here; C++ if-init with optional.
		if let Some(h) = &data.horizon
			&& (h.h_deg.is_empty()
				|| h.h_deg
					.iter()
					.any(|e| !e.is_finite() || !(-90.0..=90.0).contains(e)))
		{
			// A supplied profile must actually contain valid bins; C++ unexpected<string>.
			return Err("A supplied horizon needs nonempty finite elevation bins".into());
		}
		// Cache the default communication clearance, not mere center-above-horizon visibility; C++ member call.
		data.update_earth_margin(EARTH_MARGIN_DEG);
		// Move the validated model to the caller; C++ expected containing a moved value.
		Ok(data)
	}

	// Interpolate a supplied profile or explicitly choose flat zero; C++ const member returning float.
	pub fn horizon_at(&self, az: f32) -> f32
	{
		// Do not use core's empty-profile -90 convention for missing DEM; C++ optional transform with fallback.
		self.horizon.as_ref().map_or(0.0, |h| h.at(az))
	}

	// Recompute only when the user changes clearance; C++ mutating cache member function.
	pub fn update_earth_margin(&mut self, margin_deg: f32)
	{
		// Retain the labeled flat fallback for older datasets; C++ temporary constant profile.
		let flat = Horizon { h_deg: vec![0.0] };
		// Delegate the strict elevation > horizon + margin criterion to core; C++ vector assignment from a scientific function.
		self.earth_above = visible(
			&self.earth,
			self.horizon.as_ref().unwrap_or(&flat),
			margin_deg,
		);
	}
}

// Name pending/success/failure shared storage; C++ shared_ptr<mutex-protected optional<expected<SiteData,string>>>.
pub(crate) type Pending = Arc<Mutex<Option<Result<SiteData, String>>>>;

// Start one asynchronous load and wake egui when it finishes; C++ factory returning shared completion state.
pub(crate) fn load(ctx: egui::Context, file: String) -> Pending
{
	// None means pending, distinct from success and failure; C++ optional starts disengaged.
	let pending = Arc::new(Mutex::new(None));
	// Give the completion its own shared owner; C++ copies a shared_ptr.
	let completed = Arc::clone(&pending);
	// Move captured owners into a one-shot callback; C++ a mutable value-capturing lambda.
	let done = move |bytes: Result<Vec<u8>, String>| {
		// Decode only on successful I/O, then atomically publish the result; C++ expected::and_then under lock_guard.
		*completed.lock() = Some(bytes.and_then(|b| SiteData::parse(&b)));
		// Wake the event loop from either a worker or browser completion; C++ posts a redraw event.
		ctx.request_repaint();
	};
	// Compile the disk loader only for native targets; C++ #if around a function body.
	#[cfg(not(target_arch = "wasm32"))]
	// Keep disk I/O off the UI thread; C++ std::thread with a value-captured callback.
	std::thread::spawn(move || {
		// Accept a deployment override; C++ getenv converted to optional<filesystem::path>.
		let directory = std::env::var_os("MOON_DATA_DIR")
			.map(std::path::PathBuf::from)
			.unwrap_or_else(|| {
				// Prefer assets/data beside the executable; C++ filesystem::path with optional error handling.
				std::env::current_exe()
					.ok()
					.and_then(|p| p.parent().map(|p| p.join("assets/data")))
					.filter(|p| p.is_dir())
					// Fall back to the workspace app assets; C++ build-defined source path.
					.unwrap_or_else(|| {
						std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/data")
					})
			});
		// Include the failing path in the error instead of silently switching files; C++ filesystem read with expected.
		let path = directory.join(&file);
		// Publish exactly one disk result; C++ invokes the captured completion.
		done(std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display())));
	});
	// Compile HTTP loading only for the browser; C++ #if for platform code.
	#[cfg(target_arch = "wasm32")]
	// Fetch the same relative resource copied by Trunk, without embedding it in WASM; C++ asynchronous HTTP request.
	ehttp::fetch(
		ehttp::Request::get(format!("assets/data/{file}")),
		move |response| {
			// Reject HTTP errors even if their bodies happen to resemble JSON; C++ expected::and_then.
			done(response.and_then(|r| {
				if r.ok
				{
					Ok(r.bytes)
				}
				else
				{
					Err(format!("HTTP {} loading {file}", r.status))
				}
			}));
		},
	);
	// Return another owner while completion stays alive independently; C++ returns shared_ptr.
	pending
}

// Compile test-only fixtures out of release binaries; C++ conditional test compilation.
#[cfg(test)]
// Isolate test names; C++ anonymous/test namespace.
mod tests
{
	// Access private implementation helpers; C++ includes the implementation's test interface.
	use super::*;

	// Build a tiny editable valid schema; C++ returns a JSON value object.
	fn fixture() -> serde_json::Value
	{
		// Two aligned samples cross north, with Earth above flat zero; C++ constructs an object/array tree.
		serde_json::json!({"site":{"name":"Test","lat_deg":-89.45,"lon_deg":222.69},
			"sun":{"t":[0,3600],"az_deg":[359,1],"el_deg":[0,1]},
			"earth":{"t":[0,3600],"az_deg":[137,138],"el_deg":[1,-1]},"horizon":null})
	}

	// Register a test; C++ testing framework macro.
	#[test]
	// Verify the committed Horizons payload, not a substitute dataset; C++ a data regression test.
	fn bundled_three_years_are_aligned_hourly_and_have_terrain()
	{
		// Embed only in the test executable, never in the app/WASM; C++ test-only binary fixture.
		let d = SiteData::parse(include_bytes!("../assets/data/connecting-ridge-c1.json")).unwrap();
		// Count inclusive interval endpoints; C++ assert equality.
		assert_eq!(d.sun.t.len(), 26305);
		// Keep the request below the published row limit; C++ assert comparison.
		assert!(d.sun.t.len() < 90_000);
		// Check UTC cadence, independently of sample count; C++ ranges::all_of adjacent pairs.
		assert!(d.sun.t.windows(2).all(|t| t[1] - t[0] == 3600));
		// Verify exact endpoint dates; C++ chrono time-point equality.
		assert_eq!(d.sun.t[0], 1_798_761_600);
		// Include the leap year 2028 through the 2030 endpoint; C++ assert equality.
		assert_eq!(*d.sun.t.last().unwrap(), 1_893_456_000);
		// Real terrain uses 1,440 geographic azimuths; C++ optional extraction and vector size assertion.
		assert_eq!(d.horizon.as_ref().unwrap().h_deg.len(), 1440);
		// Keep the selected position tied to the published 2 m C1 site, not a rounded label; C++ coordinate regression.
		assert_eq!((d.site.lat_deg, d.site.lon_deg), (-89.4398, 222.83));
	}

	// Register a test; C++ testing framework macro.
	#[test]
	// Check normal flat and supplied-profile visibility; C++ unit test.
	fn flat_and_interpolated_terrain_visibility()
	{
		// Own an editable fixture; C++ JSON value.
		let mut v = fixture();
		// Parse the null fallback; C++ serialize then deserialize for schema coverage.
		let mut flat = SiteData::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
		// Earth centers use strict elevation > horizon; C++ vector comparison.
		assert_eq!(flat.earth_above, [true, false]);
		// Raising the clearance to one degree rejects an Earth center exactly one degree high; C++ strict-threshold check.
		flat.update_earth_margin(1.0);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(flat.earth_above, [false, false]);
		// Restore the default before exercising the existing horizon checks; C++ cache update.
		flat.update_earth_margin(EARTH_MARGIN_DEG);
		// At the horizon half the solar disk is visible; C++ call the scientific function.
		assert!((lunar_core::sun_fraction(0.0, flat.horizon_at(359.0)) - 0.5).abs() < 1e-6);
		// Supply a synthetic cyclic profile, not a chosen DEM; C++ replace a JSON member.
		v["horizon"] = serde_json::json!({"h_deg":[1,3]});
		// Parse and evaluate terrain through the same code path; C++ checked conversion.
		let terrain = SiteData::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
		// Uniform bins interpolate at 90 degrees; C++ floating-point tolerance assertion.
		assert!((terrain.horizon_at(90.0) - 2.0).abs() < 1e-6);
		// Both Earth samples are now below the supplied profile; C++ ranges::none_of.
		assert!(terrain.earth_above.iter().all(|v| !v));
		// Values straddling half a degree prove parsing uses the default clearance, not zero; C++ threshold fixture.
		let mut close = fixture();
		close["earth"]["el_deg"] = serde_json::json!([0.25, 0.75]);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let clearance = SiteData::parse(&serde_json::to_vec(&close).unwrap()).unwrap();
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(clearance.earth_above, [false, true]);
	}

	// Register a test; C++ testing framework macro.
	#[test]
	// Cover malformed, empty, mismatched and invalid numeric inputs; C++ table-driven rejection test.
	fn rejects_invalid_inputs()
	{
		// Empty input is not an empty successful dataset; C++ assert error alternative.
		assert!(SiteData::parse(b"").is_err());
		// Try independent corruptions of otherwise valid data; C++ range-for over replacement pairs.
		for (pointer, replacement) in [
			("/sun/t", serde_json::json!([])),
			("/sun/az_deg", serde_json::json!([1])),
			("/sun/t", serde_json::json!([0, 0])),
			("/earth/t", serde_json::json!([0, 3601])),
			("/site/lat_deg", serde_json::json!(-100)),
			("/sun/el_deg", serde_json::json!([91, 0])),
			("/sun/az_deg", serde_json::json!([361, 0])),
			("/horizon", serde_json::json!({"h_deg":[]})),
			("/horizon", serde_json::json!({"h_deg":[100]})),
		]
		{
			// Reset the fixture so errors do not mask one another; C++ a fresh local JSON object.
			let mut v = fixture();
			// Replace the selected nested member; C++ assign through a JSON pointer.
			*v.pointer_mut(pointer).unwrap() = replacement;
			// The production parser must reject every corruption; C++ assert with diagnostic context.
			assert!(
				SiteData::parse(&serde_json::to_vec(&v).unwrap()).is_err(),
				"{pointer}"
			);
		}
	}
}
```

### 21.3. globe/src/moon.vert

```glsl
// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Read one unit object-space sphere vertex from attribute location zero.
in vec3 a_position;
// Read the mesh UV from location one; fragment shading reconstructs UV from direction for seam-safe derivatives.
in vec2 a_uv;
// Receive the column-major view-to-clip perspective matrix from Rust.
uniform mat4 u_projection;
// Receive Rx(tilt)*Ry(yaw), shared with normals and CPU site markers; C++ glUniformMatrix3fv.
uniform mat3 u_model;
// Match integer precision in both stages, required by WebGL's uniform linker; C++ glUniform1i.
uniform highp int u_ray_mode;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
// Keep default smooth interpolation: derivatives of coverage-dependent centroid values worsened the filtering artifact.
out vec3 v_position;
// Pass mesh UV for the vertex interface; direction-based fragment sampling does not depend on it.
out vec2 v_uv;
// Pass unrotated object position so texture coordinates remain attached to the Moon.
out vec3 v_object;

// Run once per vertex or fragment respectively; GLSL entry points have no C++ object ownership.
void main()
{
	// Rotate about geographic north, then tilt the south pole toward the camera; C++ matrix-vector multiplication.
	vec3 p = u_model * a_position;
	// Export the rotated world position for perspective view direction and geometric day/night masking.
	v_position = p;
	// Export the mesh UV, while the fragment stage uses analytic direction-based sampling.
	v_uv = a_uv;
	// Export the original direction for texture lookup independently of rotation.
	v_object = a_position;
	// Translate world position to view space and project to homogeneous clip coordinates; rasterization divides by w.
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, 3.3), 1.0);
	// Multiplication by homogeneous w preserves constant screen width after perspective division.
	if (u_ray_mode != 0) gl_Position.xy += a_uv * gl_Position.w;
}
```
