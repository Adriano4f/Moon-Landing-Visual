# Sun and Earth directions on the rotating Moon

## Thin coverage and the optional global Sun field

Site ribbons use a 2.5-logical-point support with alpha `1 - smoothstep(0.15, 1, abs(x))` across signed transverse coordinate x. Its integrated width is approximately 1.44 logical points. Endpoint disks have a 2-point support radius and the same smooth radial coverage. Seven floats per vertex carry position, clip offset and independent coverage coordinates. Alpha blending is enabled only for diagrams; sphere opacity is unchanged. This works with WebGL triangles, without implementation-dependent line width.

The Rays section includes a separate, initially disabled **Rays across the whole Moon** option. A 512-point Fibonacci sphere uses equal-area latitude bands and a golden-angle longitude. Only points with positive dot product against the current Moon-fixed Sun direction contribute, approximately 256 rays. Each ray runs from an illustrative upstream endpoint to its surface intersection. These parallel incoming rays use the actual slider-time direction, sphere depth occlusion and a single batched draw. They do not request other sites, calculate terrain shadows or affect statistics. There are no global Earth rays. Missing lighting draws no field; invalid/zero directions produce no samples.

## 9. The local frame from zero

The panorama describes the sky at one landing site. Its azimuth A is a direction around the observer, measured from geographic north toward east. Elevation e is the angle above the observer's horizontal plane. A ray in the globe must first convert those angles into a three-dimensional direction fixed to the Moon, then apply the same presentation rotation as the site marker. It must not treat azimuth as longitude or elevation as latitude on the Moon.

The renderer's axes are unusual but explicit: +y is lunar north, +z is the equator at longitude zero, and +x is the equator at 90 degrees east. Latitude is phi and east longitude is lambda. All trigonometric arguments below are radians. The unit radial vector is the local up direction:

$$\mathbf U=(\cos\phi\sin\lambda,\ \sin\phi,\ \cos\phi\cos\lambda).$$

Increasing latitude while holding longitude fixed moves north. Differentiate U with respect to phi to obtain the unit north tangent:

$$\mathbf N=(-\sin\phi\sin\lambda,\ \cos\phi,\ -\sin\phi\cos\lambda).$$

Increasing longitude moves east. Its derivative has length cos(phi), so normalizing analytically gives:

$$\mathbf E=(\cos\lambda,\ 0,\ -\sin\lambda).$$

These three vectors are perpendicular and have unit length. The explicit east expression avoids division by cos(phi) near a pole. At an exact pole, longitude still chooses the local north/east convention; there is no globally unique north tangent without that choice. Their ordering is a geographic basis, not a requirement to change the renderer's existing Cartesian axes.

A local unit direction has east component cos(e)sin(A), north component cos(e)cos(A), and up component sin(e). Therefore:

$$\mathbf d=\cos e\sin A\,\mathbf E+\cos e\cos A\,\mathbf N+\sin e\,\mathbf U.$$

At elevation 90 degrees, d=U for every azimuth. At azimuth zero and elevation zero, d=N. At azimuth 90 and elevation zero, d=E. For latitude 30, longitude 60, azimuth 45 and elevation 30 degrees, the numeric direction is approximately (0.41602117, 0.78033010, -0.46691680). The regression test checks this nontrivial example and up/north at the actual selected C1 site, plus invalid angles. Norms and angles are unaffected by the subsequent rigid presentation rotation.

## 10. From the time slider to the renderer

Panorama::directions reads the current index from the validated aligned Sun/Earth tracks. It returns two BodyDirection values, Sun first and Earth second. Each carries az_deg, el_deg and visible. BodyDirection is a small copyable struct, like a C++ aggregate passed by value, not a reference to arrays owned by the app. No lunar_core dependency is added to globe.

For the Sun, visible means positive visible solar area using the actual terrain profile. For Earth it means the current radio safety-clearance predicate, including the adjustable 0.5 through 1.0 degree margin. Thus an Earth ray can be dim even if part of the blue disk is above terrain. This deliberately matches the panorama status rather than silently adopting a second radio definition. Loading or errors export None, the Rust equivalent of C++ nullopt. Invalid geographic or sky angles clear the pair rather than retaining a stale direction.

The app now passes a lightweight scene through Globe::set_scene: all marker coordinates and optional directions for the selected site or all loaded sites. Each pair is converted with math::local_direction using its own site's basis. The deferred GL callback captures copies of directions, origins and camera parameters together, so one draw uses one coherent sample. Slider edits appear on the following animation redraw, as do the existing top statuses. Presentation rotation does not change the selected scientific epoch. [The interactive-controls guide](09-sites-and-globe-controls.md) describes lazy per-site loading and shared UTC lookup.

The integration regression uses unequal-interval data with Sun elevation -1 then +1 degrees and Earth at +0.75 degrees. It edits the sample index and the radio clearance, verifies the exported values/flags, then removes the loaded data and verifies absence. This checks the bridge, not merely the stand-alone trigonometry.

## 11. Drawing real object-space lines

The marker is at U on the illustrative unit sphere. Each line begins at that exact same point and ends at:

$$\mathbf q=\mathbf U+\frac{0.6}{\zeta}\mathbf d.$$

The length is 0.6 lunar radii at overview zoom zeta=1, shortening as zoom increases to retain readable endpoints. It represents direction only, not the Sun/Earth distances, body size or a measured mast height. The overlay explicitly says length is not to scale. Endpoint disks are now camera-facing triangle fans, not physical body-size models.

The shader transforms both endpoints using the same uniforms already used for the Moon:

$$\mathbf p_{clip}=P\left[\begin{array}{c}M\mathbf p-(0,0,3.3)\\1\end{array}\right],\qquad M=R_x(\theta)R_y(\alpha).$$

P is the zoomed perspective matrix, M the shared rigid rotation, alpha the longitude rotation and theta the camera tilt, now initialized from the actual Earth direction rather than a fixed -30 degrees. Endpoints use actual three-dimensional geometry rather than a guessed two-dimensional arrow angle. Their camera-facing ribbon offsets are derived in the next guide. The Sun ray near the pole is almost tangent to the lunar surface because solar elevation is small. On screen that tangent may be foreshortened; horizontal in the local sky does not require a perfectly horizontal screen line.

| Geometry | Vertex layout | Draw call |
|:--|:--|:--|
| Moon | Position and UV, 20-byte stride | Indexed triangles |
| Direction ray and tip | Position and clip offset, 20-byte stride | Ribbon and disk triangles |

A separate vertex array object stores the ray layout. A small buffer receives the ribbon and endpoint triangles with STREAM_DRAW, meaning frequently replaced data. It is registered immediately in the renderer's existing owned buffer inventory; the additional vertex array is destroyed on shutdown and on partial initialization failure. The ray-buffer field is an alias, not a second owner. Cleanup clears it after deleting the owned buffers, preventing double deletion. The textured sphere's static mesh is never overwritten.

The same vertex shader is reused. A uniform integer u_ray_mode chooses a constant-color early return in the fragment shader before texture derivatives, normal mapping or Hapke calculations. Zero is restored before every Moon draw; one is selected only for the diagrams. Yellow is Sun, blue is Earth. Available directions use full RGB brightness; blocked directions use 0.35 times those display values. Blocked-state dimming remains independent of smooth edge-alpha blending. The Moon remains opaque and depth-tested.

The sphere is drawn first. The newer ribbons are direction diagrams over its visible surface: their origins must face the camera, but their segments do not depth-test against the smooth sphere. This keeps a blocked body's direction readable at high zoom instead of hiding its entire short segment underground. Their exact origin can be covered by the outlined marker, drawn afterward. Real terrain determines dimming in the app. The [real-lighting extension](10-real-lighting-and-orbit.md) now lights the globe using that same scientific Sun direction, independently of the unlit ray colors.

## 12. Verification and limits

Run workspace tests, clippy, native release and WASM checks. Build Trunk from app/ and verify Chrome phone emulation has no page or console errors. Native captures at sample 0 and sample 1138 exercise opposite sunlight/radio states. Inspect rays.png and rays_dark.png, plus rays_mobile.png and a close crop around the marker: origins agree, colors follow the selected status, and the low-elevation Sun ray lies near the local tangent rather than pointing radially away from the pole. Browser testing uses Pixel 7 emulation with DPR 2; it is not a physical phone or the user's GPU.

