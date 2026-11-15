# Sun and Earth directions on the rotating Moon

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

The app passes site coordinates first, then the selected directions, on every redraw. Globe::set_site establishes the local basis and clears old rays; set_directions converts the supplied angles with math::local_direction. The deferred GL callback captures copies of the two directions and origin together, so one draw uses one coherent sample. Slider edits appear on the following animation redraw, as do the existing top statuses. The globe's one-minute presentation rotation does not change the selected scientific epoch.

The integration regression uses unequal-interval data with Sun elevation -1 then +1 degrees and Earth at +0.75 degrees. It edits the sample index and the radio clearance, verifies the exported values/flags, then removes the loaded data and verifies absence. This checks the bridge, not merely the stand-alone trigonometry.

## 11. Drawing real object-space lines

The marker is at U on the illustrative unit sphere. Each line begins at that exact same point and ends at:

$$\mathbf q=\mathbf U+0.6\mathbf d.$$

The length is 0.6 lunar radii for both bodies. It represents direction only, not the Sun/Earth distances, body size or a measured mast height. The overlay explicitly says length is not to scale. Optional endpoint spheres are not implemented; they are unnecessary for the direction and would add a second misleading size scale.

The shader transforms both endpoints using the same uniforms already used for the Moon:

$$\mathbf p_{clip}=P\left[\begin{array}{c}M\mathbf p-(0,0,3.3)\\1\end{array}\right],\qquad M=R_x(-30\text{ degrees})R_y(\alpha).$$

P is the existing perspective matrix, M the shared rigid rotation, and alpha the animation angle. This is actual 3D line geometry, not a guessed two-dimensional arrow angle. The Sun ray near the pole is almost tangent to the lunar surface because solar elevation is small. On screen that tangent may be foreshortened; horizontal in the local sky does not require a perfectly horizontal screen line.

| Geometry | Vertex layout | Draw call |
|:--|:--|:--|
| Moon | Position and UV, 20-byte stride | Indexed triangles |
| Direction ray | Position only, 12-byte stride | Two vertices with GL_LINES |

A separate vertex array object stores the ray layout. A tiny buffer receives the two endpoints with STREAM_DRAW, meaning frequently replaced data. It is registered immediately in the renderer's existing owned buffer inventory; the additional vertex array is destroyed on shutdown and on partial initialization failure. The ray-buffer field is an alias, not a second owner. Cleanup clears it after deleting the owned buffers, preventing double deletion. The textured sphere's static mesh is never overwritten.

The same vertex shader is reused. A uniform integer u_ray_mode chooses a constant-color early return in the fragment shader before texture derivatives, normal mapping or Hapke calculations. Zero is restored before every Moon draw; one is selected only for the lines. Yellow is Sun, blue is Earth. Available directions use full RGB brightness; blocked directions use 0.35 times those display values. This is dimming, not blending, and the existing opaque depth-test contract stays intact.

The sphere is drawn first and retains its depth buffer for the lines. Thus the Moon can hide a ray that enters or passes behind it; rays do not become X-ray overlays. Their exact origin can be covered by the red marker, which is drawn afterward. The globe remains a smooth illustrative sphere, not the DEM surface. Real terrain determines dimming in the app, while smooth-sphere depth determines geometric occlusion in this picture. Its fixed photographic light is unchanged and need not align with the scientific Sun ray.

## 12. Verification and limits

Run workspace tests, clippy, native release and WASM checks. Build Trunk from app/ and verify Chrome phone emulation has no page or console errors. Native captures at sample 0 and sample 1138 exercise opposite sunlight/radio states. Inspect rays.png and rays_dark.png, plus rays_mobile.png and a close crop around the marker: origins agree, colors follow the selected status, and the low-elevation Sun ray lies near the local tangent rather than pointing radially away from the pole. Browser testing uses Pixel 7 emulation with DPR 2; it is not a physical phone or the user's GPU.

The new code does not fetch ephemerides, change horizons, change camera tilt, alter MSAA, relight the Moon or modify solar/radio statistics. GL_LINES uses a portable default one-pixel width. At phone sizes or when a direction points nearly along the camera axis, a ray can be very short or partly hidden. This is perspective and occlusion, not a changed scientific elevation.

## 13. Ray API index

