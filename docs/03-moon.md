# The rotating Moon renderer

## 1. Scope, sources, and prerequisites

The new `globe` library draws an illustrative Moon. The `app` package owns the window and embeds this renderer. Neither package sends terrain or lighting results to `lunar_core`. The Sun direction, camera, exposure, and rotation here are artistic constants, not JPL Horizons results. This display is not a landing-site visibility calculation, a calibrated radiometric simulation, or a terrain panorama.

The [Rust introduction](01-skeleton.md#2-rust-vocabulary-in-dependency-order) explains ownership, borrowing, traits, closures, slices, results, and macros. The [original API walkthrough](02-code.md) supplies detailed definitions for the existing workspace. This document defines the new graphics concepts and APIs before quoting their implementations. Rust source quotations retain the repository's tabs and Allman braces.

`pub(crate)` restricts a Rust item to its own library crate, unlike `pub`, which permits downstream crates to name it. The mathematical helpers are internal implementation details, not new scientific APIs. `mod math` loads that private module from `math.rs`. Dereference coercion lets a borrowed `Arc<glow::Context>` be passed where a borrowed Context is required, without copying the context, much as dereferencing a C++ smart pointer exposes its object.

The [NASA CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/) supplies both maps. A map is a rectangular collection of samples attached to surface coordinates. The color map supplies dimensionless display color, interpreted here as sRGB, not absolute albedo. The height map supplies unsigned 16-bit elevations. See [the asset manifest](../globe/assets/README.md) for exact URLs, byte sizes, units, hashes, credits, and reproduction instructions. Credit is NASA's Scientific Visualization Studio. The renderer embeds a 2,048 by 1,024 JPEG and a lossless 1,440 by 720 PNG conversion of the four-pixels-per-degree LOLA product. Loading them requires no runtime network access.

The workspace adds `globe` to its members. `app` adds a path dependency on it. `globe` depends on `egui` and `egui_glow` version 0.37 for integration, `bytemuck` 1.25 for checked slice reinterpretation, and `image` 0.25 with only the JPEG and PNG decoding features. Default `image` features are disabled. The manifest and Cargo.lock pin the resolved dependency graph; `lunar_core` has no new dependency and no source changes. `egui_plot` remains an existing app dependency, but the globe does not use it.

## 2. GPU and OpenGL definitions

A vertex is one mesh sample with a three-component position and a two-component texture coordinate. A triangle is an ordered group of three vertices. A mesh is the full set of vertices and triangle connections. The GPU rasterizes triangles, identifying which screen samples they cover. A fragment is a candidate contribution at such a sample. Front faces use counterclockwise order as viewed from outside. Back-face culling discards the reverse order.

A buffer is GPU-managed byte storage. The vertex buffer object, abbreviated VBO, stores interleaved position and texture coordinates. The index buffer object, abbreviated IBO, stores integer vertex references, allowing triangles to reuse vertices. A vertex array object, abbreviated VAO, records how vertex attributes are read from buffers and which index buffer is attached. An attribute is one per-vertex input, such as position. Each vertex here occupies 20 bytes: three 32-bit floats at byte offset zero, followed by two at byte offset 12.

A shader is a GPU program stage written in GLSL, the OpenGL Shading Language. A vertex shader transforms positions and provides values for interpolation across triangles. A fragment shader calculates a pixel's color from those interpolated values. Linking connects the stages into a program. A uniform is a program input constant across one draw call, such as a matrix or rotation angle. A draw call requests GPU processing of indexed triangles with the currently bound program and resources.

A texture is a GPU array of samples. A texel is one stored sample. UV coordinates are two normalized texture coordinates, named u and v. A sampler uniform selects a texture unit, an indexed binding slot, rather than a texture handle directly. Linear filtering interpolates nearby texels. Repeating the horizontal coordinate joins the two longitude edges. Clamping the vertical coordinate avoids connecting north to south. There are no mipmaps, which are successively lower-resolution texture copies. At small window sizes this simple filter can alias; future work can add appropriate filtered normal mipmaps.

The depth buffer stores a distance-related value per screen sample. The depth test keeps nearer fragments. The framebuffer is the target collection of color and optional depth attachments. The viewport maps normalized device coordinates to a pixel rectangle. The scissor test limits updates to a clip rectangle. These are distinct: a viewport transforms coordinates; a scissor rejects samples outside its rectangle. Blending combines new colors with existing ones. It is disabled for the opaque Moon.

An OpenGL context holds driver state and resources. It must be current on the calling thread when GL commands execute. Rust's type system does not establish that condition. `glow` provides Rust bindings to OpenGL and WebGL with nearly the same arguments as C++ OpenGL. `HasContext` is its trait exposing those commands. Its methods are unsafe because a valid current context, valid resources, and valid memory layouts are caller obligations.

An sRGB value is a nonlinear encoding of a color channel. A linear value is proportional to light in that channel. Multiplying sRGB samples directly is not a valid linear-light calculation. A normal is an outward unit vector perpendicular to a surface; it is directional data, not sRGB color. The two textures therefore share ordinary RGB8 storage but have different shader interpretations. An RGB8 texture stores three unsigned 8-bit channels; sampling converts them to numbers between zero and one without automatic sRGB decoding.

## 3. Sphere coordinates, mesh, and camera

Let latitude be phi, in radians, increasing northwards. Let longitude be lambda, in radians, increasing eastwards. Let pi be the half-turn constant, and tau = 2 pi. The renderer uses an independent three-dimensional object coordinate system: x points east at longitude zero, y points north, and z points toward the initial camera-facing near side. These are not the scientific crate's two-dimensional polar-projection axes.

For a unit sphere, the radial unit vector is

```text
er(phi, lambda) = (cos(phi) sin(lambda), sin(phi), cos(phi) cos(lambda)).
```

The sum of its squared components is cos(phi)^2 + sin(phi)^2 = 1. At longitude zero and latitude zero it points along positive z. At east longitude 90 degrees it points along positive x, which appears on the camera's right. North is positive y. Thus Tycho's southern latitude appears below the equator, and Mare Crisium's eastern longitude appears to the right of the near-side center, not mirrored.

The map is equirectangular: equal horizontal pixel intervals represent equal longitude intervals, and equal vertical intervals represent equal latitude intervals. Define u between zero and one from left to right and v between zero and one from top to bottom. Then lambda = tau(u - 0.5) and phi = pi/2 - pi v. No image rows are flipped. OpenGL's first uploaded row is sampled near v = 0; the shader's v = 0 deliberately means lunar north, regardless of conventions used in other renderers.

`sphere(128, 64)` constructs 129 by 65 vertices, including duplicated longitude edges with u = 0 and u = 1. It returns 8,385 vertices and 49,152 indices, describing 16,384 triangles. Some triangles at the exact poles are degenerate and contribute no area. For a cell, a is its top-left vertex and b its bottom-left. The triangles `(a, b, a+1)` and `(a+1, b, b+1)` have outward counterclockwise winding. Duplicating the seam lets interpolation follow neighboring u values rather than interpolate directly across almost a full turn.

Let theta denote the model's rotation angle. Rotation around positive y is

```text
x' = cos(theta) x + sin(theta) z
y' = y
z' = -sin(theta) x + cos(theta) z.
```

This matrix is orthogonal, so it preserves lengths and transforms normals identically to positions. No inverse-transpose correction beyond that rotation is needed. The shader receives theta in radians. The UI computes one revolution per 60 seconds of monotonic egui input time, with a -0.5 radian initial offset. This brings the near side to the center shortly after startup. It reduces time modulo 60 before converting to f32, preventing long-running floating-point angle growth. This is not the real lunar rotation period.

The camera is at C = (0, 0, 3.3) in sphere-radius units and looks down negative z, with positive y up. Subtracting C is the view transformation because the camera has no additional rotation. A point in camera coordinates is `(xc, yc, zc)`, with visible points having zc < 0. Perspective division uses clip w = -zc. Near-plane and far-plane distances are n = 0.1 and f = 10.

Let a be the positive viewport width divided by height. For landscape windows, the vertical field of view is 45 degrees. Define k = min(a, 1)/tan(pi/8). The min factor widens the vertical field in portrait windows to keep the horizontal field at 45 degrees, so the Moon still fits. The projection equations are

```text
clip_x = (k/a) xc
clip_y = k yc
clip_z = ((f+n)/(n-f)) zc + 2fn/(n-f)
clip_w = -zc.
```

At zc = -n, clip_z/clip_w = -1; at zc = -f, it is +1. These conditions derive the last two depth coefficients. The GPU maps that interval into the depth buffer's zero-to-one range. `perspective` stores this matrix column-major, as OpenGL expects with transpose set to false. A zero-sized viewport is skipped before calculating a.

## 4. Metric normal-map derivation and CPU APIs

Let R = 1,737,400 m. Let q be one NASA unsigned height sample. Its height relative to R is h = 0.5q - 10,000 m, because NASA added 20,000 half-meter units to signed heights. Let E = 2.0 be the display exaggeration, defined by `EXAGGERATION`, and let H = E h. The radial surface is P = (R+H) er. The visible mesh itself remains a unit sphere; this surface is used only to derive shading normals, not displaced vertices or self-shadowing.

The unit east vector is eE = (cos(lambda), 0, -sin(lambda)). The unit north vector is eN = (-sin(phi) sin(lambda), cos(phi), -sin(phi) cos(lambda)). These are mutually perpendicular to er and to each other. Differentiating P gives

```text
dP/dlambda = H_lambda er + (R+H) cos(phi) eE
dP/dphi   = H_phi er + (R+H) eN.
```

Here H_lambda and H_phi denote height derivatives per radian. A vector perpendicular to both tangents is

```text
Nraw = er - H_lambda / ((R+H) cos(phi)) eE - H_phi / (R+H) eN.
N = Nraw / length(Nraw).
```

For example, dotting Nraw with the longitude tangent gives H_lambda - H_lambda = 0. The latitude tangent cancels the same way. The cosine factor is essential: longitude intervals shrink in physical distance toward a pole. Omitting meters-to-radius scaling would vastly overstate slopes.

For width W and height T, the pixel-center spacings are d_lambda = tau/W and d_phi = pi/T. The longitude derivative uses `(H_east-H_west)/(2 d_lambda)` with horizontal wrapping. The latitude derivative uses `(H_north-H_south)/((south_row-north_row) d_phi)`, which has the correct sign because image rows increase southward. The first and last pixel rows use radial normals instead of an ill-conditioned polar longitude derivative. Pixel centers never reach cos(phi) = 0, and this one-row cap additionally avoids amplified polar noise. This sacrifices a quarter-degree cap at each pole for this asset, not an entire sector or seam.

The CPU stores each object-space component as round(255(N_i/2 + 1/2)). The shader reverses this encoding and normalizes after filtering. Object-space storage avoids constructing a discontinuous tangent frame at the UV seam. RGB8 quantization has a component error of at most 1/255 after decoding, before renormalization. Sub-pixel relief, true silhouette relief, cast terrain shadows, and parallax are not represented.

The following APIs appear in the CPU implementation. Their definitions also apply to later source quotations. The [standard library](https://doc.rust-lang.org/std/) and [image 0.25 API](https://docs.rs/image/0.25.10/image/) are the API references.

| API | Inputs, result, edge conditions, and C++ equivalent |
| :-- | :-- |
| `f32::sqrt`, `sin`, `cos`, `tan` | Each takes the receiver float and returns the indicated float operation. Negative sqrt gives NaN; non-finite trigonometric arguments yield NaN. These correspond to `std::sqrt`, `std::sin`, `std::cos`, and `std::tan`. This code supplies finite angles and positive lengths. |
| Array `map` | Consumes a fixed-size array and a closure, returning an array of the same length with transformed elements. Here it does not allocate. C++ uses a fixed-size array and a loop. |
| `std::array::from_fn` | Calls a closure with each index in ascending order and constructs a fixed-size array. Here type inference selects three components. C++ uses an indexed initialization loop. |
| `f32::from` | Converts a u16 integer to f32 without losing precision, since all u16 values fit exactly in f32. C++ uses `static_cast<float>`. This is a value conversion, not a bit reinterpretation. |
| `Vec::new`, `extend_from_slice`, `extend` | `new` returns an empty vector. `extend_from_slice` appends cloned slice elements; `extend` appends iterator elements, including elements of an array. Empty inputs add nothing. C++ uses `std::vector` construction and range insertion. Allocation failure is not converted into our String errors. |
| `usize::checked_mul` | Returns `Some(product)` or `None` on overflow, like checking before multiplying `size_t`. It validates sample dimensions before indexing. |
| `usize::saturating_sub` | Subtracts but clamps unsigned underflow to zero, unlike ordinary C++ unsigned subtraction. It selects the first north neighbor safely. |
| `min`, `round`, `abs` | `min` picks the smaller bound; `round` rounds halfway cases away from zero; `abs` returns magnitude. The float casts to u8 saturate out-of-range values and map NaN to zero, rather than using C++ out-of-range floating conversion semantics. Our normalized finite components are already in range. See the existing [cast discussion](02-code.md#s0). |
| `assert!`, `assert_eq!`, `assert_ne!` | These macros test a condition, equality, or inequality and panic on failure. They remain active in release builds, unlike a disabled C++ `assert`. The private mesh function asserts its minimum tessellation; production uses constants satisfying it. |
| `iter`, `all`, `enumerate`, `len`, slice indexing | `iter` borrows elements, `all` tests a predicate with early exit and is true for an empty iterator, and `enumerate` yields zero-based indices with items. `len` returns usize. Indexing is bounds checked and panics outside the slice. C++ equivalents are const iterators, `std::all_of`, an index loop, `.size()`, and checked access. |
| `is_err`, `unwrap` | These test a Result for failure or extract its successful value, panicking on error. Here unwrap appears only in tests. See [Result](https://doc.rust-lang.org/std/result/enum.Result.html). |

`direction` constructs er from latitude and longitude. `normalize` divides a nonzero three-vector by its Euclidean length; zero input would produce NaNs, but the caller's Nraw has radial component one. `sphere` appends row-major vertex records, then cell triangles. `normal_map` rejects empty, undersized, mismatched, and overflowing dimensions, derives slopes at each pixel center, applies polar caps, and returns packed normals. `perspective` returns the camera matrix, whose positive aspect precondition is checked by the callback.

The mesh test verifies counts, valid indices, coincident seam positions, east/north orientation, and outward winding. The normal test verifies flat metric data produce radial normals within quantization error, a known metric eastward height difference gives the analytically expected tilted normal, longitude neighbors wrap, polar caps remain stable, and invalid dimensions return errors. The projection test checks landscape and portrait aspect scaling and both clipping-plane endpoints. The complete mathematical implementation and these tests are quoted in Section 8.

## 5. The callback, ownership, and graphics API

`Globe` retains shared renderer handles and an optional start time. `Arc<T>` is an atomically reference-counted owner, comparable to `std::shared_ptr<T>`. `Arc::new(value)` allocates shared storage; `Arc::clone(&owner)` increments its ownership count without cloning GPU data. `egui::mutex::Mutex<T>` wraps interior mutable access. `Mutex::new(value)` constructs it, and `lock()` returns a guard granting exclusive access until the guard is dropped. This corresponds to a mutex plus a scoped lock in C++. It does not make OpenGL callable from arbitrary threads.

The renderer stores program, shader, buffer, texture, and VAO handles, not the GL context. WebGL's context is not Send + Sync, the Rust marker traits for thread transfer and shared thread-safe access. The callback requires those traits even on this browser target. Keeping the context out of captured storage follows eframe's custom glow example and avoids pretending WebGL is thread-safe. Drawing obtains the actual current context from the callback's painter. No worker thread is created.

`CreationContext.gl` is an optional shared glow context, available when using eframe's glow backend. `MoonApp::new` returns a useful error if it is absent. The native and browser options request a 24-bit depth buffer, because eframe otherwise defaults to no depth attachment. The library's integration contract requires such a buffer. This is tested with the native app, not inferred from compilation alone. The native options also set `multisampling: 4`, which asks for 4x multisample anti-aliasing (MSAA): each pixel stores 4 coverage and depth samples, and the fragment shader still runs about once per pixel per triangle, so the extra cost is mainly framebuffer memory and the final resolve. It smooths the stepped edge of the sphere. The browser path has no such field in `WebOptions`. eframe 0.37 creates the context with `get_context("webgl2")` and no attributes, so the WebGL default `antialias: true` applies; whether the browser honors it is the browser's decision.

The UI first constructs a `PaintCallback`, which stores a rectangle and a reference-counted callback object. This does not draw immediately. Later, egui_glow binds its target framebuffer and configures the callback viewport and scissor rectangle before invoking `CallbackFn`. The callback locks the renderer, supplies the pixel aspect ratio, and calls `paint`. egui_glow 0.37 currently renders directly to the default framebuffer and disables desktop `FRAMEBUFFER_SRGB` during its UI setup. The renderer therefore writes sRGB-encoded output itself. This is verified against the versioned [painter source](https://docs.rs/egui_glow/0.37.0/src/egui_glow/painter.rs.html), especially `prepare_painting`, callback dispatch, and `intermediate_fbo`. A future egui renderer upgrade must revisit the framebuffer, depth, and color contracts.

After the callback, egui_glow restores its UI blend, cull, depth, viewport, and scissor setup. Our code additionally unbinds its VAO, program, and textures, restores texture unit zero, and disables depth and culling. It clears only depth, inside the existing scissor, so it does not erase other UI regions. The panel itself supplies black color. There is no offscreen framebuffer or hidden postprocessing pass.

### 5.1 Rust and egui API reference

These APIs precede their use in the source quotations. Existing `Result`, `Option`, `?`, `map_err`, `ok_or`, `as_ref`, `into`, `format!`, `from`, `rem_euclid`, `to_string`, `push`, `zip`, and iterator syntax follow [the original walkthrough](02-code.md). `format!("{error:#x}")` specifically formats a hexadecimal error with a `0x` prefix. `..Default::default()` updates selected struct fields while keeping every other field at its default.

| API | Behavior and C++ equivalent |
| :-- | :-- |
| `include_bytes!`, `include_str!` | These compile-time macros embed a byte slice or UTF-8 string relative to the Rust source file. Missing files are compile errors. They replace a runtime file read with linked static data. |
| `image::load_from_memory_with_format` | Accepts encoded bytes and an explicit ImageFormat and returns `Result<DynamicImage, ImageError>`. Invalid or truncated data returns an error. It does not silently fetch a codec. C++ image libraries similarly decode buffers into image objects. |
| `to_rgb8`, `as_luma16`, `as_raw`, `width`, `height` | `to_rgb8` returns an owned RGB8 conversion. `as_luma16` borrows the exact Luma16 variant, returning None for another type instead of reducing precision. ImageBuffer `as_raw` borrows the channel vector; width and height return u32 dimensions. These correspond to pixel conversion, a checked image-type query, raw storage access, and dimension getters. |
| `bytemuck::cast_slice` | Borrows the same storage as a differently typed slice with compile-time bit-validity restrictions and runtime size/alignment checks. It can panic on incompatible layouts. Casting f32/u32 slices to u8 here is aligned, exact, and requires no copies. C++ uses a byte view such as `std::as_bytes`. |
| `Option::get_or_insert`, `take` | The first inserts a supplied value only if absent and returns a mutable reference to the stored value. The second returns the old option and leaves None. These correspond to optional initialization and moving out followed by reset. |
| `Vec::drain(..)` | Removes the full vector range while yielding owned elements. Unconsumed elements are dropped when the drain iterator is dropped. It lets cleanup consume every handle without retaining stale values. C++ uses iteration followed by clearing. |
| `Ui::available_size`, `allocate_exact_size` | The first returns available logical-point dimensions. The second takes a size and interaction sense and reserves a rectangle, returning it with a Response. `Sense::hover()` requests hover interaction without a drag or click control. |
| `Rect::is_positive` | Returns true when width and height are both greater than zero. Empty, negative, or NaN dimensions are not useful render targets. It prevents a zero aspect division. |
| `Ui::input` | Gives a closure borrowed access to input state and returns the closure's result. `InputState.time` is seconds of monotonic input time, not civil UTC. C++ uses a callback or a const-reference input getter. |
| `CallbackFn::new` | Takes a `Fn(PaintCallbackInfo, &Painter) + Send + Sync + 'static` closure and returns the backend callback object. `move` owns the cloned Arc and angle so borrowed UI data cannot outlive its frame. |
| `viewport_in_pixels`, `Painter::gl` | The first converts the callback rectangle to framebuffer pixel coordinates, including DPI scaling. Width and height are i32. The second borrows the painter's Arc to the current glow context. Neither creates a new context. |
| `Ui::painter`, `Painter::add`, `Ui::ctx`, `Context::request_repaint` | These borrow the UI painter, append a paint shape, borrow the UI context, and request another frame. Repaint schedules work; it does not recursively invoke drawing. |
| `CentralPanel::default`, `frame`, `show` | These construct a panel, set its frame, and run a UI closure inside it. `Frame::NONE` has no panel decoration or margins; `fill(Color32::BLACK)` makes its color opaque black. The globe occupies its available rectangle. |
| `ShaderVersion::get`, `is_new_shader_interface`, `version_declaration` | These inspect GLSL support, report whether in/out syntax is available, and supply a version preamble. Version 0.37 returns `#version 140` on modern desktop GL and `#version 300 es` on WebGL2. GLSL 1.40 is supported by GL 3.3 core; no 3.3-only shader feature is required. Legacy GLSL or WebGL1 is rejected before resource creation. |

### 5.2 Every glow call and its C++ OpenGL equivalent

Creation methods return `Result<Handle, String>`; failures propagate to app startup. Other operations return unit unless indicated. Handle options use Some(handle) to bind and None to unbind. A handle is an identifier for a driver resource, not a Rust pointer to its bytes. The declarations are verified against [glow 0.17 HasContext](https://docs.rs/glow/0.17.0/glow/trait.HasContext.html) and its resolved source. All calls below require the same current context.

| glow call | Arguments and behavior | C++ OpenGL equivalent |
| :-- | :-- | :-- |
| `create_program` | Creates an empty linked-program container. | `glCreateProgram` |
| `create_shader` | Takes VERTEX_SHADER or FRAGMENT_SHADER and creates one stage. | `glCreateShader` |
| `shader_source` | Associates a stage with source text, including its version header. | `glShaderSource` |
| `compile_shader` | Compiles one stage; failure is queried separately. | `glCompileShader` |
| `get_shader_compile_status` | Returns bool for compilation success. | `glGetShaderiv(..., GL_COMPILE_STATUS, ...)` |
| `get_shader_info_log` | Returns the driver's compilation diagnostic String, possibly empty. | `glGetShaderInfoLog` |
| `attach_shader`, `detach_shader` | Associate or dissociate a shader and program. | `glAttachShader`, `glDetachShader` |
| `bind_attrib_location` | Assigns a named vertex input an integer location before linking. | `glBindAttribLocation` |
| `link_program` | Links attached stages into one executable. | `glLinkProgram` |
| `get_program_link_status`, `get_program_info_log` | Return link success and its diagnostic String. | `glGetProgramiv(..., GL_LINK_STATUS, ...)`, `glGetProgramInfoLog` |
| `create_vertex_array`, `bind_vertex_array` | Allocate a VAO and select it or unbind it. | `glGenVertexArrays`, `glBindVertexArray` |
| `create_buffer`, `bind_buffer` | Allocate storage identity and bind it to ARRAY_BUFFER or ELEMENT_ARRAY_BUFFER. The latter binding belongs to the current VAO. | `glGenBuffers`, `glBindBuffer` |
| `buffer_data_u8_slice` | Uploads the supplied byte slice into the bound target, with STATIC_DRAW as a usage hint, not an immutability restriction. | `glBufferData` |
| `enable_vertex_attrib_array` | Enables the indexed vertex input in the current VAO. | `glEnableVertexAttribArray` |
| `vertex_attrib_pointer_f32` | Specifies index, component count, scalar type, normalized flag, byte stride, and byte offset. Our FLOAT data need no integer normalization. | `glVertexAttribPointer` |
| `get_parameter_i32` | Returns an integer driver limit. MAX_TEXTURE_SIZE is the maximum texture side length. | `glGetIntegerv` |
| `create_texture`, `bind_texture` | Allocate a texture identity and bind it as TEXTURE_2D in the active texture unit. | `glGenTextures`, `glBindTexture` |
| `active_texture` | Selects TEXTURE0 plus an integer unit index for subsequent texture commands. | `glActiveTexture` |
| `pixel_store_i32` | Sets UNPACK_ALIGNMENT, the row-start alignment for uploaded pixels. We use one byte for RGB and restore the standard value four. | `glPixelStorei` |
| `tex_image_2d` | Supplies target, mip level zero, RGB8 internal format, width, height, border zero, external RGB, UNSIGNED_BYTE, and a borrowed pixel slice. PixelUnpackData::Slice(Some(...)) means CPU memory, not a pixel-buffer offset. | `glTexImage2D` |
| `tex_parameter_i32` | Sets linear minification/magnification, horizontal REPEAT, or vertical CLAMP_TO_EDGE. These avoid a missing-mipmap texture and a longitude-edge gap. | `glTexParameteri` |
| `get_error` | Returns and removes one GL error code. NO_ERROR is success. This detects upload errors, not every possible future rendering error. | `glGetError` |
| `enable`, `disable` | Turn DEPTH_TEST, BLEND, or CULL_FACE on or off. | `glEnable`, `glDisable` |
| `depth_func`, `depth_mask` | Select LESS comparison and enable depth writes with true. | `glDepthFunc`, `glDepthMask` |
| `clear_depth_f32`, `clear` | Set the depth-clear value to one and clear DEPTH_BUFFER_BIT inside the scissor rectangle. Color is not cleared here. | `glClearDepth` or ES `glClearDepthf`, `glClear` |
| `cull_face`, `front_face` | Select BACK faces for rejection and CCW as the front-face winding. | `glCullFace`, `glFrontFace` |
| `use_program` | Binds the linked executable or unbinds with None. | `glUseProgram` |
| `get_uniform_location` | Looks up a named active uniform, returning None if absent or optimized away. | `glGetUniformLocation`, with -1 represented as None |
| `uniform_matrix_4_f32_slice` | Uploads column-major 4 by 4 matrices with transpose false. A None location is ignored. | `glUniformMatrix4fv` |
| `uniform_1_f32`, `uniform_1_i32` | Set a scalar float or integer. The sampler integer is a texture unit number, not TEXTURE0's enum value. A None location is ignored. | `glUniform1f`, `glUniform1i` |
| `draw_elements` | Takes TRIANGLES, the index count, UNSIGNED_INT index format, and byte offset zero into the VAO's IBO. | `glDrawElements` |
| `delete_texture`, `delete_buffer`, `delete_vertex_array`, `delete_shader`, `delete_program` | Delete the supplied resource handle. Cleanup drains collections and takes options so repeat cleanup never reuses those handles. | `glDeleteTextures`, `glDeleteBuffers`, `glDeleteVertexArrays`, `glDeleteShader`, `glDeleteProgram` |

`Globe::new` constructs the shared renderer only after initialization succeeds. `Globe::show` reserves the panel region, establishes a start time once, schedules the deferred draw, and requests continued animation. `Globe::destroy` delegates cleanup on the owning context. `Renderer::new` decodes embedded assets before allocating GL resources, rejects unsupported shader syntax, creates and links stages, uploads the mesh and maps, and propagates diagnostics. A Result-returning initialization closure ensures any error after partial GPU allocation reaches explicit cleanup. Successfully linked shader stages are detached and deleted immediately.

`upload_texture` checks device dimensions and selects texture unit zero explicitly. It allocates RGB8 storage and configures filtering and wrapping. `paint` returns harmlessly if the program has already been destroyed. Otherwise it clears depth, binds the program, projection, rotation, and two textures, draws the mesh, and restores its bindings. `destroy` empties all resource collections and sets optional handles to None. It is idempotent. There is deliberately no GL-calling Drop implementation: dropping a Rust object does not prove that its GL context is current. `MoonApp::on_exit` performs explicit destruction before eframe tears the context down, as in the official example. A caller embedding `Globe` elsewhere must honor this same lifecycle.

## 6. Photometry, shaders, and color

Let N be the unit shading normal, S the unit direction from the surface toward the Sun, and V the unit direction toward the camera. Incidence i is the angle between N and S. Emission e is the angle between N and V. Phase alpha is the angle between S and V, so cos(alpha) = S dot V. Define mu0 = max(N dot S, 0) and mu = max(N dot V, 0). The Sun direction is normalize(1.0, 0.2, 0.6), fixed in camera/world coordinates while the Moon rotates. Its obliquity leaves a visible terminator, the boundary between illuminated and unilluminated terrain. V varies across the perspective view, rather than being one constant direction.

Lambert's diffuse disk term is mu0. The normalized Lommel-Seeliger disk term is 2 mu0/(mu0+mu). The latter approximates single scattering in a particulate surface and differs from a conventional smooth diffuse sphere. Lunar-Lambert combines these disk functions with a weight L(alpha):

```text
r = A [2 L(alpha) mu0/(mu0+mu) + (1-L(alpha)) mu0].
```

A is the linear color-map value here. At normal incidence and emission, both disk terms equal one. Choosing L = 0 gives Lambert; choosing L = 1 gives Lommel-Seeliger. This is an empirical reflectance model, not a derivation of the microscopic scattering law. The implementation explicitly returns zero when incidence or emission is nonpositive, and guards the denominator by 0.00001 at grazing angles. It adds no ambient or earthshine term. It does not ray-trace terrain shadows, model opposition surge, or claim absolute energy calibration.

The source is **McEwen, A.S. (1991), "Photometric functions for photoclinometry and other applications," Icarus 92(2), 298-311, DOI [10.1016/0019-1035(91)90053-V](https://doi.org/10.1016/0019-1035(91)90053-V)**. The bibliographic record is verified at [USGS](https://pubs.usgs.gov/publication/70016679). The [USGS ISIS phoempglobal documentation](https://isis.astrogeology.usgs.gov/8.2.0/Application/presentation/Tabbed/phoempglobal/phoempglobal.html) explicitly attributes `b(phase) * ((1-l(phase))*u0 + 2*l(phase)*u0/(u0+u))` to McEwen. That formula includes an empirical phase-brightness multiplier b. Here the texture and exposure set brightness, and b is implicitly one.

No verified body-specific phase polynomial has been adopted. The code deliberately uses the simple constant choice **L(alpha) = 0.5**, independent of alpha. It is an illustrative mixture, not published lunar polynomial coefficients and not a fit to the Moon's actual phase curve. This qualification matters when describing the result as physically based: its geometry, metric normals, disk-function structure, and linear-light arithmetic have physical foundations, but its display albedo, exposure, fixed light, and constant mixture are artistic approximations. The NASA color map is also optimized for aesthetics. Do not infer irradiance or power from the display.

Let c be an sRGB channel between zero and one. The standard decoding is c/12.92 if c <= 0.04045, otherwise ((c+0.055)/1.055)^2.4. We decode the sampled JPEG color before multiplying by reflectance. The normal texture is never color-decoded. Filtering the JPEG occurs in its stored encoded space before decoding, a small approximation accepted for this renderer; exact linear-light filtering would require linearized upload or hardware sRGB texture storage. The shader uses an exposure multiplier of 2.0, meaning a factor of two, not an additive offset.

For exposed linear channel x >= 0, tone mapping uses x/(1+x), which approaches one without clipping finite positive values. This is the simple global Reinhard operator associated with **Reinhard, Stark, Shirley, and Ferwerda (2002), "Photographic Tone Reproduction for Digital Images," ACM Transactions on Graphics 21(3), 267-276, DOI [10.1145/566654.566575](https://doi.org/10.1145/566654.566575)**. We use only its simple per-channel compression, not the full paper's log-average exposure, local adaptation, or luminance-preserving implementation. Per-channel compression can alter hue at high values.

The inverse transfer encodes x as 12.92x if x <= 0.0031308, otherwise 1.055 x^(1/2.4) - 0.055. The shader outputs that sRGB value with alpha one. It does not rely on automatic framebuffer conversion. These transfer functions are specified by the [Khronos sRGB extension](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_sRGB.txt), matching IEC sRGB. Applying both explicit encoding and hardware framebuffer sRGB would incorrectly encode twice; egui_glow's current state avoids that.

GLSL `vec2`, `vec3`, and `vec4` are two-, three-, and four-component float vectors; `mat4` is a four-column float matrix. Constructors assemble components or repeat a scalar. Vector arithmetic is componentwise except matrix multiplication, which performs the stated linear transformation. `in` and `out` connect vertex inputs, interpolated stage outputs, and fragment output; `uniform` declares a draw-constant parameter. `precision highp float` requests high float precision on ES and is also valid in the supported desktop language. GLSL's `void main()` is the stage entry point, and `gl_Position` is the built-in clip-space output.

`texture(sampler, uv)` returns filtered texture channels. `normalize` divides a vector by its length. `dot` computes the sum of component products. `cos`, `sin`, `pow`, and `max` compute the corresponding functions componentwise when given vectors. `step(edge, x)` is zero below edge and one otherwise. `mix(a, b, t)` computes a(1-t)+bt, so step selects the sRGB branch without a per-channel if statement. The two branch formulas coincide at the breakpoint to the standard's rounding accuracy. All power bases used here are nonnegative. Fragment outputs outside the sphere never run; the existing black panel remains visible.

`moon.vert` rotates each object position, retains the rotated world position and UV for interpolation, translates relative to C, and projects it. `moon.frag` decodes and rotates the object-space normal, derives viewing and lighting cosines, evaluates the constant-weight Lunar-Lambert mixture, decodes color, applies exposure and tone mapping, and encodes the final output. Section 8 quotes both files exactly; their version headers are prepended at initialization rather than stored in those files.

## 7. App integration and operating the renderer

`MoonApp` owns one `Globe`. Its constructor converts a missing glow context or failed renderer initialization into an application startup error. Its `ui` method replaces the original placeholder with a black, margin-free CentralPanel. Its `on_exit` method releases GPU resources while the GL context is current. The native entry point retains the title "Moon Landing Visual". The browser entry point retains its canvas lookup and asynchronous WebRunner, requesting the same depth buffer. `app/index.html` retains the title "Karaya Explore" unchanged. No deployment workflow or hosting dependency is introduced.

On Gentoo, install a working OpenGL driver for the GPU and the normal native build prerequisites for eframe's X11 or Wayland backend. Mesa's software renderer is useful for headless verification, not a requirement for normal operation. Rust commands are unchanged except for the new optional globe tests:

```sh
cargo run -p app
cargo test -p globe
cargo test -p lunar_core
cargo build -p app
cargo clippy --workspace
rustup target add wasm32-unknown-unknown
cargo check -p app --target wasm32-unknown-unknown
cargo +nightly fmt --all
```

The WASM target needs a browser with WebGL2. Its compilation check does not establish that every browser and GPU combination works. No SharedArrayBuffer, threading headers, server, or network fetch is needed for assets. Initial decoding and normal generation happen synchronously at startup; a black window may precede the first completed frame in a debug build. Repaint drives continuous animation after initialization, so an open window uses rendering resources continuously.

The Python conversion script is optional tooling. It opens the NASA URL with urllib.request, reads the response into BytesIO, lets Pillow decode the TIFF, asserts its dimensions and unsigned 16-bit mode, and saves beside the script using `Path(__file__).with_name`. The context manager closes the response; the temporary TIFF exists only in memory. The committed PNG is sufficient for all Rust builds.

## 8. Exact source quotations

The preceding sections explain each function's operations and APIs in dependency order. The following listings preserve the full implementation, including tests and lifecycle handling, so no statement is hidden by an abbreviated example.

### 8.1 globe/Cargo.toml

```toml
[package]
name = "globe"
version = "0.1.0"
edition = "2024"

[dependencies]
egui = "0.37"
egui_glow = "0.37"
bytemuck = "1.25"
image = { version = "0.25", default-features = false, features = ["jpeg", "png"] }
```

### 8.2 globe/src/math.rs

```rust
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub(crate) const RADIUS_M: f32 = 1_737_400.0;
pub(crate) const EXAGGERATION: f32 = 2.0;

pub(crate) fn direction(lat: f32, lon: f32) -> [f32; 3]
{
	[lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

pub(crate) fn normalize(v: [f32; 3]) -> [f32; 3]
{
	let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
	v.map(|x| x / length)
}

/// Interleaved object position and UV, followed by counterclockwise triangle indices.
pub(crate) fn sphere(slices: u32, stacks: u32) -> (Vec<f32>, Vec<u32>)
{
	assert!(slices >= 3 && stacks >= 2);
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	for row in 0..=stacks
	{
		let v = row as f32 / stacks as f32;
		for col in 0..=slices
		{
			let u = col as f32 / slices as f32;
			let p = direction(FRAC_PI_2 - PI * v, TAU * (u - 0.5));
			vertices.extend_from_slice(&[p[0], p[1], p[2], u, v]);
		}
	}
	for row in 0..stacks
	{
		for col in 0..slices
		{
			let a = row * (slices + 1) + col;
			let b = a + slices + 1;
			indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
		}
	}
	(vertices, indices)
}

/// NASA unsigned heights are half-meters above a 1,727,400 m reference radius.
/// Differentiate metric relief on the sphere, then encode object-space normals as RGB8.
pub(crate) fn normal_map(heights: &[u16], w: usize, h: usize) -> Result<Vec<u8>, String>
{
	if w < 4 || h < 2 || w.checked_mul(h) != Some(heights.len())
	{
		return Err("Invalid height-map dimensions".into());
	}
	let mut normals = Vec::new();
	let d_lon = TAU / w as f32;
	let d_lat = PI / h as f32;
	for y in 0..h
	{
		let lat = FRAC_PI_2 - (y as f32 + 0.5) * d_lat;
		let north = y.saturating_sub(1);
		let south = (y + 1).min(h - 1);
		for x in 0..w
		{
			let lon = ((x as f32 + 0.5) / w as f32 - 0.5) * TAU;
			let radial = direction(lat, lon);
			let height = |x, y| (f32::from(heights[y * w + x]) * 0.5 - 10_000.0) * EXAGGERATION;
			let radius = RADIUS_M + height(x, y);
			let east_slope = (height((x + 1) % w, y) - height((x + w - 1) % w, y))
				/ (2.0 * d_lon * radius * lat.cos());
			let north_slope =
				(height(x, north) - height(x, south)) / ((south - north) as f32 * d_lat * radius);
			let east = [lon.cos(), 0.0, -lon.sin()];
			let north_dir = [-lat.sin() * lon.sin(), lat.cos(), -lat.sin() * lon.cos()];
			// The longitude basis is singular at a pole. Use a radial cap for one pixel row.
			let n = if y == 0 || y == h - 1
			{
				radial
			}
			else
			{
				normalize(std::array::from_fn(|i| {
					radial[i] - east_slope * east[i] - north_slope * north_dir[i]
				}))
			};
			normals.extend(n.map(|v| ((v * 0.5 + 0.5) * 255.0).round() as u8));
		}
	}
	Ok(normals)
}

/// Column-major, right-handed projection: 45-degree field on the shorter axis, near 0.1, far 10.
pub(crate) fn perspective(aspect: f32) -> [f32; 16]
{
	let f = aspect.min(1.0) / (PI / 8.0).tan();
	let near = 0.1;
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

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn mesh_orientation_and_seam()
	{
		let (v, i) = sphere(128, 64);
		assert_eq!(v.len(), 129 * 65 * 5);
		assert_eq!(i.len(), 128 * 64 * 6);
		assert!(i.iter().all(|&index| (index as usize) < v.len() / 5));
		for row in 0..=64
		{
			for axis in 0..3
			{
				assert!((v[row * 129 * 5 + axis] - v[(row * 129 + 128) * 5 + axis]).abs() < 1e-6);
			}
		}
		assert!(direction(0.0, 0.0)[2] > 0.99);
		assert!(direction(0.0, FRAC_PI_2)[0] > 0.99);
		assert!(direction(-0.75, -0.2)[1] < 0.0);
		// A middle-row triangle must face outwards, towards the camera at longitude zero.
		let a = &v[(31 * 129 + 64) * 5..];
		let b = &v[(32 * 129 + 64) * 5..];
		let c = &v[(31 * 129 + 65) * 5..];
		assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
	}

	#[test]
	fn normals_metric_slopes_caps_and_invalid_input()
	{
		assert!(normal_map(&[], 0, 0).is_err());
		assert!(normal_map(&[0; 8], 4, 3).is_err());
		assert!(normal_map(&[], usize::MAX, 2).is_err());
		let w = 16;
		let h = 8;
		let mut heights = vec![20_000; w * h];
		let flat = normal_map(&heights, w, h).unwrap();
		for y in 0..h
		{
			for x in 0..w
			{
				let expected = direction(
					FRAC_PI_2 - (y as f32 + 0.5) * PI / h as f32,
					((x as f32 + 0.5) / w as f32 - 0.5) * TAU,
				);
				for axis in 0..3
				{
					let decoded = f32::from(flat[(y * w + x) * 3 + axis]) / 255.0 * 2.0 - 1.0;
					assert!((decoded - expected[axis]).abs() < 0.004);
				}
			}
		}
		heights[4 * w + 9] += 10_000;
		let slopes = normal_map(&heights, w, h).unwrap();
		assert_ne!(
			&flat[(4 * w + 8) * 3..(4 * w + 9) * 3],
			&slopes[(4 * w + 8) * 3..(4 * w + 9) * 3]
		);
		assert_eq!(&flat[..w * 3], &slopes[..w * 3]);
		let lat = -PI / 16.0;
		let lon = TAU / 32.0;
		let radial = direction(lat, lon);
		let east = [lon.cos(), 0.0, -lon.sin()];
		let slope = 10_000.0 / (2.0 * TAU / w as f32 * RADIUS_M * lat.cos());
		let expected = normalize(std::array::from_fn(|i| radial[i] - slope * east[i]));
		for axis in 0..3
		{
			let decoded = f32::from(slopes[(4 * w + 8) * 3 + axis]) / 255.0 * 2.0 - 1.0;
			assert!((decoded - expected[axis]).abs() < 0.004);
		}
		heights[4 * w] += 10_000;
		let wrapped = normal_map(&heights, w, h).unwrap();
		assert_ne!(
			&flat[(5 * w - 1) * 3..5 * w * 3],
			&wrapped[(5 * w - 1) * 3..5 * w * 3]
		);
	}

	#[test]
	fn projection_depth_and_aspect()
	{
		let p = perspective(2.0);
		assert!((p[5] / p[0] - 2.0).abs() < 1e-6);
		let portrait = perspective(0.5);
		assert!((portrait[0] - p[5]).abs() < 1e-6);
		for (z, expected) in [(-0.1, -1.0), (-10.0, 1.0)]
		{
			assert!(((p[10] * z + p[14]) / -z - expected).abs() < 1e-5);
		}
	}
}
```

### 8.3 globe/src/lib.rs

```rust
//! An illustrative lunar renderer, independent of the landing-site scientific library.
//! See docs/03-moon.md for coordinate, photometric, color, and GL lifetime contracts.

mod math;

use egui::mutex::Mutex;
use egui_glow::glow::{self, HasContext as _};
use std::sync::Arc;

/// Owns the callback's shared renderer. Call destroy from eframe's on_exit with a current context.
pub struct Globe
{
	renderer: Arc<Mutex<Renderer>>,
	started: Option<f64>,
}

impl Globe
{
	pub fn new(gl: &Arc<glow::Context>) -> Result<Self, String>
	{
		Ok(Self {
			renderer: Arc::new(Mutex::new(Renderer::new(gl)?)),
			started: None,
		})
	}

	/// Fill the available UI rectangle and rotate once per 60 seconds of egui time.
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
		if !rect.is_positive()
		{
			return;
		}
		let time = ui.input(|input| input.time);
		let start = *self.started.get_or_insert(time);
		let angle = ((time - start).rem_euclid(60.0) / 60.0) as f32 * std::f32::consts::TAU - 0.5;
		let renderer = Arc::clone(&self.renderer);
		let callback = egui_glow::CallbackFn::new(move |info, painter| {
			let viewport = info.viewport_in_pixels();
			if viewport.width_px > 0 && viewport.height_px > 0
			{
				renderer.lock().paint(
					painter.gl(),
					viewport.width_px as f32 / viewport.height_px as f32,
					angle,
				);
			}
		});
		ui.painter().add(egui::PaintCallback {
			rect,
			callback: Arc::new(callback),
		});
		ui.ctx().request_repaint();
	}

	pub fn destroy(&mut self, gl: &glow::Context)
	{
		self.renderer.lock().destroy(gl);
	}
}

struct Renderer
{
	program: Option<glow::Program>,
	shaders: Vec<glow::Shader>,
	vao: Option<glow::VertexArray>,
	buffers: Vec<glow::Buffer>,
	textures: Vec<glow::Texture>,
	index_count: i32,
}

impl Renderer
{
	fn new(gl: &Arc<glow::Context>) -> Result<Self, String>
	{
		let albedo = image::load_from_memory_with_format(
			include_bytes!("../assets/lroc_color_2k.jpg"),
			image::ImageFormat::Jpeg,
		)
		.map_err(|e| e.to_string())?
		.to_rgb8();
		let height = image::load_from_memory_with_format(
			include_bytes!("../assets/ldem_4_uint.png"),
			image::ImageFormat::Png,
		)
		.map_err(|e| e.to_string())?;
		let height = height
			.as_luma16()
			.ok_or("Expected a 16-bit grayscale LOLA map")?;
		let normals = math::normal_map(
			height.as_raw(),
			height.width() as usize,
			height.height() as usize,
		)?;
		let version = egui_glow::ShaderVersion::get(gl);
		if !version.is_new_shader_interface()
		{
			return Err("The Moon requires modern OpenGL or WebGL2".into());
		}
		let mut renderer = Self {
			program: None,
			shaders: Vec::new(),
			vao: None,
			buffers: Vec::new(),
			textures: Vec::new(),
			index_count: 0,
		};
		let initialized = (|| -> Result<(), String> {
			// SAFETY: eframe calls the constructor with its GL context current. Every handle is
			// registered immediately so the error branch can clean partial construction.
			unsafe {
				let program = gl.create_program()?;
				renderer.program = Some(program);
				for (kind, source) in [
					(glow::VERTEX_SHADER, include_str!("moon.vert")),
					(glow::FRAGMENT_SHADER, include_str!("moon.frag")),
				]
				{
					let shader = gl.create_shader(kind)?;
					renderer.shaders.push(shader);
					gl.shader_source(
						shader,
						&format!("{}\n{source}", version.version_declaration()),
					);
					gl.compile_shader(shader);
					if !gl.get_shader_compile_status(shader)
					{
						return Err(gl.get_shader_info_log(shader));
					}
					gl.attach_shader(program, shader);
				}
				gl.bind_attrib_location(program, 0, "a_position");
				gl.bind_attrib_location(program, 1, "a_uv");
				gl.link_program(program);
				if !gl.get_program_link_status(program)
				{
					return Err(gl.get_program_info_log(program));
				}
				for shader in renderer.shaders.drain(..)
				{
					gl.detach_shader(program, shader);
					gl.delete_shader(shader);
				}
				let (vertices, indices) = math::sphere(128, 64);
				renderer.index_count = indices.len() as i32;
				renderer.vao = Some(gl.create_vertex_array()?);
				gl.bind_vertex_array(renderer.vao);
				for (target, bytes) in [
					(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices)),
					(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&indices)),
				]
				{
					let buffer = gl.create_buffer()?;
					renderer.buffers.push(buffer);
					gl.bind_buffer(target, Some(buffer));
					gl.buffer_data_u8_slice(target, bytes, glow::STATIC_DRAW);
				}
				gl.enable_vertex_attrib_array(0);
				gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 20, 0);
				gl.enable_vertex_attrib_array(1);
				gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 20, 12);
				gl.bind_vertex_array(None);
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
			renderer.upload_texture(gl, albedo.width(), albedo.height(), albedo.as_raw())?;
			renderer.upload_texture(gl, height.width(), height.height(), &normals)?;
			Ok(())
		})();
		if let Err(error) = initialized
		{
			renderer.destroy(gl);
			return Err(error);
		}
		Ok(renderer)
	}

	fn upload_texture(
		&mut self,
		gl: &glow::Context,
		width: u32,
		height: u32,
		pixels: &[u8],
	) -> Result<(), String>
	{
		// SAFETY: the context is current during initialization; decoded RGB storage is
		// width * height * 3 bytes. RGB8 is intentionally linear storage, with shader decoding.
		unsafe {
			let max_side = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32;
			if width > max_side || height > max_side
			{
				return Err("Moon texture exceeds the device texture-size limit".into());
			}
			gl.active_texture(glow::TEXTURE0);
			let texture = gl.create_texture()?;
			self.textures.push(texture);
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGB8 as i32,
				width as i32,
				height as i32,
				0,
				glow::RGB,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(pixels)),
			);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR as i32,
			);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as i32,
			);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_WRAP_T,
				glow::CLAMP_TO_EDGE as i32,
			);
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
			gl.bind_texture(glow::TEXTURE_2D, None);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Moon texture upload GL error: {error:#x}"));
			}
		}
		Ok(())
	}

	fn paint(&self, gl: &glow::Context, aspect: f32, angle: f32)
	{
		let Some(program) = self.program
		else
		{
			return;
		};
		// SAFETY: CallbackFn runs on eframe's current GL context, after its viewport/scissor
		// setup. The app requests a 24-bit depth buffer on native and web. egui_glow 0.37
		// draws to the default framebuffer, disables framebuffer-sRGB, and restores UI state.
		unsafe {
			gl.enable(glow::DEPTH_TEST);
			gl.depth_func(glow::LESS);
			gl.depth_mask(true);
			gl.clear_depth_f32(1.0);
			gl.clear(glow::DEPTH_BUFFER_BIT);
			gl.disable(glow::BLEND);
			gl.enable(glow::CULL_FACE);
			gl.cull_face(glow::BACK);
			gl.front_face(glow::CCW);
			gl.use_program(Some(program));
			gl.uniform_matrix_4_f32_slice(
				gl.get_uniform_location(program, "u_projection").as_ref(),
				false,
				&math::perspective(aspect),
			);
			gl.uniform_1_f32(gl.get_uniform_location(program, "u_angle").as_ref(), angle);
			for (unit, name) in ["u_albedo", "u_normal"].iter().enumerate()
			{
				gl.active_texture(glow::TEXTURE0 + unit as u32);
				gl.bind_texture(glow::TEXTURE_2D, Some(self.textures[unit]));
				gl.uniform_1_i32(gl.get_uniform_location(program, name).as_ref(), unit as i32);
			}
			gl.bind_vertex_array(self.vao);
			gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
			gl.bind_vertex_array(None);
			for unit in 0..2
			{
				gl.active_texture(glow::TEXTURE0 + unit);
				gl.bind_texture(glow::TEXTURE_2D, None);
			}
			gl.active_texture(glow::TEXTURE0);
			gl.use_program(None);
			gl.disable(glow::DEPTH_TEST);
			gl.disable(glow::CULL_FACE);
		}
	}

	fn destroy(&mut self, gl: &glow::Context)
	{
		// SAFETY: called with the owning context current, either on_exit or failed construction.
		// Taking/draining handles makes repeated cleanup harmless.
		unsafe {
			for texture in self.textures.drain(..)
			{
				gl.delete_texture(texture);
			}
			for buffer in self.buffers.drain(..)
			{
				gl.delete_buffer(buffer);
			}
			if let Some(vao) = self.vao.take()
			{
				gl.delete_vertex_array(vao);
			}
			for shader in self.shaders.drain(..)
			{
				gl.delete_shader(shader);
			}
			if let Some(program) = self.program.take()
			{
				gl.delete_program(program);
			}
		}
	}
}
```

### 8.4 globe/src/moon.vert

```glsl
precision highp float;
in vec3 a_position;
in vec2 a_uv;
uniform mat4 u_projection;
uniform float u_angle;
out vec3 v_position;
out vec2 v_uv;

void main()
{
	float c = cos(u_angle);
	float s = sin(u_angle);
	vec3 p = vec3(c * a_position.x + s * a_position.z, a_position.y,
		-s * a_position.x + c * a_position.z);
	v_position = p;
	v_uv = a_uv;
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, 3.3), 1.0);
}
```

### 8.5 globe/src/moon.frag

```glsl
precision highp float;
in vec3 v_position;
in vec2 v_uv;
uniform sampler2D u_albedo;
uniform sampler2D u_normal;
uniform float u_angle;
out vec4 out_color;

vec3 srgb_to_linear(vec3 x)
{
	return mix(x / 12.92, pow((x + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), x));
}

vec3 linear_to_srgb(vec3 x)
{
	return mix(x * 12.92, 1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), x));
}

void main()
{
	vec3 n = normalize(texture(u_normal, v_uv).rgb * 2.0 - 1.0);
	float c = cos(u_angle);
	float s = sin(u_angle);
	n = normalize(vec3(c * n.x + s * n.z, n.y, -s * n.x + c * n.z));
	vec3 light = normalize(vec3(1.0, 0.2, 0.6));
	vec3 view = normalize(vec3(0.0, 0.0, 3.3) - v_position);
	float mu0 = max(dot(n, light), 0.0);
	float mu = max(dot(n, view), 0.0);
	// Constant L(alpha)=0.5 is an explicit illustrative mixture, not a fitted lunar phase law.
	float lunar_weight = 0.5;
	float reflectance = 0.0;
	if (mu0 > 0.0 && mu > 0.0)
	{
		reflectance = 2.0 * lunar_weight * mu0 / max(mu0 + mu, 0.00001)
			+ (1.0 - lunar_weight) * mu0;
	}
	vec3 albedo = srgb_to_linear(texture(u_albedo, v_uv).rgb);
	vec3 radiance = 2.0 * albedo * reflectance;
	// Reinhard tone mapping; no ambient or earthshine contribution.
	vec3 mapped = radiance / (vec3(1.0) + radiance);
	out_color = vec4(linear_to_srgb(mapped), 1.0);
}
```

### 8.6 app/src/lib.rs

```rust
pub struct MoonApp
{
	globe: globe::Globe,
}

impl MoonApp
{
	pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, String>
	{
		let gl = cc
			.gl
			.as_ref()
			.ok_or("The Moon requires the glow renderer")?;
		Ok(Self {
			globe: globe::Globe::new(gl)?,
		})
	}
}

impl eframe::App for MoonApp
{
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK))
			.show(ui, |ui| self.globe.show(ui));
	}

	fn on_exit(&mut self, gl: Option<&eframe::glow::Context>)
	{
		if let Some(gl) = gl
		{
			self.globe.destroy(gl);
		}
	}
}
```

### 8.7 app/src/main.rs

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result
{
	env_logger::init();
	eframe::run_native(
		"Moon Landing Visual",
		eframe::NativeOptions {
			depth_buffer: 24,
			multisampling: 4,
			..Default::default()
		},
		Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
	)
}

#[cfg(target_arch = "wasm32")]
fn main()
{
	use eframe::wasm_bindgen::JsCast as _;
	wasm_bindgen_futures::spawn_local(async {
		let document = web_sys::window()
			.expect("No window")
			.document()
			.expect("No document");
		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("Missing canvas")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("Not a canvas");
		let result = eframe::WebRunner::new()
			.start(
				canvas,
				eframe::WebOptions {
					depth_buffer: 24,
					..Default::default()
				},
				Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
			)
			.await;
		if let Some(loading) = document.get_element_by_id("loading_text")
		{
			match result
			{
				Ok(()) => loading.remove(),
				Err(err) =>
				{
					loading.set_text_content(Some(
						"The app could not start. See the developer console.",
					));
					panic!("Failed to start eframe: {err:?}");
				}
			}
		}
	});
}
```

### 8.8 globe/assets/convert_height.py

```python
"""Download NASA's original unsigned LOLA TIFF and losslessly encode its samples as PNG.

Run with Python 3 and Pillow: python3 globe/assets/convert_height.py
The source TIFF stays in memory, so it is never accidentally committed.
"""
import io
import pathlib
import urllib.request

from PIL import Image

URL = "https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4_uint.tif"
with urllib.request.urlopen(URL, timeout=120) as response:
    source = Image.open(io.BytesIO(response.read()))
    assert source.size == (1440, 720) and source.mode == "I;16"
    source.save(pathlib.Path(__file__).with_name("ldem_4_uint.png"))
```


## 9. Verification boundaries and extensions

The tests cover the mesh seam and orientation, radial and perturbed normals with metric units, malformed dimensions, and camera projection. Native screenshot verification must additionally inspect recognizable near-side landmarks, lighting, poles, and longitudinal edges: successful compilation alone does not validate those properties. Two images separated by about 10 seconds should show about 60 degrees of model rotation while the Sun direction remains fixed. Save screenshots outside the repository.

This renderer does not load scientific polar tiles or consume the prep JSON. Its global rendering height map is intentionally separate from future scientific DEM ingestion. True displacement, terrain shadows, physically fitted phase coefficients, radiometric calibration, user camera controls, texture mipmaps, and integration with actual ephemerides are possible extensions, not implemented features. Recheck callback framebuffer assumptions when upgrading egui_glow.

The native verification run used Xvfb with Mesa 23.2.1, llvmpipe, and OpenGL 4.5 core. Its first rendered capture was saved at 01:11:25 UTC on November 15, 2026, and the second at 01:11:35 UTC, about two seconds after process launch and 10 seconds apart. The screenshots at `/home/ubuntu/plan/globe.png` and `/home/ubuntu/plan/globe2.png` show near-side landmarks in the correct hemispheres, plausible illumination, and visible rotation. Additional far-side captures showed no longitude seam. The WASM target was compile-checked, not rendered in a browser. The conversion check compared all 1,036,800 decoded PNG height samples with the source TIFF without differences.

## 10. Alphabetical API index

The generated index below covers method calls, macros, and qualified function calls in the renderer and changed app Rust sources. Section 4 explains CPU operations, Section 5 explains Rust integration, Section 5.2 explains every GL command with its C++ equivalent, and Section 6 explains all shader operations. APIs carried over from the original app are explained in docs/02-code.md.

- `abs`: [original Section 5.3](02-code.md#s5-3).
- `active_texture`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `add`: [Section 5.1](#51-rust-and-egui-api-reference).
- `all`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `allocate_exact_size`: [Section 5.1](#51-rust-and-egui-api-reference).
- `app::MoonApp::new`: [Section 5.1](#51-rust-and-egui-api-reference).
- `Arc::clone`: [Section 5.1](#51-rust-and-egui-api-reference).
- `Arc::new`: [Section 5.1](#51-rust-and-egui-api-reference).
- `as_luma16`: [Section 5.1](#51-rust-and-egui-api-reference).
- `as_raw`: [Section 5.1](#51-rust-and-egui-api-reference).
- `as_ref`: [Section 5.1](#51-rust-and-egui-api-reference).
- `assert!`: [original Section 4.8](02-code.md#s4-8).
- `assert_eq!`: [original Section 4.7](02-code.md#s4-7).
- `assert_ne!`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `attach_shader`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `available_size`: [Section 5.1](#51-rust-and-egui-api-reference).
- `bind_attrib_location`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `bind_buffer`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `bind_texture`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `bind_vertex_array`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `Box::new`: [original Section 13.1](02-code.md#s13-1).
- `buffer_data_u8_slice`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `bytemuck::cast_slice`: [Section 5.1](#51-rust-and-egui-api-reference).
- `checked_mul`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `clear`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `clear_depth_f32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `compile_shader`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `cos`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `create_buffer`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `create_program`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `create_shader`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `create_texture`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `create_vertex_array`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `ctx`: [Section 5.1](#51-rust-and-egui-api-reference).
- `cull_face`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `Default::default`: [Section 5.1](#51-rust-and-egui-api-reference).
- `delete_buffer`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `delete_program`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `delete_shader`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `delete_texture`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `delete_vertex_array`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `depth_func`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `depth_mask`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `direction`: [Section 3](#3-sphere-coordinates-mesh-and-camera).
- `destroy`: [Section 5.1](#51-rust-and-egui-api-reference).
- `detach_shader`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `disable`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `document`: [original Section 13.2](02-code.md#s13-2).
- `drain`: [Section 5.1](#51-rust-and-egui-api-reference).
- `draw_elements`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `eframe::egui::CentralPanel::default`: [original Section 12](02-code.md#s12).
- `eframe::run_native`: [original Section 13.1](02-code.md#s13-1).
- `eframe::WebRunner::new`: [original Section 13.2](02-code.md#s13-2).
- `egui::Sense::hover`: [Section 5.1](#51-rust-and-egui-api-reference).
- `egui_glow::CallbackFn::new`: [Section 5.1](#51-rust-and-egui-api-reference).
- `egui_glow::ShaderVersion::get`: [Section 5.1](#51-rust-and-egui-api-reference).
- `enable`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `enable_vertex_attrib_array`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `enumerate`: [original Section 4.5](02-code.md#s4-5).
- `env_logger::init`: [original Section 13.1](02-code.md#s13-1).
- `expect`: [original Section 13.2](02-code.md#s13-2).
- `extend`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `extend_from_slice`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `f32::from`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `fill`: [Section 5.1](#51-rust-and-egui-api-reference).
- `format!`: [original Section 4.9](02-code.md#s4-9).
- `frame`: [Section 5.1](#51-rust-and-egui-api-reference).
- `front_face`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_element_by_id`: [original Section 13.2](02-code.md#s13-2).
- `get_error`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_or_insert`: [Section 5.1](#51-rust-and-egui-api-reference).
- `get_parameter_i32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_program_info_log`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_program_link_status`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_shader_compile_status`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_shader_info_log`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `get_uniform_location`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `gl`: [Section 5.1](#51-rust-and-egui-api-reference).
- `globe::Globe::new`: [Section 5.1](#51-rust-and-egui-api-reference).
- `Globe::destroy`, `Globe::new`, `Globe::show`: [Section 5](#5-the-callback-ownership-and-graphics-api).
- `glow::PixelUnpackData::Slice`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `height`: [Section 5.1](#51-rust-and-egui-api-reference).
- `image::load_from_memory_with_format`: [Section 5.1](#51-rust-and-egui-api-reference).
- `include_bytes!`: [Section 5.1](#51-rust-and-egui-api-reference).
- `include_str!`: [Section 5.1](#51-rust-and-egui-api-reference).
- `input`: [Section 5.1](#51-rust-and-egui-api-reference).
- `into`: [original Section 4.9](02-code.md#s4-9).
- `is_err`: [original Section 5.12](02-code.md#s5-12).
- `is_new_shader_interface`: [Section 5.1](#51-rust-and-egui-api-reference).
- `is_positive`: [Section 5.1](#51-rust-and-egui-api-reference).
- `iter`: [original Section 4.7](02-code.md#s4-7).
- `len`: [original Section 4.2](02-code.md#s4-2).
- `link_program`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `lock`: [Section 5.1](#51-rust-and-egui-api-reference).
- `map`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `map_err`: [original Section 4.9](02-code.md#s4-9).
- `math::normal_map`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `math::perspective`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `math::sphere`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `mesh_orientation_and_seam`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `min`: [original Section 4.4](02-code.md#s4-4).
- `Mutex::new`: [Section 5.1](#51-rust-and-egui-api-reference).
- `normal_map`, `normalize`, `normals_metric_slopes_caps_and_invalid_input`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `ok_or`: [original Section 4.9](02-code.md#s4-9).
- `paint`: [Section 5.1](#51-rust-and-egui-api-reference).
- `painter`: [Section 5.1](#51-rust-and-egui-api-reference).
- `perspective`: [Section 3](#3-sphere-coordinates-mesh-and-camera).
- `panic!`: [original Section 13.2](02-code.md#s13-2).
- `pixel_store_i32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `projection_depth_and_aspect`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `push`: [original Section 4.9](02-code.md#s4-9).
- `rem_euclid`: [original Section 4.2](02-code.md#s4-2).
- `remove`: [original Section 13.2](02-code.md#s13-2).
- `Renderer::new`: [Section 5.1](#51-rust-and-egui-api-reference).
- `Renderer::destroy`, `Renderer::paint`, `Renderer::upload_texture`: [Section 5](#5-the-callback-ownership-and-graphics-api).
- `request_repaint`: [Section 5.1](#51-rust-and-egui-api-reference).
- `round`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `saturating_sub`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `set_text_content`: [original Section 13.2](02-code.md#s13-2).
- `shader_source`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `show`: [Section 5.1](#51-rust-and-egui-api-reference).
- `sin`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `sqrt`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `sphere`: [Section 3](#3-sphere-coordinates-mesh-and-camera).
- `start`: [original Section 13.2](02-code.md#s13-2).
- `std::array::from_fn`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `take`: [Section 5.1](#51-rust-and-egui-api-reference).
- `tan`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `tex_image_2d`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `tex_parameter_i32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `to_rgb8`: [Section 5.1](#51-rust-and-egui-api-reference).
- `to_string`: [Section 5.1](#51-rust-and-egui-api-reference).
- `uniform_1_f32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `uniform_1_i32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `uniform_matrix_4_f32_slice`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `unwrap`: [original Section 5.12](02-code.md#s5-12).
- `upload_texture`: [Section 5.1](#51-rust-and-egui-api-reference).
- `use_program`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `vec!`: [original Section 4.5](02-code.md#s4-5).
- `Vec::new`: [Section 4](#4-metric-normal-map-derivation-and-cpu-apis).
- `version_declaration`: [Section 5.1](#51-rust-and-egui-api-reference).
- `vertex_attrib_pointer_f32`: [Section 5.2](#52-every-glow-call-and-its-c-opengl-equivalent).
- `viewport_in_pixels`: [Section 5.1](#51-rust-and-egui-api-reference).
- `wasm_bindgen_futures::spawn_local`: [original Section 13.2](02-code.md#s13-2).
- `web_sys::window`: [original Section 13.2](02-code.md#s13-2).
- `width`: [Section 5.1](#51-rust-and-egui-api-reference).