The ray painter does not fetch ephemerides, change horizons, alter MSAA, relight the Moon or modify solar/radio statistics. Camera controls, thin anti-aliased ribbons, endpoint disks and all-sites selection are documented in the next guide. A nearly camera-aligned direction can project to a short segment; that is perspective, not a changed scientific elevation.

## 13. Ray API index

- `abs`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `acos`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `active_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `add`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `add_enabled`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `advance_upgrade`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `all`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `allocate_exact_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `allocate_ui`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `allow_double_click_reset`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_drag`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_scroll`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_zoom`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `and_then`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `animate_camera`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `announce`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `any`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `Arc::clone`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Arc::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `as_array`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `as_chunks`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `as_deref`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `as_f64`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `as_mut`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `as_raw`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `as_ref`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `asin`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `aspect_ratio`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `assert!`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `assert_eq!`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `assets::load`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `assets::preferences`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `assets::select`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `at`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `atan2`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `attach_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `availability`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `available_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `available_width`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `begin_frame`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `begin_upgrade`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_attrib_location`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `bind_buffer`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_vertex_array`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `blend_func`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `BodyDirection`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `browser_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `buffer_data_u8_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `build_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_ray_layout`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `button`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `bytemuck::cast_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `camera_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `card_grid`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `catalog_and_lazy_filenames`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `center`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `centered_site_and_billboard_width`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `central_view`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `chain`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `changed`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `checkbox`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `circle_filled`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `circle_stroke`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `clamp`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `clear`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `clear_depth_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `clicked`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `clone`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `collect`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `color`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Color32::from_rgb`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `columns`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `compile_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `compile_stages`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `contains`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `content_rect`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `copied`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `corner_radius`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `cos`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `crate::sky::show`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `create_buffer`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_texture`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `create_vertex_array`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `ctx`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `cull_face`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `data::load`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `default_x_bounds`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `delete_buffer`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_program`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_shader`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_vertex_array`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delta`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `depth_func`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `depth_mask`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `desktop_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `destroy`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `detach_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `direction`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `directions`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `directions_at`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `disable`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `distance_sq`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `dragged`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `drain`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `draw_arrays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `draw_elements`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `draw_marker`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `earth_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `earthshine_phase_and_scale`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `earthshine_ratio`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `eframe::egui::CentralPanel::default`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `eframe::egui::Layout::top_down`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `eframe::egui::ScrollArea::vertical`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `eframe::egui::vec2`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `egui::Button::new`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `egui::Color32::from_black_alpha`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Color32::from_rgb`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::ComboBox::from_id_salt`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `egui::FontId::proportional`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Grid::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::Rect::from_min_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::RichText::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::Sense::click_and_drag`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Slider::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Stroke::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::vec2`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui_glow::CallbackFn::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui_glow::ShaderVersion::get`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `empty`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `enable`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `enable_vertex_attrib_array`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `end_row`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `endpoint`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `enumerate`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `eprintln!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `exp`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `expand`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `extend`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `extend_from_slice`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `f64::from`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `field::illuminated_points`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `filename`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `fill`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `filter`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `filter_map`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `find`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `fixed_decimals`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `flat_map`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `fold`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `format`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `format!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `frame`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `front_face`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `generate_mipmap`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `get_error`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `get_parameter_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `get_program_info_log`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_program_link_status`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_shader_compile_status`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_shader_info_log`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_uniform_location`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `gl`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Globe`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `globe::Globe::new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `globe::help::annotated`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `globe::help::question`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `globe::moon_fixed`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `glow::PixelUnpackData::Slice`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `heading`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `height`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `help::annotated`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `help::question`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `hline`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `HLine::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `horizon_at`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `horizontal`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `horizontal_wrapped`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `hovered`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `hypot`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `include_str!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `include_y`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `initialize`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `inner_margin`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `input_mut`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `interact_pointer_pos`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `into`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `into_iter`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_ascii_alphanumeric`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `is_empty`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `is_finite`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `is_new_shader_interface`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `is_none`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_none_or`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `is_positive`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `is_some`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_some_and`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `iter`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `join`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `json!`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `last`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `last_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `left_top`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `legend`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Legend::default`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `len`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `light_uniforms`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `lighting_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `line`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Line::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `link_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `ln`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `load_error`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `loading_status`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `local_direction`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `local_sky_frame`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `lock`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `lunar_core::stats`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `lunar_core::Stats::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `main_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `map`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `map_err`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `map_or`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `map_or_else`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `margin`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `marker`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `Marker`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `math::direction`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::earthshine_ratio`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::local_direction`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `max`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `mesh_orientation_and_seam`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `min`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `model`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `moon_fixed`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `moon_fixed_ephemeris_libration_ranges`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `MoonApp`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `Mutex::new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `native_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `native_pixels`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `new`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `next`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `num_columns`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `ok`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `ok_or`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `on_exit`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `or_else`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `orbit_input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_markers`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint_opaque`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_site_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_surface_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `painter`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `painter_at`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `panorama::Panorama::new`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `parse`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `partition_point`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Pending::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `pending_label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `perspective`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `phone_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `pixel_store_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `pixels_per_point`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Plot::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `points`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Points::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `powi`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `project`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `projection`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `projection_depth_and_aspect`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `push`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `queue_gl`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `radio_controls`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `radius`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `rect_filled`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `rem_euclid`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `Renderer::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `replace`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `request_repaint`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `ribbon`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `rotation`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `saturating_sub`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `scene_identity_empty_and_invalid`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `scene_lighting`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `scene_rays`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `segments`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `selectable_value`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `selected_text`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `Self::announce`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::begin_frame`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Self::browser_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `Self::draw_marker`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::empty`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `Self::legend`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::light_uniforms`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Self::native_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `Self::native_pixels`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `Self::process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `serde_json::from_str`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `serde_json::to_vec`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `set_margin_fraction`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `set_scene`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `set_style`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `set_theme`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `set_time`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `set_width`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `settings`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `shader_source`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `show`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `show_panorama`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `show_ui`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `show_value`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sin`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `sin_cos`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `site`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `site_camera`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `site_projection_and_south_tilt`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `SiteData::parse`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `size`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `small`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `solar_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `spacing`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `spacing_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sphere`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `split`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `sqrt`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `std::array::from_fn`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `std::env::var`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `strong`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `style_mut`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `style_of`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `suffix`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sum`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `summary_card`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `summary_cards`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `summary_uses_full_duration_and_current_clearance`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `surface_ray_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `synchronize_scene`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `take`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `tan`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `technical`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `technical_controls`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `technical_items`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `tex_image_2d`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `tex_image_2d_with_html_image`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `tex_parameter_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `text`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `texture_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `theme::apply`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `then`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `time`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `time_controls`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `to_degrees`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `to_radians`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `to_string`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `truncate`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `u_ray_color`: [annotated fragment shader](#143-globesrcmoonfrag)
- `u_ray_mode`: [annotated fragment shader](#143-globesrcmoonfrag)
- `ui`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `uniform_1_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_1_i32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_3_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `unwrap`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `unwrap_or`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `unwrap_or_else`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `update`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `update_earth_margin`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `update_sites`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `upload_browser_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `upload_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `use_program`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `utc_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `vec!`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Vec::from`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Vec::new`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Vec::with_capacity`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `version_declaration`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `vertex_attrib_pointer_f32`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `vertical`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `viewport_in_pixels`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `web_sys::console::log_1`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `width`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `window`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `window_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `windows_are_utc_intervals_not_sample_counts`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `with_layout`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `wraps_do_not_draw_chords`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_axis_formatter`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `x_axis_label`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `x_grid_spacer`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `y_axis_label`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `zip`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `zoom_delta`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)

## 14. Exact ray implementation quotations

### 14.1. globe/src/math.rs