- `abs`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `active_texture`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `add`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `all`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `allocate_exact_size`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `allocate_ui`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `allow_double_click_reset`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `allow_drag`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `allow_scroll`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `allow_zoom`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `and_then`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `any`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `Arc::clone`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `Arc::new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `as_chunks`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `as_deref`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `as_raw`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `as_ref`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `aspect_ratio`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `assert!`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `assert_eq!`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `assets::load`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `assets::preferences`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `assets::select`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `at`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `attach_shader`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `availability`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `available_size`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `available_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `bind_attrib_location`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `bind_buffer`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `bind_texture`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `bind_vertex_array`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `BodyDirection`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `buffer_data_u8_slice`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `button`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `bytemuck::cast_slice`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `center`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `changed`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `checkbox`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `circle_filled`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `circle_stroke`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `clamp`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `clear`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `clear_depth_f32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `clicked`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `clone`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `collect`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `color`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Color32::from_gray`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `columns`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `compile_shader`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `contains`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `cos`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `crate::sky::show`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `create_buffer`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `create_program`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `create_shader`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `create_texture`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `create_vertex_array`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `ctx`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `cull_face`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `data::load`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `default_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `default_x_bounds`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `delete_buffer`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `delete_program`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `delete_shader`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `delete_texture`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `delete_vertex_array`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `depth_func`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `depth_mask`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `destroy`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `detach_shader`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `direction`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `directions`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `disable`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `drain`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `draw_arrays`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `draw_elements`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `eframe::egui::CentralPanel::default`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::Layout::top_down`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::ScrollArea::vertical`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `eframe::egui::vec2`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `egui::ComboBox::from_id_salt`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::FontId::proportional`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `egui::Frame::group`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Grid::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::RichText::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Sense::hover`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `egui::Slider::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui::Stroke::new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `egui::vec2`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `egui::Window::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `egui_glow::CallbackFn::new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `egui_glow::ShaderVersion::get`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `enable`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `enable_vertex_attrib_array`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `end_row`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `enumerate`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `eprintln!`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `extend_from_slice`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `f64::from`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `fill`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `filter`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `fixed_decimals`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `flat_map`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `format`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `format!`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `frame`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `front_face`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `generate_mipmap`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_error`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_or_insert`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_parameter_i32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_program_info_log`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_program_link_status`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_shader_compile_status`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_shader_info_log`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `get_uniform_location`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `gl`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `Globe`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `globe::Globe::new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `glow::PixelUnpackData::Slice`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `heading`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `height`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `hline`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `HLine::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `horizon_at`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `horizontal`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `horizontal_wrapped`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `include_str!`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `include_y`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `input`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `into`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `into_iter`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `is_empty`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `is_finite`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `is_new_shader_interface`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `is_none`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `is_none_or`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `is_positive`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `is_some`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `iter`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `json!`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `label`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `last`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `last_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `left_top`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `legend`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Legend::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `len`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `line`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Line::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `link_program`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `local_direction`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `local_sky_frame`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `lock`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `lunar_core::stats`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `lunar_core::Stats::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `map`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `map_or_else`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `marker`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `math::direction`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `math::local_direction`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `math::marker`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `math::perspective`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `math::rotation`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `math::sphere`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `max`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `mesh_orientation_and_seam`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `min`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `MoonApp`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `Mutex::new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `num_columns`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `ok`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `ok_or`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `on_exit`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `on_hover_text`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `open`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `paint`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `painter`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `panorama::Panorama::new`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `parse`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `partition_point`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Pending::default`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `perspective`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `pixel_store_i32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `Plot::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `points`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Points::new`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `projection_depth_and_aspect`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `push`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `radius`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `rem_euclid`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `Renderer::new`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `request_repaint`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `rotation`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `saturating_sub`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `segments`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `selectable_value`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `selected_text`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `serde_json::to_vec`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `set_directions`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `set_margin_fraction`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `set_max_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `set_site`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `set_theme`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `set_width`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `shader_source`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `show`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `show_ui`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `show_value`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sin`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `sin_cos`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `site`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `site_projection_and_south_tilt`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `SiteData::parse`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `size`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `small`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `spacing`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `spacing_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `sphere`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `std::array::from_fn`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `std::env::var`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `strong`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `style`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `style_mut`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `suffix`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `summary_uses_full_duration_and_current_clearance`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `take`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `tan`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `technical`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `tex_image_2d`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `tex_parameter_i32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `text`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `then`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `to_radians`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `to_string`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `transpose`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `u_ray_color`: [annotated fragment shader](#143-globesrcmoonfrag)
- `u_ray_mode`: [annotated fragment shader](#143-globesrcmoonfrag)
- `ui`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `uniform_1_i32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `uniform_3_f32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `unwrap`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `unwrap_or`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `unwrap_or_else`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `update`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `update_earth_margin`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `upload_texture`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `use_program`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `utc_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `vec!`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `Vec::new`: [annotated globe/src/math.rs](#141-globesrcmathrs)
- `version_declaration`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `vertex_attrib_pointer_f32`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `vertical`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `viewport_in_pixels`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `vscroll`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `web_sys::console::log_1`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `width`: [annotated globe/src/lib.rs](#142-globesrclibrs)
- `window`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `window_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `windows_are_utc_intervals_not_sample_counts`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `with_layout`: [annotated app/src/lib.rs](07-ui.md#83-appsrclibrs)
- `wraps_do_not_draw_chords`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_axis_formatter`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_axis_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `x_grid_spacer`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `y_axis_label`: [annotated app/src/panorama.rs](07-ui.md#82-appsrcpanoramars)
- `zip`: [annotated globe/src/lib.rs](#142-globesrclibrs)

## 14. Exact ray implementation quotations

### 14.1. globe/src/math.rs

```rust
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::f32::consts::{FRAC_PI_2, PI, TAU};

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
// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod math;

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

// Keep transformed directions distinct from local input angles; C++ internal plain value struct.
#[derive(Clone, Copy)]
struct Ray
{
	// Moon-fixed unit direction before presentation rotation; C++ array<float,3>.
	direction: [f32; 3],
	// False selects the dim material, independently of depth occlusion; C++ bool.
	visible: bool,
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
	// Optional animation epoch in seconds; C++ optional<double>.
	started: Option<f64>,
	// Optional unit object-space site direction; C++ optional<array<float,3>> without a core dependency.
	site: Option<[f32; 3]>,
	// Geographic radians define north/east even at a pole; C++ optional<pair<float,float>>.
	coordinates: Option<(f32, f32)>,
	// Sun and Earth in fixed color order, absent until a valid time sample arrives; C++ optional<array<Ray,2>>.
	rays: Option<[Ray; 2]>,
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
			loading: assets::load(tier),
			tier,
			// Optional animation epoch in seconds; C++ optional<double>.
			started: None,
			// No marker until valid geographic coordinates arrive; C++ nullopt.
			site: None,
			// No local basis or scientific rays until the app supplies a site/sample; C++ nullopt initialization.
			coordinates: None,
			rays: None,
		})
	}

	// Accept only finite geographic coordinates; absent/invalid input clears the marker; C++ optional setter.
	pub fn set_site(&mut self, site: Option<(f64, f64)>)
	{
		// Convert degrees to the renderer's object-space convention, independent of scientific data types; C++ optional transform.
		self.coordinates = site
			.filter(|&(lat, lon)| {
				lat.is_finite() && lon.is_finite() && (-90.0..=90.0).contains(&lat)
			})
			.map(|(lat, lon)| {
				(
					lat.to_radians() as f32,
					lon.rem_euclid(360.0).to_radians() as f32,
				)
			});
		// Keep the marker and local basis tied to exactly the same site; C++ optional transform.
		self.site = self.coordinates.map(|(lat, lon)| math::direction(lat, lon));
		// Never retain rays belonging to a previous location; C++ optional reset.
		self.rays = None;
	}

	// Accept current Sun/Earth angles in that order; invalid or missing data clears both rays; C++ optional value setter.
	pub fn set_directions(&mut self, bodies: Option<[BodyDirection; 2]>)
	{
		// No stale geometry may survive an absent time sample; C++ reset.
		self.rays = None;
		// Both the site's basis and the time sample are required; C++ two optional guards.
		if let (Some((lat, lon)), Some([sun, earth])) = (self.coordinates, bodies)
		{
			// Convert each value with the same geographic basis; C++ checked lambda returning optional<Ray>.
			let convert = |body: BodyDirection| {
				math::local_direction(lat, lon, body.az_deg.to_radians(), body.el_deg.to_radians())
					.map(|direction| Ray {
						direction,
						visible: body.visible,
					})
			};
			// Submit a complete pair only when both directions are valid; C++ zip/transform of two optionals.
			self.rays = convert(sun).zip(convert(earth)).map(|(s, e)| [s, e]);
		}
	}

	/// Fill the available UI rectangle and rotate once per 60 seconds of egui time.
	// Queue a paint callback for this frame and request the next; C++ an immediate-mode UI calls this each frame.
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if self.renderer.lock().is_none()
		{
			// Choose an error message or the current loading label; C++ uses an owning string.
			let label = self
				.loading
				// Acquire exclusive access until the temporary guard is dropped; C++ lock_guard plus protected access.
				.lock()
				.error
				// Copy the optional diagnostic or increment a shared ownership count, not GPU resources; C++ value copy/shared_ptr copy.
				.clone()
				// Compute the fallback only when absent; C++ checks optional before calling a fallback lambda.
				.unwrap_or_else(|| format!("Loading Moon {}k...", self.tier));
			// Paint the pending/error message on the black panel; C++ immediate-mode UIs similarly submit a label each frame.
			ui.label(label);
		}
		// Reserve the available paint rectangle; C++ would receive a rectangle/response pair.
		let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if !rect.is_positive()
		{
			// Stop this frame's work without a value; C++ return exits a void function.
			return;
		}
		// Read monotonic egui time in seconds; C++ uses the frame clock.
		let time = ui.input(|input| input.time);
		// Initialize the first-frame epoch once, then copy it; C++ optional::emplace and dereference.
		let start = *self.started.get_or_insert(time);
		// Wrap time to one turn per 60 seconds and offset the initial longitude; C++ fmod plus a float cast.
		let angle = ((time - start).rem_euclid(60.0) / 60.0) as f32 * std::f32::consts::TAU - 0.5;
		// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
		let renderer = Arc::clone(&self.renderer);
		// Keep the loader alive until this callback finishes; C++ copies its shared_ptr.
		let loading = Arc::clone(&self.loading);
		// Snapshot the selected sample for the deferred GL callback; C++ copy small optional values into a lambda.
		let rays = self.site.zip(self.rays);
		// Capture shared state for egui's later current-context draw; C++ stores a value-capturing std::function.
		let callback = egui_glow::CallbackFn::new(move |info, painter| {
			// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
			let mut renderer = renderer.lock();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if renderer.is_none() && loading.lock().error.is_none()
			{
				// Carry either the next successful value or a diagnostic; C++ could use expected<T,string>.
				let result = loading.lock().take().and_then(|images| {
					images
						// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
						.map(|images| Renderer::new(painter.gl(), images))
						// Convert optional<Result<T,E>> to Result<optional<T>,E>; C++ explicitly handles the three possible states.
						.transpose()
				});
				// Exhaustively branch on the result variant; C++ switch over a variant tag requires explicit extraction.
				match result
				{
					// Move the completed renderer into its shared slot; C++ moves the success value into optional storage.
					Ok(Some(ready)) => *renderer = Some(ready),
					// Report that construction is still pending; C++ optional remains empty without signaling failure.
					Ok(None) =>
					{}
					// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
					Err(error) => loading.lock().error = Some(error),
				}
			}
			// Convert egui's logical rectangle to actual pixel dimensions; C++ receives integer width/height.
			let viewport = info.viewport_in_pixels();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if viewport.width_px > 0 && viewport.height_px > 0
			{
				// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
				if let Some(renderer) = renderer.as_ref()
				{
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.paint(
						painter.gl(),
						viewport.width_px as f32 / viewport.height_px as f32,
						angle,
						rays,
					);
				}
			}
		});
		// Append the custom GL callback to egui's paint list; C++ queues an owned rendering command.
		ui.painter().add(egui::PaintCallback {
			rect,
			callback: Arc::new(callback),
		});
		// Draw the marker only after the Moon is available and the site faces the camera; C++ checked optional projection.
		if self.renderer.lock().is_some()
			&& let Some(ndc) = self
				.site
				.and_then(|p| math::marker(p, angle, rect.aspect_ratio()))
		{
			// Convert NDC to top-left-origin logical UI coordinates, reversing vertical sign; C++ viewport transform.
			let position = rect.center()
				+ egui::vec2(ndc[0] * rect.width() * 0.5, -ndc[1] * rect.height() * 0.5);
			// Keep marker radius constant on screen, independent of texture tier; C++ draw a filled overlay circle.
			ui.painter()
				.circle_filled(position, 4.5, egui::Color32::LIGHT_RED);
			// Add an outline for contrast against either highlands or a dark terminator; C++ stroked-circle overlay.
			ui.painter()
				.circle_stroke(position, 4.5, egui::Stroke::new(1.0, egui::Color32::WHITE));
		}
		// Distinguish the presentation camera/light from scientific ephemerides in the adjacent plot; C++ overlay text.
		ui.painter().text(
			rect.left_top() + egui::vec2(8.0, 8.0),
			egui::Align2::LEFT_TOP,
			"Site marker: red | illustrative south view",
			egui::FontId::proportional(12.0),
			egui::Color32::LIGHT_RED,
		);
		// Explain ray colors, dimming and the non-distance scale without changing the phone layout; C++ multiline overlay label.
		ui.painter().text(
			rect.left_top() + egui::vec2(8.0, 25.0),
			egui::Align2::LEFT_TOP,
			"Yellow: Sun | blue: Earth | dim: blocked\nRays use selected time; length not to scale",
			egui::FontId::proportional(11.0),
			egui::Color32::LIGHT_GRAY,
		);
		// Request another frame so loading and animation progress; C++ asks the event loop to redraw.
		ui.ctx().request_repaint();
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

// Declare named fields; Rust methods live in a separate impl, unlike inline C++ class definitions.
struct Renderer
{
	// Optional owned linked-program handle; C++ optional<GLuint>.
	program: Option<glow::Program>,
	// Own any not-yet-deleted shader handles during construction; C++ vector<GLuint>.
	shaders: Vec<glow::Shader>,
	// Optional owned vertex-array handle; C++ optional<GLuint>.
	vao: Option<glow::VertexArray>,
	// Own a separate position-only layout for the two direction lines; C++ optional<GLuint>.
	ray_vao: Option<glow::VertexArray>,
	// Alias a buffer owned by buffers for dynamic ray uploads; C++ nonowning optional<GLuint>.
	ray_buffer: Option<glow::Buffer>,
	// Own vertex and index buffer handles; C++ vector<GLuint>.
	buffers: Vec<glow::Buffer>,
	// Own the albedo and normal textures in sampler order; C++ vector<GLuint>.
	textures: Vec<glow::Texture>,
	// Number of indices passed to the draw, not the number of triangles; C++ GLsizei.
	index_count: i32,
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Renderer
{
	// Construct and validate owned state, returning an error instead of throwing; C++ uses a factory returning expected.
	fn new(gl: &glow::Context, [albedo, normals]: [image::RgbImage; 2]) -> Result<Self, String>
	{
		// Inspect GLSL capabilities rather than assuming desktop syntax; C++ queries GL strings.
		let version = egui_glow::ShaderVersion::get(gl);
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if !version.is_new_shader_interface()
		{
			// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
			return Err("The Moon requires modern OpenGL or WebGL2".into());
		}
		// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
		let mut renderer = Self {
			// Optional owned linked-program handle; C++ optional<GLuint>.
			program: None,
			// Own any not-yet-deleted shader handles during construction; C++ vector<GLuint>.
			shaders: Vec::new(),
			// Optional owned vertex-array handle; C++ optional<GLuint>.
			vao: None,
			// Ray resources participate in the same partial-construction cleanup; C++ nullopt initialization.
			ray_vao: None,
			ray_buffer: None,
			// Own vertex and index buffer handles; C++ vector<GLuint>.
			buffers: Vec::new(),
			// Own the albedo and normal textures in sampler order; C++ vector<GLuint>.
			textures: Vec::new(),
			// Number of indices passed to the draw, not the number of triangles; C++ GLsizei.
			index_count: 0,
		};
		// Run fallible GPU construction in a local closure so partial resources can be cleaned; C++ try/catch is analogous.
		let initialized = (|| -> Result<(), String> {
			// SAFETY: eframe calls the constructor with its GL context current. Every handle is
			// registered immediately so the error branch can clean partial construction.
			// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
			unsafe {
				// Allocate a linkable program; C++ glCreateProgram returns a GLuint, while glow can return an error.
				let program = gl.create_program()?;
				// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
				renderer.program = Some(program);
				// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
				for (kind, source) in [
					// Embed only the vertex shader's small UTF-8 source, never image tiers; C++ embeds a string literal.
					(glow::VERTEX_SHADER, include_str!("moon.vert")),
					// Embed the matching fragment shader; C++ embeds another shader source string.
					(glow::FRAGMENT_SHADER, include_str!("moon.frag")),
				]
				{
					// Allocate one shader stage; C++ glCreateShader takes the same stage enum.
					let shader = gl.create_shader(kind)?;
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.shaders.push(shader);
					// Supply GLSL text with the selected desktop/WebGL version; C++ glShaderSource takes pointer arrays.
					gl.shader_source(
						shader,
						&format!("{}\n{source}", version.version_declaration()),
					);
					// Compile the stage now; C++ glCompileShader uses the same handle.
					gl.compile_shader(shader);
					// Test compilation before attaching; C++ glGetShaderiv with GL_COMPILE_STATUS.
					if !gl.get_shader_compile_status(shader)
					{
						// Return the compiler diagnostic as String; C++ glGetShaderInfoLog fills a character buffer.
						return Err(gl.get_shader_info_log(shader));
					}
					// Attach this compiled stage to the program; C++ glAttachShader.
					gl.attach_shader(program, shader);
				}
				// Fix the input location to match the vertex layout; C++ glBindAttribLocation before linking.
				gl.bind_attrib_location(program, 0, "a_position");
				// Fix the input location to match the vertex layout; C++ glBindAttribLocation before linking.
				gl.bind_attrib_location(program, 1, "a_uv");
				// Connect vertex outputs to fragment inputs; C++ glLinkProgram.
				gl.link_program(program);
				// Reject incompatible stages; C++ glGetProgramiv with GL_LINK_STATUS.
				if !gl.get_program_link_status(program)
				{
					// Recover the linker's diagnostic; C++ glGetProgramInfoLog.
					return Err(gl.get_program_info_log(program));
				}
				// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
				for shader in renderer.shaders.drain(..)
				{
					// Release the program's stage attachment after a successful link; C++ glDetachShader.
					gl.detach_shader(program, shader);
					// Delete the no-longer-needed shader object; C++ glDeleteShader.
					gl.delete_shader(shader);
				}
				// Own interleaved position/UV values on the CPU; C++ vector<float>.
				let (vertices, indices) = math::sphere(256, 128);
				// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
				renderer.index_count = indices.len() as i32;
				// Allocate vertex-layout state; C++ glGenVertexArrays.
				renderer.vao = Some(gl.create_vertex_array()?);
				// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
				gl.bind_vertex_array(renderer.vao);
				// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
				for (target, bytes) in [
					// Borrow the float vertex vector as bytes without copying; C++ reinterpret_cast to a const byte span.
					(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices)),
					// Borrow the u32 index vector as upload bytes; C++ a const byte span preserves the original storage.
					(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&indices)),
				]
				{
					// Allocate storage ownership for vertices or indices; C++ glGenBuffers.
					let buffer = gl.create_buffer()?;
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.buffers.push(buffer);
					// Select the buffer receiving the next operation; C++ glBindBuffer(target, handle or 0).
					gl.bind_buffer(target, Some(buffer));
					// Copy CPU bytes to static-draw GPU storage; C++ glBufferData(target, byte_count, data, GL_STATIC_DRAW).
					gl.buffer_data_u8_slice(target, bytes, glow::STATIC_DRAW);
				}
				// Enable a vertex input array at this location; C++ glEnableVertexAttribArray.
				gl.enable_vertex_attrib_array(0);
				// Describe floats, byte stride, and byte offset in each vertex; C++ glVertexAttribPointer.
				gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 20, 0);
				// Enable a vertex input array at this location; C++ glEnableVertexAttribArray.
				gl.enable_vertex_attrib_array(1);
				// Describe floats, byte stride, and byte offset in each vertex; C++ glVertexAttribPointer.
				gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 20, 12);
				// Allocate a second VAO without disturbing the Moon's indexed mesh; C++ glGenVertexArrays.
				renderer.ray_vao = Some(gl.create_vertex_array()?);
				// Configure only this layout for position triples; C++ glBindVertexArray.
				gl.bind_vertex_array(renderer.ray_vao);
				// Allocate a tiny dynamic buffer for four ray vertices; C++ glGenBuffers.
				let ray_buffer = gl.create_buffer()?;
				// Register ownership immediately for cleanup on later failure; C++ owned handle vector append.
				renderer.buffers.push(ray_buffer);
				// Retain a nonowning handle for subsequent uploads; C++ handle copy.
				renderer.ray_buffer = Some(ray_buffer);
				// Make the new buffer the source of position attribute zero; C++ glBindBuffer.
				gl.bind_buffer(glow::ARRAY_BUFFER, Some(ray_buffer));
				// Enable only position; unused UV defaults to zero for the constant-color branch; C++ glEnableVertexAttribArray.
				gl.enable_vertex_attrib_array(0);
				// Three tightly packed floats describe each ray vertex; C++ glVertexAttribPointer.
				gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 12, 0);
				// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
				gl.bind_vertex_array(None);
				// Select the buffer receiving the next operation; C++ glBindBuffer(target, handle or 0).
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			renderer.upload_texture(gl, albedo.width(), albedo.height(), albedo.as_raw(), true)?;
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			renderer.upload_texture(
				gl,
				normals.width(),
				normals.height(),
				normals.as_raw(),
				false,
			)?;
			// Wrap a successful result without throwing; C++ expected stores its value.
			Ok(())
		})();
		// Detect a failed initialization so partial GPU ownership can be cleaned; C++ checks expected's error.
		if let Err(error) = initialized
		{
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			renderer.destroy(gl);
			// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
			return Err(error);
		}
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		eprintln!("Moon textures ready");
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		web_sys::console::log_1(&"Moon textures ready".into());
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(renderer)
	}

	// Upload RGB bytes and configure mipmapped sampling; C++ receives a const byte span plus dimensions.
	fn upload_texture(
		&mut self,
		gl: &glow::Context,
		width: u32,
		height: u32,
		pixels: &[u8],
		srgb: bool,
	) -> Result<(), String>
	{
		// WebGL2 can generate mipmaps for SRGB8_ALPHA8, but not for bare SRGB8.
		// Expand only albedo to opaque RGBA; C++ builds the same vector from RGB triples.
		let rgba: Option<Vec<u8>> = srgb.then(|| {
			pixels
				.as_chunks::<3>()
				.0
				.iter()
				.flat_map(|p| [p[0], p[1], p[2], 255])
				.collect()
		});
		// Borrow expanded bytes when present, otherwise the original normal-map bytes; C++ span.
		let upload = rgba.as_deref().unwrap_or(pixels);
		// SAFETY: the context is current during initialization; decoded RGB storage is
		// width * height * 3 bytes. SRGB8_ALPHA8 decodes albedo before filtering; RGB8 stores normals.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Read an integer capability into Rust; C++ glGetIntegerv writes through an output pointer.
			let max_side = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32;
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if width > max_side || height > max_side
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err("Moon texture exceeds the device texture-size limit".into());
			}
			// Select which texture unit subsequent binds affect; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Allocate a texture handle owned by this renderer; C++ glGenTextures.
			let texture = gl.create_texture()?;
			// Register ownership before any later upload operation can fail; C++ vector::push_back records the GLuint.
			self.textures.push(texture);
			// Bind or unbind an image at the active unit; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Set row-byte alignment for RGB upload, then restore egui's default; C++ glPixelStorei.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			// Upload level zero with explicit color-space storage; C++ glTexImage2D receives the pixel pointer.
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				width as i32,
				height as i32,
				0,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(upload)),
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as i32,
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			// Build all lower-resolution levels from level zero; C++ glGenerateMipmap.
			gl.generate_mipmap(glow::TEXTURE_2D);
			// Keep the texture's default anisotropy ratio of one: textureGrad with anisotropy made dotted limb arcs on llvmpipe.
			// Trilinear mipmaps and MSAA remain enabled; C++ likewise leaves GL_TEXTURE_MAX_ANISOTROPY_EXT at its default.
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_WRAP_T,
				glow::CLAMP_TO_EDGE as i32,
			);
			// Set row-byte alignment for RGB upload, then restore egui's default; C++ glPixelStorei.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
			// Bind or unbind an image at the active unit; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, None);
			// Retrieve and clear a GL error flag after upload; C++ glGetError.
			let error = gl.get_error();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if error != glow::NO_ERROR
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err(format!("Moon texture upload GL error: {error:#x}"));
			}
		}
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(())
	}

	// Draw one opaque sphere inside egui's already configured viewport; C++ issues the equivalent GL commands.
	fn paint(&self, gl: &glow::Context, aspect: f32, angle: f32, rays: Option<([f32; 3], [Ray; 2])>)
	{
		// Borrow the optional program handle or return from an already-destroyed renderer; C++ checks optional before dereferencing.
		let Some(program) = self.program
		// Handle the complementary case; C++ uses the same else branch.
		else
		{
			// Stop this frame's work without a value; C++ return exits a void function.
			return;
		};
		// SAFETY: CallbackFn runs on eframe's current GL context, after its viewport/scissor
		// setup. The app requests a 24-bit depth buffer on native and web. egui_glow 0.37
		// draws to the default framebuffer, disables framebuffer-sRGB, and restores UI state.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Enable the selected fixed-function test; C++ glEnable.
			gl.enable(glow::DEPTH_TEST);
			// Keep fragments closer than the stored depth; C++ glDepthFunc(GL_LESS).
			gl.depth_func(glow::LESS);
			// Allow surviving fragments to update depth; C++ glDepthMask(GL_TRUE).
			gl.depth_mask(true);
			// Set the farthest normalized depth as the clear value; C++ glClearDepth or WebGL clearDepth.
			gl.clear_depth_f32(1.0);
			// Reset depth for this callback's scissored viewport; C++ glClear(GL_DEPTH_BUFFER_BIT).
			gl.clear(glow::DEPTH_BUFFER_BIT);
			// Disable blending for opaque drawing, or restore depth/culling after it; C++ glDisable.
			gl.disable(glow::BLEND);
			// Enable the selected fixed-function test; C++ glEnable.
			gl.enable(glow::CULL_FACE);
			// Discard back-facing triangles; C++ glCullFace(GL_BACK).
			gl.cull_face(glow::BACK);
			// Treat counterclockwise vertex order as front-facing; C++ glFrontFace(GL_CCW).
			gl.front_face(glow::CCW);
			// Select linked shaders, or unbind them; C++ glUseProgram(handle or 0).
			gl.use_program(Some(program));
			// Always restore the textured material before drawing the Moon; C++ glUniform1i.
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 0);
			// Send a column-major projection without transposing; C++ glUniformMatrix4fv.
			gl.uniform_matrix_4_f32_slice(
				// Look up an active uniform, returning None if absent; C++ glGetUniformLocation returns -1.
				gl.get_uniform_location(program, "u_projection").as_ref(),
				false,
				&math::perspective(aspect),
			);
			// Share one column-major rigid model matrix with geometry, normals, and CPU marker projection; C++ glUniformMatrix3fv.
			gl.uniform_matrix_3_f32_slice(
				gl.get_uniform_location(program, "u_model").as_ref(),
				false,
				&math::rotation(angle),
			);
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for (unit, name) in ["u_albedo", "u_normal"].iter().enumerate()
			{
				// Select which texture unit subsequent binds affect; C++ glActiveTexture.
				gl.active_texture(glow::TEXTURE0 + unit as u32);
				// Bind or unbind an image at the active unit; C++ glBindTexture.
				gl.bind_texture(glow::TEXTURE_2D, Some(self.textures[unit]));
				// Tell a sampler which texture-unit index to use; C++ glUniform1i.
				gl.uniform_1_i32(gl.get_uniform_location(program, name).as_ref(), unit as i32);
			}
			// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
			gl.bind_vertex_array(self.vao);
			// Submit indexed triangles with 32-bit indices starting at byte zero; C++ glDrawElements.
			gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
			// Draw real object-space direction lines against the sphere's existing depth buffer; C++ optional guard.
			if let Some((origin, bodies)) = rays
			{
				// Reuse the exact model/projection uniforms, but skip all texture/photometry work; C++ material switch.
				gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 1);
				// Select the independent, position-only line layout; C++ glBindVertexArray.
				gl.bind_vertex_array(self.ray_vao);
				// Upload two endpoints per ray without modifying the sphere buffers; C++ glBindBuffer.
				gl.bind_buffer(glow::ARRAY_BUFFER, self.ray_buffer);
				// Preserve Sun/yellow then Earth/blue order from the app; C++ array zip iteration.
				for (body, color) in bodies
					.into_iter()
					.zip([[1.0, 0.9, 0.05], [0.15, 0.65, 1.0]])
				{
					// Direction rays are 0.6 lunar radii, not body distances; C++ endpoint vector arithmetic.
					let end: [f32; 3] =
						std::array::from_fn(|i| origin[i] + 0.6 * body.direction[i]);
					// The first vertex equals the marker's surface point exactly; C++ contiguous two-vertex array.
					let vertices = [origin, end];
					// Twelve floats at most per frame need no persistent mapping; C++ glBufferData with GL_STREAM_DRAW.
					gl.buffer_data_u8_slice(
						glow::ARRAY_BUFFER,
						bytemuck::cast_slice(&vertices),
						glow::STREAM_DRAW,
					);
					// Dim failed terrain/radio predicates without disabling physical sphere occlusion; C++ conditional scalar.
					let strength = if body.visible { 1.0 } else { 0.35 };
					// Send an unlit display color rather than illuminating rays with the globe's illustration light; C++ glUniform3f.
					gl.uniform_3_f32(
						gl.get_uniform_location(program, "u_ray_color").as_ref(),
						color[0] * strength,
						color[1] * strength,
						color[2] * strength,
					);
					// Use the portable one-pixel GL line rasterizer on desktop and WebGL2; C++ glDrawArrays(GL_LINES,0,2).
					gl.draw_arrays(glow::LINES, 0, 2);
				}
				// Do not leak a dynamic buffer binding into egui; C++ glBindBuffer(...,0).
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
			// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
			gl.bind_vertex_array(None);
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for unit in 0..2
			{
				// Select which texture unit subsequent binds affect; C++ glActiveTexture.
				gl.active_texture(glow::TEXTURE0 + unit);
				// Bind or unbind an image at the active unit; C++ glBindTexture.
				gl.bind_texture(glow::TEXTURE_2D, None);
			}
			// Select which texture unit subsequent binds affect; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Select linked shaders, or unbind them; C++ glUseProgram(handle or 0).
			gl.use_program(None);
			// Disable blending for opaque drawing, or restore depth/culling after it; C++ glDisable.
			gl.disable(glow::DEPTH_TEST);
			// Disable blending for opaque drawing, or restore depth/culling after it; C++ glDisable.
			gl.disable(glow::CULL_FACE);
		}
	}

	// Release registered GPU objects using the current context; C++ explicit teardown avoids a late invalid-context destructor.
	fn destroy(&mut self, gl: &glow::Context)
	{
		// SAFETY: called with the owning context current, either on_exit or failed construction.
		// Taking/draining handles makes repeated cleanup harmless.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for texture in self.textures.drain(..)
			{
				// Release GPU texture storage while the context is current; C++ glDeleteTextures.
				gl.delete_texture(texture);
			}
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for buffer in self.buffers.drain(..)
			{
				// Release vertex/index storage while the context is current; C++ glDeleteBuffers.
				gl.delete_buffer(buffer);
			}
			// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
			if let Some(vao) = self.vao.take()
			{
				// Release vertex-layout state while the context is current; C++ glDeleteVertexArrays.
				gl.delete_vertex_array(vao);
			}
			// Delete the ray layout once; its buffer is already owned by the shared buffer inventory; C++ optional extraction.
			if let Some(vao) = self.ray_vao.take()
			{
				// Release layout state while the context is current; C++ glDeleteVertexArrays.
				gl.delete_vertex_array(vao);
			}
			// Clear the nonowning alias after buffer destruction; C++ handle reset.
			self.ray_buffer = None;
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for shader in self.shaders.drain(..)
			{
				// Delete the no-longer-needed shader object; C++ glDeleteShader.
				gl.delete_shader(shader);
			}
			// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
			if let Some(program) = self.program.take()
			{
				// Release the linked executable while the context is current; C++ glDeleteProgram.
				gl.delete_program(program);
			}
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
uniform int u_ray_mode;
// Direction-line colors are display colors, independent of illustrative illumination; C++ RGB uniform.
uniform vec3 u_ray_color;
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

// Run once per vertex or fragment respectively; GLSL entry points have no C++ object ownership.
void main()
{
	// All fragments in this draw take the same branch, before texture derivatives or Hapke evaluation; C++ uniform material switch.
	if (u_ray_mode != 0)
	{
		// Preserve opaque depth-tested lines; dimming is encoded in RGB, not an altered depth or lighting model.
		out_color = vec4(u_ray_color, 1.0);
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
	// Apply the geometry's rigid rotation and south tilt to the normal, preserving lighting-space consistency.
	n = normalize(u_model * n);
	// Fix a normalized directional Sun vector in world space so a terminator is visible.
	vec3 light = normalize(vec3(1.0, 0.2, 0.6));
	// Point from the surface fragment to the perspective camera at z=3.3.
	vec3 view = normalize(vec3(0.0, 0.0, 3.3) - v_position);
	// Fade unresolved relief in the foreshortened limb band; use the true sphere normal below cosine 0.35.
	vec3 geometric = normalize(v_position);
	// Retain full relief above cosine 0.65, with a continuous smoothstep transition between both angles.
	float normal_strength = smoothstep(0.35, 0.65, max(dot(geometric, view), 0.0));
	// A normal map cannot change silhouette visibility: back-facing mapped facets must not cut black holes.
	// Add only the missing view component, then restore unit length; leave already front-facing normals alone.
	n = normalize(n + max(0.0, MIN_EMISSION_COSINE - dot(n, view)) * view);
	// Evaluate radiance factor from incidence, emission, and phase, with no ambient component.
	float mapped_reflectance = hapke(dot(n, light), dot(n, view), dot(light, view));
	// Evaluate the smooth sphere separately: terrain normals cannot describe resolved visibility at the limb.
	float geometric_reflectance = hapke(dot(geometric, light), dot(geometric, view), dot(light, view));
	// Cross-fade the two sunlight responses, rather than inventing ambient light or clamping the display color.
	float reflectance = mix(geometric_reflectance, mapped_reflectance, normal_strength);
	// Prevent local normal tilts lighting the geometric night side without terrain shadows.
	// Mask the geometric night side because this smooth silhouette cannot cast resolved terrain shadows.
	if (dot(normalize(v_position), light) <= 0.0)
	{
		// Remove sunlight on the geometric back side even if a mapped normal tilts toward the light.
		reflectance = 0.0;
	}
	// SRGB8_ALPHA8 texture sampling already decodes color and filters in linear space.
	// Sample SRGB8_ALPHA8 color with explicit seam-safe derivatives; the GPU returns linear color.
	vec3 albedo = textureGrad(u_albedo, uv, gx, gy).rgb;
	// Solid-angle-weighted mean linear luminance; this is not a per-pixel inversion.
	// Modulate the averaged photometric response by relative spatial color and apply exposure.
	vec3 radiance = EXPOSURE * (albedo / 0.4832) * reflectance;
	// Compress positive linear radiance with the Reinhard operator before display encoding.
	vec3 mapped = radiance / (vec3(1.0) + radiance);
	// Encode once to sRGB and set alpha to one; egui_glow disables framebuffer sRGB encoding.
	out_color = vec4(linear_to_srgb(mapped), 1.0);
}
```