```rust
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// Estimate reflected visible irradiance relative to the Sun using NASA geometric albedo and mean separation; C++ scalar helper.
pub(crate) fn earthshine_ratio(sun: [f32; 3], earth: [f32; 3]) -> f32
{
	// Normalize the phase-angle dot product independently of small trigonometric roundoff; C++ norm product.
	let norm =
		(sun.iter().map(|x| x * x).sum::<f32>() * earth.iter().map(|x| x * x).sum::<f32>()).sqrt();
	// Invalid or empty directions must not create spurious Earth light; C++ finite/range guard.
	if !norm.is_finite() || norm < 1e-6
	{
		// Invalid input contributes no reflected irradiance; C++ early return.
		return 0.0;
	}
	// Earth's phase is approximately the supplement of the lunar phase at the Moon; C++ acos after clamp.
	let alpha = PI
		- ((sun[0] * earth[0] + sun[1] * earth[1] + sun[2] * earth[2]) / norm)
			.clamp(-1.0, 1.0)
			.acos();
	// Integrate a Lambert sphere's illuminated visible disk, normalized to one at full Earth; C++ scalar expression.
	let phase = ((alpha.sin() + (PI - alpha) * alpha.cos()) / PI).max(0.0);
	// NASA Earth geometric albedo 0.434, mean radius 6371 km, lunar semimajor axis 384400 km; C++ inverse-square scaling.
	0.434 * (6371.0_f32 / 384400.0).powi(2) * phase
}

// Keep all CPU/GPU camera parameters in one copyable value; C++ plain value struct.
#[derive(Clone, Copy)]
pub(crate) struct View
{
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub yaw: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub tilt: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub zoom: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub aspect: f32,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub pixels: [f32; 2],
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub scale: f32,
}

// Reuse identical rigid/perspective transforms for markers and triangle ribbons; C++ member definitions.
impl View
{
	// Latitude tilt after longitude rotation; C++ matrix composition.
	pub fn model(self) -> [f32; 9]
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let (s, c) = self.yaw.sin_cos();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let (st, ct) = self.tilt.sin_cos();
		[c, st * s, -ct * s, 0.0, ct, st, s, -st * c, ct * c]
	}
	// Zoom narrows the field of view without entering the sphere; C++ matrix scale.
	pub fn projection(self) -> [f32; 16]
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut p = perspective(self.aspect);
		p[0] *= self.zoom;
		p[5] *= self.zoom;
		p
	}
	// Optionally reject sphere-occluded surface markers, but project off-sphere ray tips; C++ checked projection.
	pub fn project(self, point: [f32; 3], surface: bool) -> Option<[f32; 2]>
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let m = self.model();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let p: [f32; 3] =
			std::array::from_fn(|i| m[i] * point[0] + m[i + 3] * point[1] + m[i + 6] * point[2]);
		if (surface && 3.3 * p[2] <= 1.0) || p[2] >= 3.2
		{
			return None;
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let projection = self.projection();
		Some([
			projection[0] * p[0] / (3.3 - p[2]),
			projection[5] * p[1] / (3.3 - p[2]),
		])
	}
	// Build two camera-facing triangles and a 20-triangle endpoint disk in clip-offset coordinates; C++ vector<float>.
	pub fn ribbon(self, origin: [f32; 3], end: [f32; 3]) -> Vec<f32>
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let (Some(a), Some(b)) = (self.project(origin, false), self.project(end, false))
		else
		{
			return vec![];
		};
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dx = (b[0] - a[0]) * self.pixels[0];
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dy = (b[1] - a[1]) * self.pixels[1];
		// Bind this intermediate value for the current operation; C++ local initialization.
		let length = dx.hypot(dy).max(1e-6);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let offset = [
			-dy / length * 2.5 * self.scale / self.pixels[0],
			dx / length * 2.5 * self.scale / self.pixels[1],
		];
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut vertices = Vec::with_capacity(462);
		// Visit each value without transferring the owner's resources; C++ range-for loop.
		for (p, sign) in [
			(origin, -1.0),
			(end, -1.0),
			(end, 1.0),
			(origin, -1.0),
			(end, 1.0),
			(origin, 1.0),
		]
		{
			vertices.extend_from_slice(&[
				p[0],
				p[1],
				p[2],
				offset[0] * sign,
				offset[1] * sign,
				sign,
				0.0,
			]);
		}
		self.endpoint(&mut vertices, end);

		vertices
	}
	fn endpoint(self, vertices: &mut Vec<f32>, end: [f32; 3])
	{
		// Visit each value without transferring the owner's resources; C++ range-for loop.
		for i in 0..20
		{
			// Visit each value without transferring the owner's resources; C++ range-for loop.
			for angle in [
				None,
				Some(i as f32 * TAU / 20.0),
				Some((i + 1) as f32 * TAU / 20.0),
			]
			{
				// Bind this intermediate value for the current operation; C++ local initialization.
				let uv = angle.map_or([0.0, 0.0], |a| {
					[
						a.cos() * 4.0 * self.scale / self.pixels[0],
						a.sin() * 4.0 * self.scale / self.pixels[1],
					]
				});
				vertices.extend_from_slice(&[
					end[0],
					end[1],
					end[2],
					uv[0],
					uv[1],
					angle.map_or(0.0, f32::cos),
					angle.map_or(0.0, f32::sin),
				]);
			}
		}
	}
}

// Convert latitude/longitude in radians to a unit object-space direction; C++ returns array<float,3>.
pub(crate) fn direction(lat: f32, lon: f32) -> [f32; 3]
{
	// Return x=cos(lat)sin(lon), y=sin(lat), z=cos(lat)cos(lon); C++ returns an aggregate array.
	[lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

// Convert local north-clockwise azimuth/elevation to a Moon-fixed unit vector, all angles in radians; C++ optional<array<float,3>>.
pub(crate) fn local_direction(lat: f32, lon: f32, az: f32, el: f32) -> Option<[f32; 3]>
{
	// Invalid external angles must clear the rays rather than contaminate GPU vertices; C++ finite/range guards.
	if [lat, lon, az, el].iter().any(|v| !v.is_finite())
		|| lat.abs() > FRAC_PI_2
		|| el.abs() > FRAC_PI_2
	{
		// No valid direction exists for this input; C++ nullopt.
		return None;
	}
	// Up points away from the center in the renderer's y-north, z-zero-longitude convention; C++ radial vector.
	let up = direction(lat, lon);
	// Differentiate the radial vector with respect to latitude; C++ normalized north tangent.
	let north = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
	// Differentiate with respect to longitude and normalize analytically, including at the pole; C++ east tangent.
	let east = [lon.cos(), 0.0, -lon.sin()];
	// Resolve the local unit vector onto east, north and up; C++ component-wise linear combination.
	Some(std::array::from_fn(|i| {
		east[i] * el.cos() * az.sin() + north[i] * el.cos() * az.cos() + up[i] * el.sin()
	}))
}

/// Interleaved object position and UV, followed by counterclockwise triangle indices.
// Generate a UV sphere with duplicate seam vertices and indexed triangles; C++ returns two vectors.
pub(crate) fn sphere(slices: u32, stacks: u32) -> (Vec<f32>, Vec<u32>)
{
	// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
	assert!(slices >= 3 && stacks >= 2);
	// Own interleaved position/UV values on the CPU; C++ vector<float>.
	let mut vertices = Vec::new();
	// Own counterclockwise triangle index triples on the CPU; C++ vector<uint32_t>.
	let mut indices = Vec::new();
	// Visit every latitude ring, including both polar endpoints; C++ uses an inclusive integer for loop.
	for row in 0..=stacks
	{
		// Compute the north-to-south normalized row coordinate; C++ float division after explicit casts.
		let v = row as f32 / stacks as f32;
		// Visit every longitude endpoint, including the duplicate seam column; C++ uses an inclusive integer for loop.
		for col in 0..=slices
		{
			// Compute the west-to-east normalized column coordinate; C++ float division after explicit casts.
			let u = col as f32 / slices as f32;
			// Compute this mesh corner's unit object-space position; C++ std::array<float,3>.
			// Collapse polar rings to exact points and make both seam copies bit-identical.
			// C++ likewise must avoid tiny position differences between duplicated boundary vertices.
			let p = if row == 0
			{
				[0.0, 1.0, 0.0]
			}
			else if row == stacks
			{
				[0.0, -1.0, 0.0]
			}
			else
			{
				direction(
					FRAC_PI_2 - PI * v,
					TAU * (if col == slices { 0.0 } else { u } - 0.5),
				)
			};
			// Append position xyz and UV to one interleaved vertex; C++ vector::insert appends these five floats.
			vertices.extend_from_slice(&[p[0], p[1], p[2], u, v]);
		}
	}
	// Visit each latitude cell band, excluding the final endpoint ring; C++ uses row < stacks.
	for row in 0..stacks
	{
		// Visit each longitude cell between neighboring columns; C++ uses col < slices.
		for col in 0..slices
		{
			// Index this cell's upper-left corner; C++ uint32_t indexes the vertex array.
			let a = row * (slices + 1) + col;
			// Index the lower-left corner one complete ring later; C++ integer addition.
			let b = a + slices + 1;
			// Append two counterclockwise triangles for the current grid cell; C++ vector::insert appends six indices.
			indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
		}
	}
	// Move both vectors into the returned tuple; C++ returns a pair with move semantics.
	(vertices, indices)
}

/// Column-major, right-handed projection: 45-degree field on the shorter axis, near 0.1, far 10.
// Return a column-major right-handed projection; C++ returns array<float,16>.
pub(crate) fn perspective(aspect: f32) -> [f32; 16]
{
	// Compute cotangent of half the field of view, with portrait compensation; C++ tan from cmath.
	let f = aspect.min(1.0) / (PI / 8.0).tan();
	// Set the near clipping distance to 0.1 sphere radii; C++ const float.
	let near = 0.1;
	// Set the far clipping distance to 10 sphere radii; C++ const float.
	let far = 10.0;
	[
		f / aspect,
		0.0,
		0.0,
		0.0,
		0.0,
		f,
		0.0,
		0.0,
		0.0,
		0.0,
		(far + near) / (near - far),
		-1.0,
		0.0,
		0.0,
		2.0 * far * near / (near - far),
		0.0,
	]
}

// Compose a 30-degree south presentation tilt after geographic-north rotation; C++ returns a column-major array<float,9>.
#[cfg(test)]
pub(crate) fn rotation(angle: f32) -> [f32; 9]
{
	// Compute the animated longitude rotation; C++ separate std::sin/std::cos calls.
	let (s, c) = angle.sin_cos();
	// Negative x rotation brings the south pole toward the camera; C++ radians and trigonometric calls.
	let (st, ct) = (-PI / 6.0).sin_cos();
	// Return Rx(-30 deg)*Ry(angle), with each consecutive triple a column; C++ aggregate initialization.
	[c, st * s, -ct * s, 0.0, ct, st, s, -st * c, ct * c]
}

// Project a unit object-space site only when visible on the sphere; C++ optional<array<float,2>>.
#[cfg(test)]
pub(crate) fn marker(point: [f32; 3], angle: f32, aspect: f32) -> Option<[f32; 2]>
{
	// Do not let invalid external coordinates or an empty viewport create NaN UI positions; C++ finite-value guard.
	if !aspect.is_finite()
		|| aspect <= 0.0
		|| !angle.is_finite()
		|| point.iter().any(|p| !p.is_finite())
	{
		// Absence means no marker rather than an invented position; C++ nullopt.
		return None;
	}
	// Use exactly the shader's rigid transform; C++ array<float,9>.
	let m = rotation(angle);
	// Multiply column-major matrix by the object direction; C++ array<float,3> computed by a loop.
	let p: [f32; 3] =
		std::array::from_fn(|i| m[i] * point[0] + m[i + 3] * point[1] + m[i + 6] * point[2]);
	// A unit-sphere point faces eye (0,0,3.3) iff p dot (eye-p)>0; C++ analytic sphere occlusion test.
	if 3.3 * p[2] <= 1.0
	{
		// Hide far-side and tangent markers instead of drawing through the Moon; C++ nullopt.
		return None;
	}
	// Reuse the GPU perspective scales and camera distance; C++ projection matrix lookup.
	let projection = perspective(aspect);
	// Divide clip x/y by clip w=3.3-p.z; C++ returns an optional projected point.
	Some([
		projection[0] * p[0] / (3.3 - p[2]),
		projection[5] * p[1] / (3.3 - p[2]),
	])
}

// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
#[cfg(test)]
// Group tests in a nested module; C++ puts test helpers in a namespace.
mod tests
{
	// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
	use super::*;

	// Check full, new, quarter Earth and malformed vectors independently of shader display exposure; C++ test registration.
	#[test]
	fn earthshine_phase_and_scale()
	{
		// Opposite Sun/Earth directions at the Moon imply full Earth; C++ physical reference value.
		let full = earthshine_ratio([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]);
		// Full Earth is about one part in 8388 of sunlight; C++ near-equality assertion.
		assert!((full - 0.000119214).abs() < 1e-8);
		// Coincident Sun and Earth directions imply new Earth; C++ zero assertion.
		assert_eq!(earthshine_ratio([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]), 0.0);
		// Quarter phase is 1/pi, not one half, for a Lambert sphere; C++ ratio assertion.
		assert!(
			(earthshine_ratio([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]) / full - 1.0 / PI).abs() < 1e-6
		);
		// Empty direction vectors must not create light; C++ invalid-input assertion.
		assert_eq!(earthshine_ratio([0.0; 3], [0.0; 3]), 0.0);
		// Nonfinite ephemerides must not enter shader uniforms; C++ finite-input assertion.
		assert_eq!(earthshine_ratio([f32::NAN; 3], [1.0; 3]), 0.0);
	}

	// Camera centering and billboards must agree with the GPU at high zoom; C++ regression registration.
	#[test]
	fn centered_site_and_billboard_width()
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let lat = (-89.4398_f32).to_radians();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let lon = 222.83_f32.to_radians();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let view = View {
			yaw: -lon,
			tilt: lat,
			zoom: 128.0,
			aspect: 1.0,
			pixels: [800.0, 800.0],
			scale: 2.0,
		};
		// Bind this intermediate value for the current operation; C++ local initialization.
		let point = direction(lat, lon);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let center = view.project(point, true).unwrap();
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(center[0].abs() < 1e-4 && center[1].abs() < 1e-4);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let vertices = view.ribbon(point, [point[0] + 0.6, point[1], point[2]]);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(vertices.len(), 462);
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(vertices.iter().all(|v| v.is_finite()));
		// Bind this intermediate value for the current operation; C++ local initialization.
		let width = (vertices[3].powi(2) + vertices[4].powi(2)).sqrt() * 800.0;
		// Enforce the independent regression expectation; C++ test assertion.
		assert!((width - 5.0).abs() < 1e-4);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let aligned = view.ribbon(point, point);
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(aligned.iter().all(|v| v.is_finite()));
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(view.project([0.0, 0.0, -4.0], true).is_none());
		// Reject points behind the camera instead of emitting invalid ribbon vertices; C++ empty-vector error case.
		assert!(view.ribbon(point.map(|v| v * 3.3), point).is_empty());
		// A camera-aligned direction retains its finite endpoint disk; C++ vertex-layout assertion.
		assert_eq!(aligned.len(), 462);
		// The first outer disk vertex is 2 logical points from its center; C++ screen-space radius check.
		assert!((aligned[52].hypot(aligned[53]) * 400.0 - 4.0).abs() < 1e-4);
	}

	// Register an independent local-frame regression; C++ test registration.
	#[test]
	fn local_sky_frame()
	{
		// Exercise the actual selected south-pole site; C++ scalar setup.
		let lat = (-89.4398_f32).to_radians();
		// Longitude remains east-positive despite the renderer's unusual axes; C++ scalar setup.
		let lon = 222.83_f32.to_radians();
		// Straight up must equal the geographic radial vector; C++ optional extraction.
		let up = local_direction(lat, lon, 0.0, FRAC_PI_2).unwrap();
		// Geographic north is the derivative of the radial vector with latitude; C++ expected components.
		let north = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
		// Check both basis directions component by component; C++ counted loop.
		for i in 0..3
		{
			// Single-precision trigonometry warrants a small absolute tolerance; C++ numerical assertion.
			assert!((up[i] - direction(lat, lon)[i]).abs() < 1e-6);
			// Zero azimuth/elevation must point north, not along longitude or toward the center; C++ numerical assertion.
			assert!((local_direction(lat, lon, 0.0, 0.0).unwrap()[i] - north[i]).abs() < 1e-6);
		}
		// A nontrivial closed-form example detects swapped east/north or renderer axes; C++ fixture vector.
		let expected = [0.41602117, 0.7803301, -0.4669168];
		// At latitude 30, longitude 60, azimuth 45, elevation 30 degrees; C++ direction calculation.
		let actual = local_direction(PI / 6.0, PI / 3.0, PI / 4.0, PI / 6.0).unwrap();
		// All components must agree independently of the pole fixture; C++ counted assertions.
		for i in 0..3
		{
			// Enforce the independent regression expectation; C++ test assertion.
			assert!((actual[i] - expected[i]).abs() < 1e-6);
		}
		// Invalid data must not become a GPU coordinate; C++ error-case assertion.
		assert!(local_direction(f32::NAN, 0.0, 0.0, 0.0).is_none());
		// Elevation beyond the physical sky range is rejected; C++ range-error assertion.
		assert!(local_direction(0.0, 0.0, 0.0, PI).is_none());
	}

	// Register a marker projection regression; C++ testing macro.
	#[test]
	// Verify orientation, occlusion, all rotation phases and invalid input; C++ unit test.
	fn site_projection_and_south_tilt()
	{
		// A near-side equatorial point must be visible; C++ assert optional engagement.
		assert!(marker(direction(0.0, 0.0), 0.0, 1.0).is_some());
		// Its antipode must be hidden; C++ assert disengagement.
		assert!(marker(direction(0.0, PI), 0.0, 1.0).is_none());
		// No marker should be emitted for invalid projection inputs; C++ finite-value error cases.
		assert!(marker([f32::NAN, 0.0, 1.0], 0.0, 1.0).is_none());
		// Zero width cannot be projected; C++ assert error/absence.
		assert!(marker(direction(0.0, 0.0), 0.0, 0.0).is_none());
		// Check the actual example site throughout a complete turn; C++ counted loop.
		for deg in 0..360
		{
			// Convert geographic degrees to the same unit direction used by set_site; C++ trig helper call.
			let point = direction((-89.45_f32).to_radians(), 222.69_f32.to_radians());
			// South tilt must expose this near-pole site at every phase; C++ checked optional extraction.
			let projected = marker(point, (deg as f32).to_radians(), 0.6).unwrap();
			// The marker stays in the lower hemisphere and inside the viewport; C++ tolerance bounds.
			assert!(projected[0].abs() < 1.0 && (-1.0..0.0).contains(&projected[1]));
		}
		// Guard the shared shader contract, so geometry and normals cannot silently diverge; C++ source contract test.
		assert!(include_str!("moon.vert").contains("uniform mat3 u_model;"));
		// Both stages must use the same matrix; C++ source contract assertion.
		assert!(include_str!("moon.frag").contains("uniform mat3 u_model;"));
		// WebGL rejects shared uniforms whose integer precisions differ between stages; C++ shader-contract assertion.
		for shader in [include_str!("moon.vert"), include_str!("moon.frag")]
		{
			// Require the explicit matching qualifier instead of different stage defaults; C++ substring assertion.
			assert!(shader.contains("uniform highp int u_ray_mode;"));
		}
	}

	// Register the following function with Rust's test runner; C++ needs a test framework registration.
	#[test]
	// Check the production mesh's sizes, seam, orientation and winding; C++ would use test assertions.
	fn mesh_orientation_and_seam()
	{
		// Generate the production mesh for regression checks; C++ structured binding receives two vectors.
		let (v, i) = sphere(256, 128);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(v.len(), 257 * 129 * 5);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(i.len(), 256 * 128 * 6);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(i.iter().all(|&index| (index as usize) < v.len() / 5));
		// Check every duplicated seam row, including the polar rows; C++ loops over rows.
		for row in 0..=128
		{
			// Check xyz separately so every seam component is bit-identical; C++ loops over array indices.
			for axis in 0..3
			{
				// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
				assert_eq!(v[row * 257 * 5 + axis], v[(row * 257 + 256) * 5 + axis]);
			}
		}
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(direction(0.0, 0.0)[2] > 0.99);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(direction(0.0, FRAC_PI_2)[0] > 0.99);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(direction(-0.75, -0.2)[1] < 0.0);
		// A middle-row triangle must face outwards, towards the camera at longitude zero.
		// Select the first cell vertex or a borrowed triangle corner; C++ integer or span.
		let a = &v[(63 * 257 + 128) * 5..];
		// Select the next-row vertex or a borrowed triangle corner; C++ integer or span.
		let b = &v[(64 * 257 + 128) * 5..];
		// Select the third triangle corner for the winding check; C++ span.
		let c = &v[(63 * 257 + 129) * 5..];
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
	}

	// Register the following function with Rust's test runner; C++ needs a test framework registration.
	#[test]
	// Check portrait scaling and near/far depth endpoints; C++ would use test assertions.
	fn projection_depth_and_aspect()
	{
		// Construct the landscape matrix for endpoint checks; C++ std::array<float,16>.
		let p = perspective(2.0);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!((p[5] / p[0] - 2.0).abs() < 1e-6);
		// Build the portrait projection to check that the short-axis field stays fixed; C++ array<float,16>.
		let portrait = perspective(0.5);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!((portrait[0] - p[5]).abs() < 1e-6);
		// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
		for (z, expected) in [(-0.1, -1.0), (-10.0, 1.0)]
		{
			// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
			assert!(((p[10] * z + p[14]) / -z - expected).abs() < 1e-5);
		}
	}
}
```

### 14.2. globe/src/lib.rs

```rust
//! An illustrative lunar renderer, independent of the landing-site scientific library.
//! See docs/03-moon.md for coordinate, photometric, color, and GL lifetime contracts.

// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod assets;
mod camera;
mod renderer;
mod view;
use renderer::Renderer;
mod field;
// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod math;
// Share accessible explanatory widgets without introducing an app dependency; C++ public utility namespace.
pub mod help;

// Bound enlargement by the available polar texture detail, not GPU capacity; C++ constexpr float.
const MAX_ZOOM: f32 = 8.0;

// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui::mutex::Mutex;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui_glow::glow::{self, HasContext as _};
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::sync::Arc;

// Copy a local sky direction across the app/renderer boundary without a core dependency; C++ plain value struct.
#[derive(Clone, Copy, Debug)]
pub struct BodyDirection
{
	// Azimuth is measured clockwise from north, in degrees; C++ float.
	pub az_deg: f32,
	// Elevation is measured above the site's horizontal plane, in degrees; C++ float.
	pub el_deg: f32,
	// The app supplies its terrain/clearance availability predicate; C++ bool.
	pub visible: bool,
}

// Convert a topocentric sky angle to the renderer's Moon-fixed unit vector; C++ optional<array<float,3>>.
pub fn moon_fixed(lat_deg: f64, lon_deg: f64, body: BodyDirection) -> Option<[f32; 3]>
{
	// Reuse the tested east/north/up basis independently of local visibility; C++ checked coordinate transform.
	math::local_direction(
		(lat_deg as f32).to_radians(),
		(lon_deg as f32).to_radians(),
		body.az_deg.to_radians(),
		body.el_deg.to_radians(),
	)
}

// Snapshot real directions and explicitly optional display aids for one deferred draw; C++ value aggregate.
#[derive(Clone, Copy)]
struct Lighting
{
	// Store the selected epoch's incoming solar direction; C++ fixed-size vector.
	sun: [f32; 3],
	// Store the selected epoch's incoming Earth direction; C++ fixed-size vector.
	earth: [f32; 3],
	// Express visible reflected irradiance relative to sunlight; C++ float.
	ratio: f32,
	// Keep the optional Earth-only exposure multiplication explicit; C++ bool.
	boosted: bool,
	// Keep artificial far-side light independent of physical energy; C++ bool.
	fill: bool,
}

// Keep transformed directions distinct from local input angles; C++ internal plain value struct.
#[derive(Clone, Copy)]
struct Ray
{
	// Moon-fixed unit direction before presentation rotation; C++ array<float,3>.
	direction: [f32; 3],
	// False selects the dim material, independently of depth occlusion; C++ bool.
	visible: bool,
}

// Lightweight catalog metadata plus optional current scientific sample; C++ value struct.
#[derive(Clone)]
pub struct Marker
{
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub id: usize,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub name: String,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub lat_deg: f64,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub lon_deg: f64,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub directions: Option<[BodyDirection; 2]>,
}

/// Owns the callback's shared renderer. Call destroy from eframe's on_exit with a current context.
// Declare named fields; Rust methods live in a separate impl, unlike inline C++ class definitions.
pub struct Globe
{
	// Shared optional GPU state, absent while loading; C++ shared_ptr<mutex-protected optional<Renderer>>.
	renderer: Arc<Mutex<Option<Renderer>>>,
	// Shared CPU-side completion state; C++ shared_ptr to a mutex-protected Loading.
	loading: Arc<Mutex<assets::Loading>>,
	// Selected size label 2/4/8, with width equal to 1,024 times the label; C++ uint32_t.
	tier: u32,
	// Requested final detail is independent of the first visible 2k pair; C++ unsigned members.
	target_tier: u32,
	max_tier: u32,
	// Optional illustrative parallel Sun rays; C++ bool.
	surface_rays: bool,
	// Interactive presentation state is independent of ephemeris time; C++ value members.
	scene: Vec<Marker>,
	selected: usize,
	yaw: f32,
	tilt: f32,
	zoom: f32,
	paused: bool,
	// Track the current ephemeris Earth line of sight instead of spinning; C++ bool.
	earth_view: bool,
	// Multiply only Earthshine for display when explicitly requested; C++ bool.
	boosted: bool,
	// Add an explicitly artificial far-side fill only on request; C++ bool.
	fill: bool,
	last_time: Option<f64>,
	focus: Option<(f64, [f32; 3], [f32; 3])>,
}

// Exercise scene validation without opening an OpenGL context; C++ isolated test namespace.
#[cfg(test)]
mod scene_tests
{
	// Reuse the production state and aggregate definitions; C++ qualified test access.
	use super::*;

	// Verify invalid entries cannot shift stable identifiers or retain a stale transition; C++ test registration.
	#[test]
	fn scene_identity_empty_and_invalid()
	{
		// Supply uninitialized GPU state so no renderer or network is needed; C++ aggregate test fixture.
		let mut globe = Globe {
			renderer: Arc::new(Mutex::new(None)),
			loading: Arc::new(Mutex::new(assets::Loading {
				images: [None, None],
				error: None,
			})),
			tier: 2,
			target_tier: 2,
			max_tier: 8,
			surface_rays: false,
			scene: vec![],
			selected: 0,
			yaw: 0.0,
			tilt: 0.0,
			zoom: 1.0,
			paused: true,
			earth_view: true,
			boosted: false,
			fill: false,
			last_time: None,
			focus: Some((0.0, [0.0; 3], [1.0; 3])),
		};
		// Keep the valid identifier deliberately different from its vector index; C++ aggregate fixture.
		let valid = Marker {
			id: 17,
			name: "Published".into(),
			lat_deg: -89.0,
			lon_deg: 220.0,
			directions: None,
		};
		// Represent malformed external metadata without corrupting the valid entry; C++ aggregate copy override.
		let invalid = Marker {
			id: 8,
			lat_deg: f64::NAN,
			..valid.clone()
		};
		// Apply the same filtering path used on every application redraw; C++ scene setter.
		globe.set_scene(vec![invalid, valid], 17);
		// Filtering must not renumber the published marker identifier; C++ collection assertion.
		assert_eq!(
			globe.scene.iter().map(|s| s.id).collect::<Vec<_>>(),
			vec![17]
		);
		// A changed selection cancels the previous site's camera animation; C++ optional-state assertion.
		assert!(globe.focus.is_none());
		// Empty data clears markers and rays instead of retaining old state; C++ empty-container setter.
		globe.set_scene(vec![], 17);
		// No stale scientific direction survives scene replacement; C++ collection assertion.
		assert!(globe.scene.is_empty());
	}
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Globe
{
	// Construct and validate owned state, returning an error instead of throwing; C++ uses a factory returning expected.
	pub fn new(gl: &Arc<glow::Context>) -> Result<Self, String>
	{
		// Record whether the browser looks mobile; C++ stores the corresponding bool.
		let (mobile, forced) = assets::preferences();
		// SAFETY: eframe makes this context current before calling the constructor.
		// Clamp the requested tier to the current GPU's maximum width; C++ passes a const reference to preferences.
		let tier = assets::select(
			// Read an integer capability into Rust; C++ glGetIntegerv writes through an output pointer.
			// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
			unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
			mobile,
			forced.as_deref(),
		)?;
		// Format a visible diagnostic naming the chosen tier; C++ std::format returns a string.
		let status = format!("Moon tier {tier}k (mobile={mobile})");
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		eprintln!("{status}");
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		web_sys::console::log_1(&status.into());
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(Self {
			// Shared optional GPU state, absent while loading; C++ shared_ptr<mutex-protected optional<Renderer>>.
			renderer: Arc::new(Mutex::new(None)),
			// Shared CPU-side completion state; C++ shared_ptr to a mutex-protected Loading.
			surface_rays: false,
			loading: assets::load(2, None),
			tier: 2,
			target_tier: tier,
			max_tier: assets::select(
				unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
				false,
				Some("8"),
			)?,
			// Default to the current Earth line of sight, without artificial spin or tilt; C++ scalar initialization.
			scene: Vec::new(),
			selected: 0,
			yaw: 0.0,
			tilt: 0.0,
			zoom: 1.0,
			paused: true,
			earth_view: true,
			boosted: false,
			fill: false,
			last_time: None,
			focus: None,
		})
	}

	// Copy metadata for every marker and scientific directions only for requested sites; C++ scene value transfer.
	pub fn set_scene(&mut self, scene: Vec<Marker>, selected: usize)
	{
		// Cancel an in-flight transition if another site is chosen; C++ identity guard.
		if self.selected != selected
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.focus = None;
		}
		// Update this persistent member explicitly; C++ member assignment.
		self.scene = scene
			.into_iter()
			.filter(|m| {
				m.lat_deg.is_finite()
					&& m.lon_deg.is_finite()
					&& (-90.0..=90.0).contains(&m.lat_deg)
			})
			.collect();
		// Update this persistent member explicitly; C++ member assignment.
		self.selected = selected;
	}

	// Release registered GPU objects using the current context; C++ explicit teardown avoids a late invalid-context destructor.
	pub fn destroy(&mut self, gl: &glow::Context)
	{
		// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
		if let Some(mut renderer) = self.renderer.lock().take()
		{
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			renderer.destroy(gl);
		}
	}
}
```

### 14.3. globe/src/moon.frag

```glsl
// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Pass world-space surface position to the fragment stage with perspective-correct interpolation.
in vec3 v_position;
// Pass unrotated object position so texture coordinates remain attached to the Moon.
in vec3 v_object;
// Bind the selected tier's SRGB8_ALPHA8 color texture through its texture-unit index.
uniform sampler2D u_albedo;
// Bind the selected tier's linear RGB8 object-space normal texture.
uniform sampler2D u_normal;
// Receive the same rigid model matrix as the vertex shader and CPU marker; C++ glUniformMatrix3fv.
uniform mat3 u_model;
// Write opaque sRGB display color to egui's framebuffer.
out vec4 out_color;
// A uniform branch reuses the exact globe vertex transform for unlit direction lines; C++ integer material flag.
uniform highp int u_ray_mode;
// Interpolated signed-distance coordinates, independent of clip-space offsets.
in vec2 v_coverage;
// Direction-line colors are display colors, independent of illustrative illumination; C++ RGB uniform.
uniform vec3 u_ray_color;
// Receive the actual Moon-fixed Sun direction; C++ glUniform3f.
uniform vec3 u_sun;
// Receive the actual Moon-fixed Earth direction; C++ glUniform3f.
uniform vec3 u_earth;
// Disable physical sunlight when ephemerides are unavailable; C++ glUniform1f.
uniform float u_sun_scale;
// Scale reflected Earth light relative to solar irradiance; C++ glUniform1f.
uniform float u_earthshine;
// Enable a nonphysical visual aid independently of physical energy; C++ glUniform1f.
uniform float u_far_fill;
// Define pi once so all angular expressions use radians consistently.
const float PI = 3.141592653589793;
// Project mapped normals into the view-facing half-space before renormalizing; dimensionless cosine.
const float MIN_EMISSION_COSINE = 0.01;
// Display exposure is eight times relative linear radiance, one stop above the previous value of four.
const float EXPOSURE = 8.0;
// Area-weighted 566 nm WAC map means, not a new fit. See assets/photometry.py.
// ISIS3 fadd004da2f733525209ee4f66ffdb3a9a9d193e:
// isis/src/lro/apps/lrowacphomap/HapkeLROC.cpp, HGfunc and photometry.
// Set the area-weighted single-scattering albedo: the fraction of energy one grain scattering retains.
const float W = 0.3261142672;
// Set the HG lobe shape coefficient from the 566 nm parameter map, not the opposition amplitude.
const float B = 0.2348928321;
// Set the signed HG mixing coefficient; positive C favors backward scattering with weight (1+C)/2.
const float C = 0.3574639778;
// Set the amplitude of shadow-hiding opposition brightening from the same map.
const float BS0 = 1.7707997700;
// Set its dimensionless tan(phase/2) width parameter, not a width measured directly in degrees.
const float HS = 0.0622193166;
// Convert the map's mean unresolved roughness angle from degrees to radians.
const float THETA = 23.65660095 * PI / 180.0;

// Approximate Chandrasekhar multiple scattering with the logarithmic Hapke/ISIS expression.
float h_function(float x)
{
	// Use the continuous H(0)=1 limit to numerical tolerance without evaluating log(1/0).
	x = max(x, 0.000001);
	// Compute the square root of the absorbed fraction used by the H-function approximation.
	float gamma = sqrt(1.0 - W);
	// Compute Hapke's diffuse reflectance parameter from single-scattering albedo.
	float r0 = (1.0 - gamma) / (1.0 + gamma);
	// Evaluate the logarithmic H approximation as a reciprocal, not the simpler rational approximation.
	return 1.0 / (1.0 - W * x * (r0 + 0.5 * (1.0 - 2.0 * r0 * x) * log((1.0 + x) / x)));
}

// Return Hapke I/F, a dimensionless radiance factor rather than a conventional BRDF per steradian.
float hapke(float mu0, float mu, float cosg)
{
	// Return black for a locally unlit or invisible surface instead of dividing at the terminator.
	if (mu0 <= 0.0 || mu <= 0.0)
	{
		// Emit zero reflected sunlight; no ambient or earthshine is added.
		return 0.0;
	}
	// Convert the clamped light-view dot product into phase angle in radians.
	float g = acos(clamp(cosg, -1.0, 1.0));
	// Reuse the squared HG shape parameter in both lobe denominators.
	float b2 = B * B;
	// Weight the backward lobe by (1+C)/2; the continuation adds the forward lobe.
	float phase = (1.0 + C) * 0.5 * (1.0 - b2) / pow(1.0 - 2.0 * B * cosg + b2, 1.5)
		// Add the forward HG lobe with weight (1-C)/2 and the opposite sign in its denominator.
		+ (1.0 - C) * 0.5 * (1.0 - b2) / pow(1.0 + 2.0 * B * cosg + b2, 1.5);
	// Compute the shadow-hiding profile, which equals one exactly at opposition.
	float shoe = 1.0 / (1.0 + tan(g * 0.5) / HS);
	// Published Bc0=0 and phi=0 imply no CBOE multiplier and K=1.
	// Compute tangent of the unresolved mean surface tilt for the roughness correction.
	float t = tan(THETA);
	// Compute the roughness normalization 1/sqrt(1+pi*tan(theta)^2).
	float chi = inversesqrt(1.0 + PI * t * t);
	// Recover sine of incidence from its cosine, clamping roundoff before sqrt.
	float sini = sqrt(max(0.0, 1.0 - mu0 * mu0));
	// Recover sine of emission from its cosine, clamping roundoff before sqrt.
	float sine = sqrt(max(0.0, 1.0 - mu * mu));
	// Compute cotangent of incidence with a finite normal-incidence limit.
	float coti = mu0 / max(sini, 0.000001);
	// Compute cotangent of emission with a finite normal-emission limit.
	float cote = mu / max(sine, 0.000001);
	// Evaluate the first roughness exponential at incidence.
	float e1i = exp(-2.0 * coti / (PI * t));
	// Evaluate the second roughness exponential at incidence.
	float e2i = exp(-coti * coti / (PI * t * t));
	// Evaluate the first roughness exponential at emission.
	float e1e = exp(-2.0 * cote / (PI * t));
	// Evaluate the second roughness exponential at emission.
	float e2e = exp(-cote * cote / (PI * t * t));
	// Compute effective incidence cosine for zero relative azimuth.
	float etai = chi * (mu0 + sini * t * e2i / (2.0 - e1i));
	// Compute effective emission cosine for zero relative azimuth.
	float etae = chi * (mu + sine * t * e2e / (2.0 - e1e));
	// Recover the azimuth between incidence and emission planes; use zero azimuth at the normal singularity.
	float cospsi = sini * sine < 0.000001 ? 1.0 : clamp((cosg - mu * mu0) / (sini * sine), -1.0, 1.0);
	// Convert that bounded cosine to a relative azimuth in radians.
	float psi = acos(cospsi);
	// Compute the azimuth-dependent overlap profile, taking its exact zero limit at pi.
	float fpsi = psi >= PI - 0.000001 ? 0.0 : exp(-2.0 * tan(psi * 0.5));
	// Use the half-angle identity to obtain sin(psi/2)^2 without another trigonometric call.
	float sin2 = (1.0 - cospsi) * 0.5;
	// Hold the roughness-adjusted incidence cosine, assigned in both geometric branches.
	float mu0e;
	// Hold the roughness-adjusted emission cosine, assigned in both geometric branches.
	float mue;
	// Hold the branch-dependent ratio used to normalize roughness shadowing.
	float q;
	// Select the i<=e roughness branch because acos is decreasing on the valid cosine interval.
	if (mu0 >= mu)
	{
		// Compute the current incidence/emission branch's common roughness denominator.
		float denominator = 2.0 - e1e - psi / PI * e1i;
		// Assign the branch's adjusted incidence cosine from its roughness exponentials.
		mu0e = chi * (mu0 + sini * t * (cospsi * e2e + sin2 * e2i) / denominator);
		// Assign the branch's adjusted emission cosine from its roughness exponentials.
		mue = chi * (mu + sine * t * (e2e - sin2 * e2i) / denominator);
		// Assign the matching incidence or emission normalization ratio for shadowing.
		q = mu0 / etai;
	}
	// Use the complementary i>e roughness branch, exchanging the appropriate incidence/emission roles.
	else
	{
		// Compute the current incidence/emission branch's common roughness denominator.
		float denominator = 2.0 - e1i - psi / PI * e1e;
		// Assign the branch's adjusted incidence cosine from its roughness exponentials.
		mu0e = chi * (mu0 + sini * t * (e2i - sin2 * e2e) / denominator);
		// Assign the branch's adjusted emission cosine from its roughness exponentials.
		mue = chi * (mu + sine * t * (cospsi * e2i + sin2 * e2e) / denominator);
		// Assign the matching incidence or emission normalization ratio for shadowing.
		q = mu / etae;
	}
	// Assemble the unresolved shadowing factor; guard the grazing-angle denominator against zero.
	float shadow = mue * mu0 * chi / max(etae * etai * (1.0 - fpsi + fpsi * chi * q), 0.000001);
	// Add isotropic multiple scattering as the product of two H-functions minus one.
	float multiple = h_function(mu0e) * h_function(mue) - 1.0;
	// Multiply single scattering, opposition, multiple scattering, and roughness into I/F; K=1 and Bc0=0.
	return W * 0.25 * mu0e / max(mu0e + mue, 0.000001) * (phase * (1.0 + BS0 * shoe) + multiple) * shadow;
}

// Encode linear display intensities with the standard piecewise sRGB transfer function.
vec3 linear_to_srgb(vec3 x)
{
	// Select the low-intensity linear branch or the power-law branch independently for each color channel.
	return mix(x * 12.92, 1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), x));
}

// Evaluate one directional light without night-side normal-map leakage; C++ float-valued helper.
float light_response(vec3 n, vec3 geometric, vec3 view, vec3 light, float normal_strength)
{
	// Local terrain normals cannot substitute for resolved shadows; C++ geometric visibility guard.
	if (dot(geometric, light) <= 0.0)
	{
		// Suppress this source on its geometric night side; C++ early return.
		return 0.0;
	}
	// Evaluate the mapped response with the shared photometric model; C++ float initialization.
	float mapped_reflectance = hapke(dot(n, light), dot(n, view), dot(light, view));
	// Evaluate the smooth-sphere response independently of texture normals; C++ float initialization.
	float geometric_reflectance = hapke(dot(geometric, light), dot(geometric, view), dot(light, view));
	// Fade unresolved relief at grazing emission angles; C++ linear-interpolation return.
	return mix(geometric_reflectance, mapped_reflectance, normal_strength);
}

// Run once per vertex or fragment respectively; GLSL entry points have no C++ object ownership.
void main()
{
	// All fragments in this draw take the same branch, before texture derivatives or Hapke evaluation; C++ uniform material switch.
	if (u_ray_mode != 0)
	{
		// Preserve opaque diagram colors; dimming is encoded in RGB, not photographic lighting.
		out_color = vec4(u_ray_color, 1.0 - smoothstep(0.15, 1.0, length(v_coverage)));
		// Rays require no albedo, normals or photometric calculations; C++ early return.
		return;
	}
	// Normalize the interpolated object-space direction before reconstructing longitude and latitude.
	vec3 p = normalize(v_object);
	// Map atan longitude and acos latitude into normalized equirectangular coordinates.
	vec2 uv = vec2(atan(p.x, p.z) / (2.0 * PI) + 0.5, acos(clamp(p.y, -1.0, 1.0)) / PI);
	// Differentiate the continuous direction, not the wrapped atan output.
	// Differentiate the continuous direction across adjacent screen fragments horizontally.
	vec3 dx = dFdx(p);
	// Differentiate the continuous direction across adjacent screen fragments vertically.
	vec3 dy = dFdy(p);
	// Bound the squared horizontal radius so analytic UV derivatives remain finite at poles.
	float horizontal = max(dot(p.xz, p.xz), 0.000001);
	// Apply the analytic longitude/latitude derivative to horizontal screen variation, avoiding the wrap jump.
	vec2 gx = vec2((p.z * dx.x - p.x * dx.z) / (2.0 * PI * horizontal), -dx.y / (PI * sqrt(horizontal)));
	// Apply the same continuous derivative to vertical screen variation for texture footprint estimation.
	vec2 gy = vec2((p.z * dy.x - p.x * dy.z) / (2.0 * PI * horizontal), -dy.y / (PI * sqrt(horizontal)));
	// Decode the RGB8 object-space normal and normalize after filtering.
	// Texture state retains trilinear mipmaps but not the anisotropic path that produced dotted arcs in fixed-pose tests.
	vec3 n = normalize(textureGrad(u_normal, uv, gx, gy).rgb * 2.0 - 1.0);
	// Apply the camera's rigid rotation to the normal, preserving lighting-space consistency.
	n = normalize(u_model * n);
	// Transform real Moon-fixed lights with the same rigid matrix as the Moon and markers.
	vec3 light = normalize(u_model * u_sun);
	// Keep the Earth source in the same world frame; C++ matrix-vector product and normalization.
	vec3 earth_light = normalize(u_model * u_earth);
	// Point from the surface fragment to the perspective camera at z=3.3.
	vec3 view = normalize(vec3(0.0, 0.0, 3.3) - v_position);
	// Fade unresolved relief in the foreshortened limb band; use the true sphere normal below cosine 0.35.
	vec3 geometric = normalize(v_position);
	// Retain full relief above cosine 0.65, with a continuous smoothstep transition between both angles.
	float normal_strength = smoothstep(0.35, 0.65, max(dot(geometric, view), 0.0));
	// Suppress unresolved equirectangular polar detail, rather than displaying longitude streaks as terrain.
	float polar_fallback = smoothstep(cos(15.0 * PI / 180.0), cos(10.0 * PI / 180.0), abs(p.y));
	// Within ten degrees of either pole the overview deliberately uses a schematic sphere normal.
	normal_strength *= 1.0 - polar_fallback;
	// A normal map cannot change silhouette visibility: back-facing mapped facets must not cut black holes.
	// Add only the missing view component, then restore unit length; leave already front-facing normals alone.
	n = normalize(n + max(0.0, MIN_EMISSION_COSINE - dot(n, view)) * view);
	// Add physically distinct direct solar and phase-scaled reflected Earth contributions.
	float reflectance = u_sun_scale * light_response(n, geometric, view, light, normal_strength)
		+ u_earthshine * light_response(n, geometric, view, earth_light, normal_strength);
	// SRGB8_ALPHA8 texture sampling already decodes color and filters in linear space.
	// Sample SRGB8_ALPHA8 color with explicit seam-safe derivatives; the GPU returns linear color.
	vec3 albedo = textureGrad(u_albedo, uv, gx, gy).rgb;
	// Use the documented mean reflectance in the unresolved cap; this is not a new terrain or color measurement.
	albedo = mix(albedo, vec3(0.4832), polar_fallback);
	// Solid-angle-weighted mean linear luminance; this is not a per-pixel inversion.
	// Modulate the averaged photometric response by relative spatial color and apply exposure.
	vec3 radiance = EXPOSURE * (albedo / 0.4832) * reflectance;
	// Only longitudes beyond +/-90 degrees from mean Earth longitude zero receive this visual aid.
	float far_side = smoothstep(0.0, 0.15, -p.z / max(length(p.xz), 0.000001));
	// This soft gray-blue term is deliberately not labeled physical illumination.
	radiance += u_far_fill * far_side * vec3(0.035, 0.045, 0.065) * (albedo / 0.4832);
	// Compress positive linear radiance with the Reinhard operator before display encoding.
	vec3 mapped = radiance / (vec3(1.0) + radiance);
	// Encode once to sRGB and set alpha to one; egui_glow disables framebuffer sRGB encoding.
	out_color = vec4(linear_to_srgb(mapped), 1.0);
}
```

## 15. Surface field source

### 15.1. globe/src/field.rs

```rust
// Sample equal-area Fibonacci directions on the sunlit hemisphere; C++ vector<array<float,3>>.
pub(crate) fn illuminated_points(sun: [f32; 3]) -> Vec<[f32; 3]>
{
	// Reject absent/invalid directions instead of inventing a light; C++ finite-vector guard.
	if sun.iter().any(|v| !v.is_finite()) || sun.iter().map(|v| v * v).sum::<f32>() < 1e-12
	{
		return Vec::new();
	}
	// Half-offset latitudes avoid singular pole samples; C++ indexed loop with a golden-angle longitude.
	(0..512)
		.filter_map(|i| {
			let y = 1.0 - 2.0 * (i as f32 + 0.5) / 512.0;
			let r = (1.0 - y * y).sqrt();
			let a = i as f32 * 2.3999631;
			let p = [r * a.cos(), y, r * a.sin()];
			(p.iter().zip(sun).map(|(p, s)| p * s).sum::<f32>() > 0.0).then_some(p)
		})
		.collect()
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn field_is_finite_even_and_sunlit()
	{
		for sun in [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]
		{
			let points = illuminated_points(sun);
			assert!((250..=262).contains(&points.len()));
			for p in points
			{
				assert!((p.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-5);
				assert!(p.iter().zip(sun).map(|(v, s)| v * s).sum::<f32>() > 0.0);
			}
		}
		assert!(illuminated_points([0.0; 3]).is_empty());
		assert!(illuminated_points([f32::NAN, 0.0, 0.0]).is_empty());
	}
}
```
