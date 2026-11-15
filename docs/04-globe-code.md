# Rust and the complete globe code

## 1. Rust from zero, with C++ comparisons

Read [the rendering definitions and equations](03-moon.md) before the shaders below. This guide defines the language tools before quoting their uses. Rust syntax is case-sensitive. A statement ending in a semicolon discards its expression value; a block's final expression without a semicolon is its value. This is why a function can end in Ok(value) without a return statement. C++ normally needs return there.

### 1.1 Values, types, and ownership

A binding names a value: `let x: f32 = 1.0;` corresponds to `const float x = 1.0f;`. The mut keyword permits reassignment, as in `let mut indices = Vec::new();`, comparable to a mutable vector. f32/f64 are 32/64-bit floats; u8/u32 are unsigned integers; i32 is signed; usize indexes memory; bool stores a truth value. Rust does not implicitly convert these types. `n as f32` is an explicit numerical cast, like static_cast<float>(n), and can lose precision. Array types such as [f32;3] have a fixed size, like std::array<float,3>. A tuple such as (bool,Option<String>) groups differently typed values, like std::tuple.

String owns UTF-8 text; Vec<T> owns a growable contiguous allocation, like std::string and std::vector<T>. A slice [T] describes a contiguous range whose length is known at runtime; borrowing it as &[T] resembles std::span<const T>. str is a UTF-8 string slice, not an owning string. Indexing outside a slice panics, rather than invoking C++ undefined behavior. Vec::new creates an empty vector; push appends one value; extend_from_slice copies a borrowed range. len returns the number of elements, not bytes.

Every owning value has one owner unless explicitly shared. Assignment and function arguments usually move non-Copy values. `let b = a;` for Vec transfers ownership like std::move(a), but Rust prevents later use of a automatically. Primitive numbers and small numeric arrays implement Copy and are copied implicitly. Drop releases ordinary Rust-owned memory when its owner leaves scope, like a C++ destructor. clone explicitly duplicates owned data, but Arc::clone only increments a reference count. These operations have different costs.

### 1.2 References, lifetimes, and mutable access

`&value` borrows without taking ownership, similar to a const reference. `&mut value` borrows with exclusive mutable access, similar to a mutable reference with a compiler-enforced nonaliasing contract. Many shared borrows or one exclusive borrow may exist for a region of memory at once. Neither borrowed form frees its referent. A method's &self or &mut self is the corresponding borrowed receiver, comparable to const/nonconst C++ member functions.

A lifetime describes how long a borrow remains valid. The compiler usually infers it. The app's CreationContext<'_> uses an inferred lifetime placeholder, not a new allocation or a manual lifetime extension. Closures kept by egui must not borrow stack locals that disappear at the end of a frame. Static strings, including include_str! results, live for the program lifetime. C++ const char* does not itself enforce either guarantee.

Dereferencing uses *, as in *renderer=Some(ready). Here renderer is a lock guard implementing dereference access to protected data. A borrowed slice &vertices does not copy the vector. as_ref borrows an Option's value; as_deref also dereferences its inner String or Vec to str or a slice. take on an Option moves its contents out and leaves None, similar to exchange(optional,nullopt), not optional::value which leaves it present.

### 1.3 Optional values and errors

Option<T> is either Some(T) or None, comparable to std::optional<T>. Result<T,E> is either Ok(T) or Err(E), comparable to std::expected<T,E>. They are tagged enums, not null pointers. Result<(),String> means successful completion has no payload, or failure owns a diagnostic. The empty tuple () is Rust's unit type, comparable to void for returns.

match selects a variant exhaustively. if let selects one pattern. let Some(program)=self.program else { return; }; exits when absent and unwraps safely otherwise. The | in Some("2" | "2k") means alternative patterns. The same character delimits closure arguments in |n| n+1; context distinguishes it. Rust's => separates a match pattern from its result.

The ? operator unwraps success and returns an error immediately from the enclosing Result-returning function or closure. C++ expected needs an explicit check and return at each site. unwrap panics on Err/None; expect does the same with an explanatory message. They are appropriate only after an invariant has been checked, or for impossible browser startup assumptions. Asset and GPU failures normally propagate as Result rather than panicking.

map transforms the contained success/present value; map_err transforms only the error. and_then chains an operation already returning the same wrapper family. ok discards a Result error and creates an Option. ok_or converts absence to the supplied error. unwrap_or supplies an eager fallback; unwrap_or_else calls its closure only on absence/error. unwrap_or_default uses Default's value, such as an empty String. transpose converts Option<Result<T,E>> into Result<Option<T>,E>: absent remains Ok(None), present error becomes Err, and present success becomes Ok(Some(T)). is_none/is_some/is_err return a boolean without extracting anything. An empty image result is distinct from a failed image result.

### 1.4 Modules, traits, impl, and dynamic dispatch

A crate is one Rust compilation unit; Cargo packages its manifest and source. mod assets declares a sibling module; use imports names into lexical scope, not source text. pub exposes a symbol outside the crate and pub(crate) exposes it only within the crate. Unmarked fields/functions are private to their module and descendants. super names the parent module, so use super::* exposes the parent's names in its tests.

An impl block supplies a type's methods, like C++ out-of-class member definitions. A trait specifies callable behavior, comparable to an interface or constrained template contract. impl eframe::App for MoonApp implements eframe's interface. HasContext supplies glow's GL methods; importing it as _ makes method lookup work without introducing a used local name. From/Into implement conversions; .into() asks for the target type inferred from context. Default::default constructs a type's documented default. Struct update syntax ..Default::default() supplies all fields not explicitly set.

Box<T> owns a heap allocation, similar to unique_ptr<T>. Box<dyn eframe::App> combines ownership with a trait-object vtable, like unique_ptr<Base> with virtual dispatch. eframe's factory accepts a closure returning an owned application; the concrete Box<MoonApp> coerces to that trait object. The source's Box::new creates these boxes and the factory callable.

### 1.5 Closures, sharing, asynchronous work, and unsafe

A closure is an unnamed callable: |x| x+1 resembles a C++ lambda. move asks it to own its captures rather than borrow this stack frame; it does not create a thread. A closure captures an Arc cheaply because the code clones the shared owner before moving it. FnOnce denotes a callable allowed to consume captures on its one invocation. Browser ehttp uses a one-shot completion.

Arc<T> is atomic reference-counted shared ownership, like shared_ptr<T>. It does not make arbitrary T thread-safe. Mutex<T> supplies exclusive access; lock returns a guard and dropping the guard unlocks. This code uses egui's Mutex wrapper, not std's poison-returning lock API. Therefore lock here returns a guard directly, not Result<Guard,PoisonError>. The callback owns Arc<Mutex<Option<Renderer>>> and Arc<Mutex<Loading>>, avoiding dangling frame-local references.

Native std::thread::spawn runs its moved callable on another thread. Dropping the unused JoinHandle detaches that worker; it still owns its shared loading state. Browser ehttp::fetch schedules network work and later calls its completion. async blocks create futures; .await suspends that future until an operation completes. spawn_local schedules a non-Send future on the browser's thread. It does not launch a CPU worker. The app uses it for WebRunner::start.

unsafe permits operations whose invariants the compiler cannot prove. It is not permission to violate them. GL calls require a current context and valid handles/buffer sizes. C++ requires the same contracts without an unsafe marker. The renderer does not retain glow::Context in the Send+Sync callback: browser context internals are not Sync. It borrows painter.gl() only during drawing. No Drop implementation calls GL after context teardown; on_exit explicitly destroys resources while the context is current.

### 1.6 Attributes, macros, iterators, and tests

#[cfg(...)] includes an item only for the selected build configuration, comparable to #if. target_arch="wasm32" selects browser-specific paths, not a runtime branch. #[cfg(test)] and #[test] register test-only modules/functions. cfg_attr conditionally applies another attribute; windows_subsystem changes Windows release startup behavior without affecting Linux. Attributes decorate syntax and are not function calls.

Macros use ! and expand syntax. format! creates a String with checked formatting, like std::format. println!/eprintln! print formatted output. env! embeds a build-time environment value; std::env::var instead reads the runtime environment. include_str! embeds a UTF-8 file as text; no texture tier is embedded. assert!, assert_eq!, and panic! fail through Rust's panic mechanism. Assertions remain active in release builds unless specifically using debug_assert!.

An iterator produces values lazily. iter borrows elements; into_iter consumes the array here; enumerate adds zero-based usize indices. find returns the first matching item as Option; any/all short-circuit on the first decisive boolean. flatten skips None entries in the completion array. filter retains matching values. Iterator map transforms each element and is distinct from Option::map even though the method name matches. drain(..) removes and yields all owned handles for deletion. A range a..b excludes b; a..=b includes b; .. denotes an entire range. C++ uses iterator pairs or ranges for these operations.

as_chunks::<3>() borrows RGB as fixed-size triples and returns (complete_triples,remainder). The .0 selects the first tuple field; iter borrows each [u8;3]. Decoded RGB images guarantee an empty remainder. flat_map turns each triple into a four-byte RGBA array and flattens the results. collect allocates the inferred Vec<u8>. then evaluates its closure only when its bool receiver is true and wraps it in Some. bytemuck::cast_slice borrows numeric vectors as bytes with checked size/alignment and Pod requirements; C++ would use a const byte span after a carefully justified reinterpret_cast. No vector contents move during that borrow.

## 2. Library APIs and GL contracts

The API descriptions were checked against [Rust std](https://doc.rust-lang.org/std/), [glow 0.17](https://docs.rs/glow/0.17.0/glow/trait.HasContext.html), [egui_glow 0.37](https://docs.rs/egui_glow/0.37.0/egui_glow/struct.CallbackFn.html), [egui 0.37](https://docs.rs/egui/0.37.0/egui/), [image 0.25](https://docs.rs/image/0.25.10/image/), and [ehttp 0.7](https://docs.rs/ehttp/0.7.3/ehttp/fn.fetch.html). Each glow statement below includes its C++ OpenGL equivalent immediately above it; the operation is the same OpenGL API, but native GLuint handles and browser objects use platform-specific Rust types.

### 2.1 Loading and platform APIs

std::env::var returns a UTF-8 String or an error; var_os permits OS-native non-UTF-8 paths. current_exe returns a fallible executable PathBuf; parent returns an optional borrowed directory. Path::new borrows a path-like string, PathBuf::from owns one, join constructs a child path, and is_dir tests filesystem metadata and returns false on inaccessible/non-directory paths. fs::read reads a file into Vec<u8> or returns an I/O error. C++ filesystem::path and ifstream provide similar operations but with different error conventions.

image::load_from_memory detects a supported encoded image from &[u8] and returns Result<DynamicImage,ImageError>. Only JPEG/PNG features are enabled. to_rgb8 converts it to an owned three-channel u8 image. width/height return pixel dimensions; as_raw borrows contiguous row-major bytes. RgbImage::new allocates a zero-initialized image for tests. to_string formats a Display value into an owned String; format! preserves the filename in I/O diagnostics.

web_sys::window returns Option<Window>; navigator and location access DOM objects. user_agent, inner_width, search, and UrlSearchParams::new_with_str are fallible bindings; JS errors are converted to Rust Result. as_f64 extracts a JS numeric value as Option<f64>. URLSearchParams::get returns the first named value or None. A failed width lookup becomes 0 and conservatively selects the phone tier. to_lowercase allocates lowercase text. contains searches a substring, not a token or regex. console::log_1 logs one JS value; into converts the Rust text to a JS string.

ehttp::Request::get builds a GET. fetch takes it and a 'static one-shot completion receiving Result<Response,String>. Transport failure is Err, but HTTP 404 is a Response with ok=false; our code checks that field and status before decoding bytes. Relative URLs resolve against the document location. The browser's same-origin/CORS rules apply. There is no automatic alternate-tier retry.

### 2.2 UI, shader creation, and math APIs

CentralPanel::default and Frame::NONE.fill(Color32::BLACK) make the app's background. show executes a UI closure. available_size reports remaining logical points; allocate_exact_size reserves that area and returns (Rect,Response). Sense::hover requests no drag interaction. is_positive excludes empty/inverted rectangles. input passes current input state to a closure; its time is seconds. get_or_insert initializes an Option only once and returns a mutable reference. rem_euclid(60.0) wraps finite elapsed time into [0,60), unlike a negative C++ remainder. The frame code casts the fraction to f32 and scales it by TAU=2*pi.

CallbackFn::new stores a Send+Sync closure for deferred GL execution. PaintCallback associates it with a rectangle. painter().add queues it; viewport_in_pixels supplies integer dimensions after egui sets viewport/scissor; painter.gl() borrows the current context. ctx().request_repaint schedules another frame. The UI may run before the assets finish; only the paint callback constructs Renderer because it has the right context current.

ShaderVersion::get chooses a GLSL dialect from the current context. is_new_shader_interface prevents using unsupported older interfaces. version_declaration supplies the #version prefix for desktop or ES; format! joins it to include_str! sources. Shader creation, compilation, status/error queries, attachment, linking, and deletion remain separate fallible operations. The initialization closure records each handle before the next operation can fail; failure calls destroy to clean partial state.

abs, min, max, sin, cos, tan, and atan/acos have their ordinary numerical meanings, with radians for angles; NaN is not a meaningful input to our geometry. sqrt needs a nonnegative input. PI, FRAC_PI_2, and TAU are typed f32 constants. Sphere dimensions are asserted before integer division; direction has no allocation. Perspective returns sixteen column-major f32 entries and assumes a positive viewport aspect, enforced by the caller's dimension test.

GL creation methods return Result<Handle,String>, while later GL commands mostly return unit and report driver errors through get_error. get_uniform_location returns None for inactive/missing names; uniform setters ignore None. This is how glow represents C++ location -1. Texture unit zero's index is 0, whereas TEXTURE0 is an enum: sampler uniforms receive indices, active_texture receives enums. Attribute offsets/strides and draw index offsets are bytes, not element counts. STATIC_DRAW is a usage hint, not immutable storage.

Albedo uses opaque SRGB8_ALPHA8 storage with RGBA input so WebGL2 permits generate_mipmap. Bare SRGB8 permits sampling but is not a WebGL2 color-renderable mip-generation format. Normal storage remains RGB8. The temporary RGBA conversion adds up to 4*width*height CPU bytes during upload; it is released afterward. UNPACK_ALIGNMENT is set to 1 for RGB rows and restored to 4. REPEAT applies to longitude, CLAMP_TO_EDGE to latitude. LINEAR_MIPMAP_LINEAR uses the full generated chain; magnification uses LINEAR.

New textures retain the default anisotropy ratio one. We do not query or enable the optional anisotropic path: controlled 8k tests isolated it as a trigger for the dotted arc on Mesa llvmpipe. Trilinear mipmaps remain active. This trades some grazing-angle sharpness for reliable filtering without guessing a driver-name blacklist. get_error clears one pending driver error; texture upload returns a readable diagnostic if nonzero. The callbacks disable blending, enable depth writes/test and back-face culling, bind their program/VAO/textures, and unbind/disable those states afterward. egui restores its own paint state for later UI shapes.

### 2.3 GLSL APIs

smoothstep(edge0,edge1,x) returns t*t*(3-2*t), where t=clamp((x-edge0)/(edge1-edge0),0,1). The edges must satisfy edge0<edge1; results outside that contract are undefined. Our edges are 0.35 and 0.65. In C++, compute the same expression with std::clamp. mix(a,b,t) computes (1-t)*a+t*b componentwise; it is a linear interpolation, comparable to std::lerp for scalar inputs. We fade two sunlight responses to the geometric-sphere response near the limb, after correcting the mapped normal's view component. The derivation and presentation thresholds are in rendering-guide section 7.

vec2/vec3/vec4 construct float vectors; mat4 is a four-column matrix. Arithmetic on vectors is componentwise except matrix multiplication. normalize divides by length; dot sums component products. acos maps a cosine in [-1,1] to [0,pi]; clamp enforces that interval against roundoff. atan(y,x) returns the quadrant-aware angle. exp and log are natural exponential/logarithm; log requires a positive argument. pow raises a positive base to an exponent. sqrt receives max(...,0) in roundoff-sensitive expressions. inversesqrt(x) computes 1/sqrt(x) for positive x; our roughness normalization argument is at least one.

max/min bound values; abs removes sign; tan has poles, so azimuth pi is handled by its limiting branch. mix(a,b,t)=a*(1-t)+b*t; step(edge,x) returns 0 below edge, 1 otherwise. Together they choose the piecewise sRGB branch per channel. dFdx/dFdy estimate local screen-space changes across fragment quads. textureGrad samples an explicitly supplied footprint, returning four channels; unlike an ordinary texture call it does not derive the wrapped longitude's discontinuity. All those functions use GLSL scalar/vector overloads, not Rust methods.

Default floating-point in/out interpolation is smooth and perspective-correct, commonly at the pixel center but more generally at an implementation-selected location within the pixel. With MSAA, coverage can include sample locations even if that interpolation location is outside the triangle. The optional centroid auxiliary qualifier instead requests an inside-primitive location within the pixel when possible. It does not enable per-sample shading. Coverage-dependent locations can differ between adjacent pixels, complicating dFdx/dFdy. Our fixed-pose centroid trial worsened the texture pattern, so the production declarations do not use it. Rendering-guide sections 5 and 6 explain coverage, the trial matrix, and why disabling optional anisotropy was selected instead. The shaders retain their acos argument clamp and bounded polar denominator.

## 3. File and function walkthrough

globe/Cargo.toml makes this an independent library depending only on egui, egui_glow, bytemuck, and JPEG/PNG image decoding. WASM-only dependencies add ehttp and browser bindings. No lunar_core dependency is introduced. app depends on globe by path. index.html copies the six static assets for Trunk, sets the browser title to Karaya Explorer, and sets a black page background.

assets.rs defines Loading with two optional results and a terminal diagnostic. select parses a requested label, chooses a default from mobile status, then searches descending supported widths under both limits. preferences obtains runtime platform hints. load starts two disk workers or two HTTP completions, decodes each image, and writes its fixed slot under the mutex. Loading::take first returns any existing error; only when both slots exist does it move both out. Otherwise it returns Ok(None). tiers_and_loading covers default/forced/clamped quality, invalid quality/GPU size, incomplete data, a failed slot, successful transfer, and the emptied state.

math.rs defines direction, the trigonometric convention. sphere asserts dimensions, emits interleaved vertices row by row, makes exact poles/seam copies, and appends six indices per cell. perspective constructs the projective matrix derived in the rendering guide. The test-only legacy rotation helper preserves a former 30-degree presentation example. Production View composes free yaw/tilt and zoom, and its projection sphere-occludes site directions; the default Earth-following camera is described in the real-lighting guide. The [panorama guide](05-panorama.md) derives the shared CPU/GPU transformation. Tests check mesh dimensions, index bounds, exact seams, orientation, winding, depth endpoints, portrait scaling, and the example marker through a full turn.

lib.rs exposes Globe. new queries the GPU limit while eframe's context is current, selects/logs one tier, and starts loading; it does not decode all tiers. show submits a loading/error label if needed, reserves the viewport, computes rotation, captures shared state, and queues CallbackFn. That callback consumes completed images, constructs Renderer on the current GL context, stores errors, and paints when ready. Zero-area viewports are skipped. destroy takes the optional renderer and frees it once.

Renderer::new validates the shader dialect, records resource handles, compiles and links both stages, checks diagnostics, deletes successfully linked stage objects, uploads the production mesh and both images, and logs readiness. Its local Result closure centralizes cleanup of partial initialization. upload_texture validates device dimensions, makes opaque RGBA only for sRGB albedo, uploads, generates mipmaps, retains default isotropic filtering, and reports GL failure. paint clears depth, binds projection/rotation/samplers, draws indexed triangles, and releases its bindings. Renderer::destroy drains textures/buffers/shaders and takes optional VAO/program handles before deleting them; repeated calls do nothing.

moon.vert computes rotation, passes both world and object coordinates, subtracts the camera, and projects. moon.frag's h_function implements the logarithmic multiple-scattering approximation. hapke returns zero for locally hidden facets, evaluates the signed HG lobes, SHOE, both roughness branches and H-functions, then multiplies their radiance-factor expression. linear_to_srgb encodes display light. main reconstructs seam-safe UV/gradients, samples and rotates normals, forms light/view geometry, masks the geometric night side, samples relative linear color, exposes/tone-maps it, and writes opaque output.

app/src/lib.rs owns Globe and Panorama in MoonApp. new requires glow and starts one asynchronous scientific data load. ui consumes any completion, supplies geographic marker coordinates, and displays responsive black-backed panes. A 700-point breakpoint switches columns to a vertical stack; a bounded child layout inside a scroll area handles short screens. on_exit destroys resources before eframe tears down GL. app/src/main.rs has conditional native/browser entry points, with statement-level C++ comparisons below. The [panorama guide](05-panorama.md) teaches the new data loader, plot APIs, UTC controls, and visibility readouts and quotes both new modules in full.

## 4. Exact, statement-annotated source

The comments describe what each statement does, why it is needed, and the corresponding C++ construct or GL function. Continued argument lines belong to the statement introduced above them. Delimiters close that same statement/scope rather than introduce another computation. The source is quoted in full, including tests, rather than shortened examples.

### 4.1. globe/Cargo.toml

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

[target.'cfg(target_arch = "wasm32")'.dependencies]
ehttp = "0.7"
web-sys = { version = "0.3", features = ["Window", "Navigator", "Location", "UrlSearchParams", "HtmlImageElement", "WebGl2RenderingContext", "console"] }
wasm-bindgen-futures = "0.4"
send_wrapper = "0.6"
```

### 4.2. globe/src/assets.rs

```rust
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui::mutex::Mutex;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
#[cfg(not(target_arch = "wasm32"))]
use image::RgbImage;

// Native workers return owned RGB bytes; C++ image value alias.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type TextureImage = RgbImage;
// Keep DOM images on their creation thread even across Send-bound UI callbacks; C++ checked thread-affine handle.
#[cfg(target_arch = "wasm32")]
pub(crate) type TextureImage = send_wrapper::SendWrapper<web_sys::HtmlImageElement>;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::sync::Arc;

// Keep asynchronous CPU results separate from current-context GPU state; C++ defines a similar aggregate.
pub(crate) struct Loading
{
	// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
	pub images: [Option<Result<TextureImage, String>>; 2],
	// An optional terminal loading/upload diagnostic; C++ optional<string>.
	pub error: Option<String>,
}

// Select one supported tier or return a diagnostic; C++ expected<unsigned,string> replaces Result.
pub(crate) fn select(max: i32, mobile: bool, forced: Option<&str>) -> Result<u32, String>
{
	// Choose a requested width before applying the GPU limit; C++ switch selects an integer.
	let desired = match forced
	{
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some("2" | "2k") => 2048,
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some("4" | "4k") => 4096,
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some("8" | "8k") => 8192,
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some(_) => return Err("MOON_TIER must be 2, 4 or 8".into()),
		// Represent an absent optional alternative; C++ nullopt.
		None =>
		{
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if mobile
			{
				2048
			}
			// Handle the complementary case; C++ uses the same else branch.
			else
			{
				8192
			}
		}
	};
	// Search widths largest first so clamping preserves the highest allowed tier; C++ scans an ordered array.
	[8192, 4096, 2048]
		// Consume this array into an iterator; C++ iterates its owned local elements.
		.into_iter()
		// Return the first tier satisfying both bounds; C++ ranges::find_if plus optional conversion.
		.find(|&n| n <= max && n <= desired)
		// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
		.map(|n| n as u32 / 1024)
		// Convert absence into the given error value; C++ constructs expected from optional after a presence test.
		.ok_or("Moon needs MAX_TEXTURE_SIZE >= 2048".into())
}

// Read platform-specific quality hints without doing I/O; C++ conditional compilation selects the implementation.
pub(crate) fn preferences() -> (bool, Option<String>)
{
	// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
	#[cfg(not(target_arch = "wasm32"))]
	{
		// Native machines default to desktop policy; C++ getenv plus optional<string> supplies an override.
		(false, std::env::var("MOON_TIER").ok())
	}
	// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
	#[cfg(target_arch = "wasm32")]
	{
		// Obtain the browser Window object; C++ native builds do not have this DOM API.
		let window = web_sys::window().expect("Browser window");
		// Normalize the user-agent string for case-insensitive matching; C++ transforms a string to lowercase.
		let ua = window
			// Access the browser's Navigator for device hints; this is a web platform API.
			.navigator()
			// Read the user-agent string, which is a heuristic and not a hardware guarantee.
			.user_agent()
			// Use an empty string if the browser provides no user agent; C++ optional::value_or(string{}).
			.unwrap_or_default()
			// Allocate a lowercase Unicode string for matching; C++ transforms characters into a new string.
			.to_lowercase();
		// Record whether the browser looks mobile; C++ stores the corresponding bool.
		let mobile = ["android", "iphone", "ipad", "mobile"]
			// Borrow elements without moving them; C++ cbegin/cend or a const range.
			.iter()
			// Return true at the first matching heuristic/extension; C++ ranges::any_of short-circuits.
			.any(|s| ua.contains(s))
			|| window
				// Read browser viewport width in CSS pixels; high-density screens still use logical CSS units.
				.inner_width()
				// Discard a platform error and retain only success as an Option; C++ converts expected to optional.
				.ok()
				// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
				.and_then(|n| n.as_f64())
				// Substitute the stated default if absent; C++ optional::value_or.
				.unwrap_or(0.0)
				<= 600.0;
		// Read an optional user tier override; C++ would use optional<string>.
		let forced = window
			// Access the current browser URL; this is a web platform API.
			.location()
			// Read its query string, including the leading question mark if present.
			.search()
			// Discard a platform error and retain only success as an Option; C++ converts expected to optional.
			.ok()
			// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
			.and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
			// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
			.and_then(|p| p.get("moon_tier"));
		// Return both browser hints without allocating another aggregate; C++ returns pair<bool,optional<string>>.
		(mobile, forced.or_else(|| (!mobile).then(|| "4".into())))
	}
}

// Start exactly two asynchronous image reads for one tier; C++ shared_ptr keeps completion state alive.
pub(crate) fn load(tier: u32, ctx: Option<egui::Context>) -> Arc<Mutex<Loading>>
{
	// Share two pending image results with asynchronous producers; C++ shared_ptr protects a mutex-backed aggregate.
	let state = Arc::new(Mutex::new(Loading {
		// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
		images: [None, None],
		// An optional terminal loading/upload diagnostic; C++ optional<string>.
		error: None,
	}));
	// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
	let normal = if cfg!(target_arch = "wasm32")
	{
		format!("normal_{tier}k.webp")
	}
	else
	{
		format!("normal_{tier}k.png")
	};
	for (index, filename) in [format!("albedo_{tier}k.jpg"), normal]
		// Consume this array into an iterator; C++ iterates its owned local elements.
		.into_iter()
		// Attach each element's zero-based index; C++ an indexed range or explicit counter.
		.enumerate()
	{
		// Share two pending image results with asynchronous producers; C++ shared_ptr protects a mutex-backed aggregate.
		let state = Arc::clone(&state);
		// Wake a paused globe when either upgrade image completes; C++ optional UI event-loop handle.
		let ctx = ctx.clone();
		// Decode only native images in Rust; browser images stay in the native browser decoder.
		#[cfg(not(target_arch = "wasm32"))]
		// Capture the completion action by ownership; C++ uses a value-capturing callable.
		let done = move |bytes: Result<Vec<u8>, String>| {
			// Carry either the next successful value or a diagnostic; C++ could use expected<T,string>.
			let result = bytes.and_then(|b| {
				// Detect JPEG/PNG and decode only this downloaded image; C++ would use an image decoder on a byte span.
				image::load_from_memory(&b)
					// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
					.map(|i| i.to_rgb8())
					// Transform only an error into a printable diagnostic; C++ maps the error alternative of expected.
					.map_err(|e| e.to_string())
			});
			// Access an image-completion slot under exclusive synchronization; C++ locks its mutex before accessing the slot.
			state.lock().images[index] = Some(result);
			if let Some(ctx) = ctx
			{
				ctx.request_repaint();
			}
		};
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Move disk I/O and image decoding to a native worker thread; C++ launches a thread with a value capture.
		read_native(filename, done);
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Browser-native decoding runs asynchronously rather than in the WASM frame; C++ platform decoder task.
		read_browser(filename, state, index, ctx);
	}
	state
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Loading
{
	// Move both completed images out only when neither is missing or erroneous; C++ expected<optional<array<Image,2>>,string>.
	pub fn take(&mut self) -> Result<Option<[TextureImage; 2]>, String>
	{
		// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
		for result in self.images.iter().flatten()
		{
			// Extract a completed error before consuming either image; C++ tests expected::has_value and reads its error.
			if let Err(error) = result
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err(error.clone());
			}
		}
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if self.images.iter().all(Option::is_some)
		{
			// Wrap a successful result without throwing; C++ expected stores its value.
			Ok(Some([
				// Access an image-completion slot under exclusive synchronization; C++ locks its mutex before accessing the slot.
				self.images[0].take().unwrap()?,
				// Access an image-completion slot under exclusive synchronization; C++ locks its mutex before accessing the slot.
				self.images[1].take().unwrap()?,
			]))
		}
		// Handle the complementary case; C++ uses the same else branch.
		else
		{
			// Report that construction is still pending; C++ optional remains empty without signaling failure.
			Ok(None)
		}
	}
}

// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
#[cfg(test)]
// Group tests in a nested module; C++ puts test helpers in a namespace.
mod tests
{
	// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
	use super::*;
	// Register the following function with Rust's test runner; C++ needs a test framework registration.
	#[test]
	// Exercise normal, clamped, invalid, pending, failed, and completed loader states; C++ would use test assertions.
	fn tiers_and_loading()
	{
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(16384, false, None).unwrap(), 8);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(16384, true, None).unwrap(), 2);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(4096, false, Some("8")).unwrap(), 4);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(8192, false, Some("2")).unwrap(), 2);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(select(1024, true, None).is_err());
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(select(8192, false, Some("bad")).is_err());
		// Create a loader fixture with neither image complete; C++ initializes a local aggregate.
		let mut s = Loading {
			// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
			images: [None, None],
			// An optional terminal loading/upload diagnostic; C++ optional<string>.
			error: None,
		};
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(s.take().unwrap().is_none());
		// Simulate completion of the first image; C++ assigns an expected<Image,string> into an optional slot.
		s.images[0] = Some(Ok(RgbImage::new(2, 1)));
		// Simulate an HTTP/disk/decoder failure for the second image; C++ stores unexpected<string>.
		s.images[1] = Some(Err("missing file".into()));
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(s.take().unwrap_err(), "missing file");
		// Replace the failure with a successful completion and verify both images can be consumed.
		s.images[1] = Some(Ok(RgbImage::new(2, 1)));
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(s.take().unwrap().is_some());
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(s.take().unwrap().is_none());
	}
}

#[cfg(not(target_arch = "wasm32"))]
fn read_native(filename: String, done: impl FnOnce(Result<Vec<u8>, String>) + Send + 'static)
{
	std::thread::spawn(move || {
		// Resolve the native asset directory once in this worker; C++ filesystem::path owns its path.
		let directory = std::env::var_os("MOON_ASSET_DIR")
			// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
			.map(std::path::PathBuf::from)
			// Compute the fallback only when absent; C++ checks optional before calling a fallback lambda.
			.unwrap_or_else(|| {
				// Try assets/globe beside the executable before the development fallback; C++ optional<path>.
				let beside = std::env::current_exe()
					// Discard a platform error and retain only success as an Option; C++ converts expected to optional.
					.ok()
					// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
					.and_then(|p| p.parent().map(|p| p.join("assets/globe")));
				// Use the executable-relative directory only if it exists; C++ filesystem::is_directory guards a path.
				beside.filter(|p| p.is_dir()).unwrap_or_else(|| {
					// Fall back to the compile-time crate directory for workspace runs; C++ a build-defined source path is similar.
					std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
				})
			});
		// Complete this asset exactly once with bytes or a diagnostic; C++ invokes the captured completion callable.
		done(std::fs::read(directory.join(&filename)).map_err(|e| format!("{filename}: {e}")));
	});
}

#[cfg(target_arch = "wasm32")]
fn read_browser(
	filename: String,
	state: Arc<Mutex<Loading>>,
	index: usize,
	ctx: Option<egui::Context>,
)
{
	wasm_bindgen_futures::spawn_local(async move {
		// Resolve DOM construction failures through the ordinary loader diagnostic; C++ expected-returning factory.
		let result = match web_sys::HtmlImageElement::new()
		{
			Ok(image) =>
			{
				// Same-origin assets need no cross-origin permission or JS byte copies; C++ URL assignment.
				image.set_src(&format!("assets/globe/{filename}"));
				// Await the browser's own decoder and retain its native upload source; C++ future continuation.
				wasm_bindgen_futures::JsFuture::from(image.decode())
					.await
					.map(|_| send_wrapper::SendWrapper::new(image))
					.map_err(|e| format!("{filename}: {e:?}"))
			}
			Err(e) => Err(format!("{filename}: {e:?}")),
		};
		// Publish a complete image or a genuine error, never a half-decoded buffer; C++ synchronized slot.
		state.lock().images[index] = Some(result);
		// Background completion must repaint even when the first Moon is already visible; C++ event-loop wake.
		if let Some(ctx) = ctx
		{
			ctx.request_repaint();
		}
	});
}
```

### 4.3. globe/src/math.rs

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

### 4.4. globe/src/lib.rs

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

### 4.5. globe/src/moon.vert

```glsl
// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Read one unit object-space sphere vertex from attribute location zero.
in vec3 a_position;
// Read the mesh UV from location one; fragment shading reconstructs UV from direction for seam-safe derivatives.
in vec2 a_uv;
// Signed cross-strip or circular coordinates for analytic ray coverage; C++ vertex attribute.
in vec2 a_coverage;
out vec2 v_coverage;
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
	v_coverage = a_coverage;
	// Export the original direction for texture lookup independently of rotation.
	v_object = a_position;
	// Translate world position to view space and project to homogeneous clip coordinates; rasterization divides by w.
	gl_Position = u_projection * vec4(p - vec3(0.0, 0.0, 3.3), 1.0);
	// Multiplication by homogeneous w preserves constant screen width after perspective division.
	if (u_ray_mode != 0) gl_Position.xy += a_uv * gl_Position.w;
}
```

### 4.6. globe/src/moon.frag

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

### 4.7. app/src/lib.rs

```rust
// Compile the validated data loader; C++ includes a project module's declarations.
mod data;
// Compile the immediate-mode panorama view; C++ includes a project module's declarations.
mod panorama;
// Compile the angular sky-strip painter; C++ project drawing module.
mod sky;
// Keep grouped presentation settings outside scientific orchestration; C++ module declaration.
mod layout;
mod menu;

// Deserialize the small metadata catalog only, never embed the per-site ephemerides; C++ aggregate schema.
#[derive(serde::Deserialize)]
struct Entry
{
	name: String,
	lat_deg: f64,
	lon_deg: f64,
	source: String,
}
// Derive the same ASCII filename convention used by prep; C++ string transform.
fn filename(name: &str) -> String
{
	format!(
		"{}.json",
		name.split(|c: char| !c.is_ascii_alphanumeric())
			.filter(|s| !s.is_empty())
			.map(str::to_ascii_lowercase)
			.collect::<Vec<_>>()
			.join("-")
	)
}

// Own persistent renderer and scientific-view state; C++ application class members.
pub struct MoonApp
{
	// Keep presentation independent of the scientific core; C++ renderer value member.
	globe: globe::Globe,
	// Embed only published metadata; C++ vector of catalog value records.
	catalog: Vec<Entry>,
	// Cache one optional asynchronous view model per stable catalog identifier; C++ vector<optional<View>>.
	panoramas: Vec<Option<panorama::Panorama>>,
	// Share the selected catalog identifier with globe picking; C++ array index.
	selected: usize,
	// Default to the selected site's pair rather than launching all requests; C++ bool option.
	all_rays: bool,
	// Preserve a UTC epoch independently of each view's sample index; C++ optional<int64_t>.
	time: Option<i64>,
	// Persist only whether the phone drawer is open; C++ bool member.
	menu_open: bool,
}

// Define application member functions; C++ out-of-class member definitions.
impl MoonApp
{
	// Construct both views with the active GL context and event-loop handle; C++ expected-returning factory.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, String>
	{
		// Borrow the glow context or propagate a readable error; C++ checked optional reference.
		let gl = cc
			.gl
			.as_ref()
			.ok_or("The Moon requires the glow renderer")?;
		// Use readable light text on dark panels consistently across platforms; C++ UI theme setter.
		cc.egui_ctx.set_theme(eframe::egui::ThemePreference::Dark);
		// Parse exact published metadata before initializing any lazy track loads; C++ checked JSON decode.
		let catalog: Vec<Entry> =
			serde_json::from_str(include_str!("../../sites.json")).map_err(|e| e.to_string())?;
		// Check this state or input before acting; C++ conditional branch.
		if catalog.is_empty()
		{
			return Err("Empty site catalog".into());
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut panoramas: Vec<_> = (0..catalog.len()).map(|_| None).collect();
		panoramas[0] = Some(panorama::Panorama::new(
			cc.egui_ctx.clone(),
			data::FILE.into(),
		));
		// Move independently owned models into the app; C++ aggregate initialization.
		Ok(Self {
			// Propagate renderer initialization errors; C++ expected error forwarding.
			globe: globe::Globe::new(gl)?,
			// Start one asynchronous data request, not one per frame; C++ copy the shared UI context handle.
			catalog,
			panoramas,
			selected: 0,
			all_rays: false,
			time: None,
			menu_open: false,
		})
	}
	// Render the selected scientific view while preserving the shared epoch; C++ member function.
	fn show_panorama(&mut self, ui: &mut eframe::egui::Ui)
	{
		if let Some(p) = &mut self.panoramas[self.selected]
		{
			if let Some(time) = self.time
			{
				p.set_time(time);
			}
			p.show(ui);
			if let Some(time) = p.time()
			{
				self.time = Some(time);
			}
		}
		else
		{
			globe::help::annotated(
				ui,
				"Loading selected site...",
				"Wait for the selected site's validated hourly data. No results are invented while it loads.",
			);
		}
	}
}

// Check catalog/filename invariants without GPU initialization or ephemeris I/O; C++ test namespace.
#[cfg(test)]
mod tests
{
	use super::*;
	// Validate the real prepared ephemerides through the same public ENU conversion used by lighting; C++ integration test.
	#[test]
	fn moon_fixed_ephemeris_libration_ranges()
	{
		// Decode actual hourly data without adding JSON or I/O dependencies to the renderer; C++ test fixture.
		let data: serde_json::Value =
			serde_json::from_str(include_str!("../assets/data/connecting-ridge-c1.json")).unwrap();
		// Use the exact prepared observer position, not a coordinate synthesized to match a range; C++ fixture reads.
		let lat = data["site"]["lat_deg"].as_f64().unwrap();
		// Preserve the same east-longitude convention as production markers; C++ checked fixture access.
		let lon = data["site"]["lon_deg"].as_f64().unwrap();
		// Check both physical ranges and nonconstant evolution over the three-year series; C++ counted regression loop.
		for (body, latitude_limit, longitude_limit) in [("sun", 1.61, 180.0), ("earth", 7.4, 8.4)]
		{
			// Accumulate transformed latitudes for bounds and temporal-span checks; C++ vector<float>.
			let mut latitudes = Vec::new();
			// Accumulate longitudes independently so axis swaps cannot hide; C++ vector<float>.
			let mut longitudes = Vec::new();
			// Visit every aligned hourly pair, not merely one hand-picked date; C++ zipped range iteration.
			for (az, el) in data[body]["az_deg"]
				.as_array()
				.unwrap()
				.iter()
				.zip(data[body]["el_deg"].as_array().unwrap())
			{
				// The same public conversion feeds both real lights and the Earth camera; C++ optional unwrap in a test.
				let direction = globe::moon_fixed(
					lat,
					lon,
					globe::BodyDirection {
						az_deg: az.as_f64().unwrap() as f32,
						el_deg: el.as_f64().unwrap() as f32,
						visible: false,
					},
				)
				.unwrap();
				// Recover latitude from the Moon-fixed north component; C++ asin after a roundoff clamp.
				latitudes.push(direction[1].clamp(-1.0, 1.0).asin().to_degrees());
				// Recover east longitude from the zero-longitude and east axes; C++ atan2.
				longitudes.push(direction[0].atan2(direction[2]).to_degrees());
			}
			// Latitude and longitude bounds catch swapped basis axes or treating local azimuth as longitude; C++ assertions.
			assert!(latitudes.iter().all(|x| x.abs() < latitude_limit));
			// Reject nonphysical lunar libration longitude or a malformed azimuth transform; C++ all_of assertion.
			assert!(longitudes.iter().all(|x| x.abs() <= longitude_limit));
			// Reject a constant placeholder with an otherwise plausible latitude; C++ max/min range assertion.
			assert!(
				latitudes.iter().copied().fold(f32::NEG_INFINITY, f32::max)
					- latitudes.iter().copied().fold(f32::INFINITY, f32::min)
					> if body == "sun" { 2.5 } else { 10.0 }
			);
			// Solar longitude must sweep around the Moon; Earth longitude must librate substantially; C++ range assertion.
			assert!(
				longitudes.iter().copied().fold(f32::NEG_INFINITY, f32::max)
					- longitudes.iter().copied().fold(f32::INFINITY, f32::min)
					> if body == "sun" { 350.0 } else { 10.0 }
			);
		}
	}
	#[test]
	fn catalog_and_lazy_filenames()
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let catalog: Vec<Entry> = serde_json::from_str(include_str!("../../sites.json")).unwrap();
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(catalog.len(), 7);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(catalog[0].name, "Connecting Ridge C1");
		// Bind this intermediate value for the current operation; C++ local initialization.
		let files: std::collections::HashSet<_> =
			catalog.iter().map(|s| filename(&s.name)).collect();
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(files.len(), catalog.len());
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(catalog.iter().all(|s| s.source.contains("https://")
			&& (-90.0..=-85.0).contains(&s.lat_deg)
			&& s.lon_deg.is_finite()));
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(filename("Shackleton  S1"), "shackleton-s1.json");
	}
}

// Implement the framework's application trait; C++ override a virtual application interface.
impl eframe::App for MoonApp
{
	// Rebuild responsive UI each frame; C++ virtual draw callback.
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		self.update_sites(ui);
		self.synchronize_scene();
		// Draw settings before the central view so desktop width and phone drawer are independent; C++ responsive panel call.
		let phone = ui.ctx().content_rect().width() < 760.0;
		self.settings(ui, phone);
		self.central_view(ui);
	}

	// Destroy GPU resources while the framework still has a current context; C++ virtual shutdown callback.
	fn on_exit(&mut self, gl: Option<&eframe::glow::Context>)
	{
		// Only issue GL commands when a context was supplied; C++ if(optional).
		if let Some(gl) = gl
		{
			// Explicit renderer teardown avoids invalid-context destruction; C++ member cleanup call.
			self.globe.destroy(gl);
		}
	}
}
```

### 4.8. app/src/main.rs

```rust
// Hide the console window only in optimized Windows builds; C++ a platform-specific linker subsystem setting.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Compile this entry point only for native targets; C++ #if around main.
#[cfg(not(target_arch = "wasm32"))]
// Return framework startup errors to the runtime; C++ an expected-returning launcher.
fn main() -> eframe::Result
{
	// Initialize environment-controlled native logging; C++ logging-library initialization.
	env_logger::init();
	// Start the native event loop with the unchanged window title; C++ GUI application launcher.
	eframe::run_native(
		"Moon Landing Visual",
		// Configure depth and multisample storage before context creation; C++ window/context descriptor.
		eframe::NativeOptions {
			// Allocate 24-bit depth for sphere visibility; C++ depth buffer request.
			depth_buffer: 24,
			// Retain four coverage samples per pixel; C++ MSAA sample-count request.
			multisampling: 4,
			// Keep all other options at framework defaults; C++ default construction followed by overrides.
			..Default::default()
		},
		// Own a startup closure returning an owned trait object; C++ unique_ptr<App> from a lambda factory.
		Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
	)
}

// Compile a different entry point for the browser; C++ #if for a web build.
#[cfg(target_arch = "wasm32")]
// Start asynchronous browser initialization without blocking JavaScript; C++ browser bridge entry point.
fn main()
{
	// Bring checked JavaScript casts into method resolution without binding a name; C++ a cast-helper interface.
	use eframe::wasm_bindgen::JsCast as _;
	// Schedule a local asynchronous task; C++ a browser coroutine on the main event loop.
	wasm_bindgen_futures::spawn_local(async {
		// Require the browser DOM, reporting missing startup prerequisites explicitly; C++ checked optional extraction.
		let document = web_sys::window()
			.expect("No window")
			.document()
			.expect("No document");
		// Locate and type-check the canvas created by index.html; C++ checked downcast of a DOM object.
		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("Missing canvas")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("Not a canvas");
		// Start eframe on the existing canvas and suspend until initialization completes; C++ co_await startup.
		let result = eframe::WebRunner::new()
			.start(
				canvas,
				// Request a depth buffer while retaining other web defaults; C++ context descriptor.
				eframe::WebOptions {
					// Keep sphere depth precision consistent with native; C++ context depth request.
					depth_buffer: 24,
					// Inherit unspecified framework defaults; C++ default initialization.
					..Default::default()
				},
				// Move the app into a boxed framework trait object; C++ unique_ptr<App> from a factory lambda.
				Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
			)
			.await;
		// Update the initial HTML loading label only if it exists; C++ if(optional).
		if let Some(loading) = document.get_element_by_id("loading_text")
		{
			// Exhaustively handle web startup success/failure; C++ switch over expected's alternatives.
			match result
			{
				// Remove the HTML overlay once egui owns the screen; C++ DOM removal call.
				Ok(()) => loading.remove(),
				// Keep an actionable startup error visible; C++ error alternative.
				Err(err) =>
				{
					// Explain failure in the DOM even if the canvas could not start; C++ textContent assignment.
					loading.set_text_content(Some(
						"The app could not start. See the developer console.",
					));
					// Include detailed startup diagnostics in the browser console; C++ terminate after logging.
					panic!("Failed to start eframe: {err:?}");
				}
			}
		}
	});
}
```

### 4.9. app/index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="utf-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Karaya Explorer</title>
	<link data-trunk rel="rust">
	<link data-trunk rel="copy-dir" href="assets">
	<link data-trunk rel="copy-file" href="../globe/assets/albedo_2k.jpg" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/normal_2k.webp" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/albedo_4k.jpg" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/normal_4k.webp" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/albedo_8k.jpg" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/normal_8k.webp" data-target-path="assets/globe">
	<base data-trunk-public-url>
	<style>
		html, body { margin: 0; width: 100%; height: 100%; overflow: hidden; background: black; }
		canvas { position: absolute; width: 100%; height: 100%; }
		#loading_text { position: absolute; top: 50%; width: 100%; text-align: center; color: white; font-family: sans-serif; }
	</style>
</head>
<body>
	<canvas id="the_canvas_id"></canvas>
	<div id="loading_text">Loading...<noscript>JavaScript is required.</noscript></div>
</body>
</html>
```


The globe separates scene ownership (`lib.rs`), interaction and projection (`view.rs`), camera controls (`camera.rs`), GPU drawing (`renderer.rs`), construction (`renderer/setup.rs`), and texture upload (`renderer/textures.rs`). Application layout and panorama drawing are independent modules. Function spans include comments and blank lines: the longest Rust function changed from 362 to 64 lines; production functions are at most 60 lines. The 64-line function is an unchanged test. Numerical operations, input defaults and draw ordering are unchanged.

### 4.20. globe/src/renderer.rs

```rust
mod setup;
mod textures;

use super::glow;
use super::{Lighting, Ray, assets, field, math};
use glow::HasContext as _;

// Declare named fields; Rust methods live in a separate impl, unlike inline C++ class definitions.
pub(super) struct Renderer
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
	// Record the quality actually uploaded, independently of the requested upgrade; C++ unsigned member.
	pub(super) tier: u32,
	// Signal only after the renderer has drawn with real sky directions; C++ bool.
	pub(super) visible_announced: bool,
	// Retain decoded upgrades between bounded GPU-upload frames; C++ optional pair with image iterator.
	pending: Option<(u32, std::vec::IntoIter<assets::TextureImage>)>,
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Renderer
{
	// Begin a new quality request while retaining the last complete pair; C++ cancellation and staging.
	pub(super) fn begin_upgrade(&mut self, gl: &glow::Context, images: [assets::TextureImage; 2])
	{
		// Discard only abandoned replacement textures, never the visible pair; C++ GL owner cleanup.
		for texture in self.textures.drain(2..)
		{
			unsafe {
				gl.delete_texture(texture);
			}
		}
		// Store the real image width and consume the pair in order; C++ pair<unsigned, image iterator>.
		self.pending = Some((images[0].width() / 1024, Vec::from(images).into_iter()));
	}

	// Advance one native/browser upload per frame, then atomically expose the complete pair; C++ state machine.
	pub(super) fn advance_upgrade(&mut self, gl: &glow::Context) -> Result<bool, String>
	{
		// Temporarily own the in-progress upload, restoring it only after success; C++ optional exchange.
		let Some((tier, mut images)) = self.pending.take()
		else
		{
			return Ok(false);
		};
		// Process only one image in this frame; C++ iterator increment and optional extraction.
		if let Some(image) = images.next()
		{
			// The first replacement is albedo, the second is directional data; C++ bool.
			let srgb = self.textures.len() == 2;
			#[cfg(not(target_arch = "wasm32"))]
			let result =
				self.upload_texture(gl, image.width(), image.height(), image.as_raw(), srgb);
			#[cfg(target_arch = "wasm32")]
			let result = self.upload_browser_texture(gl, &image, srgb);
			if let Err(error) = result
			{
				// SAFETY: this callback owns the current GL context; C++ delete only registered replacement handles.
				for texture in self.textures.drain(2..)
				{
					unsafe {
						gl.delete_texture(texture);
					}
				}
				return Err(error);
			}
			// Resume at the next image in a later frame; C++ iterator move into optional.
			self.pending = Some((tier, images));
			return Ok(true);
		}
		// SAFETY: old textures are no longer sampled after this callback-local swap; C++ owner retirement.
		for texture in self.textures.drain(..2)
		{
			unsafe {
				gl.delete_texture(texture);
			}
		}
		// Expose the completed pair and schedule a new actual-visibility diagnostic; C++ member assignments.
		self.tier = tier;
		self.visible_announced = false;
		Ok(false)
	}

	// Draw one opaque sphere inside egui's already configured viewport; C++ issues the equivalent GL commands.
	pub(super) fn paint(
		&self,
		gl: &glow::Context,
		view: math::View,
		rays: &[([f32; 3], [Ray; 2])],
		lighting: Option<Lighting>,
		surface_rays: bool,
	)
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
			Self::begin_frame(gl, program);
			Self::light_uniforms(gl, program, lighting);
			self.paint_opaque(gl, program, view);
			// Smooth alpha coverage needs blending on native and WebGL; C++ glEnable/glBlendFunc.
			gl.enable(glow::BLEND);
			gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
			gl.depth_mask(false);
			if surface_rays
			{
				self.paint_surface_rays(gl, program, view, lighting);
			}
			self.paint_site_rays(gl, program, view, rays);
			// Restore write/blend policy after transparent diagrams; C++ GL state restoration.
			gl.depth_mask(true);
			gl.disable(glow::BLEND);
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

	// Batch the entire Sun field into one cheap draw; C++ dynamic interleaved vertex vector.
	fn paint_surface_rays(
		&self,
		gl: &glow::Context,
		program: glow::Program,
		view: math::View,
		lighting: Option<Lighting>,
	)
	{
		let Some(light) = lighting
		else
		{
			return;
		};
		let mut vertices = Vec::new();
		for point in field::illuminated_points(light.sun)
		{
			let origin = std::array::from_fn(|i| point[i] + 0.6 / view.zoom * light.sun[i]);
			let mut ribbon = view.ribbon(origin, point);
			ribbon.truncate(42);
			vertices.extend(ribbon);
		}
		// SAFETY: the callback owns the current GL context; the ray layout matches seven floats per vertex.
		unsafe {
			gl.enable(glow::DEPTH_TEST);
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 1);
			gl.uniform_3_f32(
				gl.get_uniform_location(program, "u_ray_color").as_ref(),
				1.0,
				0.9,
				0.05,
			);
			gl.bind_vertex_array(self.ray_vao);
			gl.bind_buffer(glow::ARRAY_BUFFER, self.ray_buffer);
			gl.buffer_data_u8_slice(
				glow::ARRAY_BUFFER,
				bytemuck::cast_slice(&vertices),
				glow::STREAM_DRAW,
			);
			gl.disable(glow::CULL_FACE);
			gl.draw_arrays(glow::TRIANGLES, 0, (vertices.len() / 7) as i32);
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
	}

	// Release registered GPU objects using the current context; C++ explicit teardown avoids a late invalid-context destructor.
	pub(super) fn destroy(&mut self, gl: &glow::Context)
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

impl Renderer
{
	unsafe fn light_uniforms(gl: &glow::Context, program: glow::Program, lighting: Option<Lighting>)
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// Supply harmless finite directions when ephemerides are absent, but no invented sunlight; C++ optional fallback.
			let light = lighting.unwrap_or(Lighting {
				sun: [0.0, 0.0, 1.0],
				earth: [0.0, 0.0, 1.0],
				ratio: 0.0,
				boosted: false,
				fill: false,
			});
			// Send both Moon-fixed light directions into the shared model transform; C++ uniform vector loop.
			for (name, direction) in [("u_sun", light.sun), ("u_earth", light.earth)]
			{
				gl.uniform_3_f32(
					gl.get_uniform_location(program, name).as_ref(),
					direction[0],
					direction[1],
					direction[2],
				);
			}
			// Keep physical illumination and artificial display aids separate; C++ scalar uniforms.
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_sun_scale").as_ref(),
				if lighting.is_some() { 1.0 } else { 0.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_earthshine").as_ref(),
				light.ratio * if light.boosted { 1000.0 } else { 1.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_far_fill").as_ref(),
				if light.fill { 1.0 } else { 0.0 },
			);
		}
	}

	unsafe fn paint_site_rays(
		&self,
		gl: &glow::Context,
		program: glow::Program,
		view: math::View,
		rays: &[([f32; 3], [Ray; 2])],
	)
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// These are direction diagrams, not physical beams: retain blocked rays on the visible surface; C++ depth-state change.
			gl.disable(glow::DEPTH_TEST);
			// Draw real object-space direction lines against the sphere's existing depth buffer; C++ optional guard.
			for &(origin, bodies) in rays
			{
				// Never expose diagrams from the back of the Moon; C++ visibility guard.
				if view.project(origin, true).is_none()
				{
					continue;
				}
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
						std::array::from_fn(|i| origin[i] + 0.6 / view.zoom * body.direction[i]);
					// Build screen-width triangle ribbons and a circular tip; C++ dynamic interleaved vertex array.
					let vertices = view.ribbon(origin, end);
					// Stream the ribbon and endpoint triangles without persistent mapping; C++ glBufferData.
					gl.buffer_data_u8_slice(
						glow::ARRAY_BUFFER,
						bytemuck::cast_slice(&vertices),
						glow::STREAM_DRAW,
					);
					// Dim failed terrain/radio predicates on the visible-side direction diagram; C++ conditional scalar.
					let strength = if body.visible { 1.0 } else { 0.35 };
					// Send an unlit display color rather than illuminating rays with the globe's illustration light; C++ glUniform3f.
					gl.uniform_3_f32(
						gl.get_uniform_location(program, "u_ray_color").as_ref(),
						color[0] * strength,
						color[1] * strength,
						color[2] * strength,
					);
					// Draw both ribbon faces and the endpoint fan as portable triangles; C++ glDrawArrays(GL_TRIANGLES).
					gl.disable(glow::CULL_FACE);
					gl.draw_arrays(glow::TRIANGLES, 0, (vertices.len() / 7) as i32);
				}
				// Do not leak a dynamic buffer binding into egui; C++ glBindBuffer(...,0).
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
		}
	}
}

impl Renderer
{
	unsafe fn begin_frame(gl: &glow::Context, program: glow::Program)
	{
		// SAFETY: the calling paint callback owns the current context; C++ has the same contract.
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
		}
	}
}

impl Renderer
{
	unsafe fn paint_opaque(&self, gl: &glow::Context, program: glow::Program, view: math::View)
	{
		// SAFETY: the calling paint callback owns the current context; C++ has the same contract.
		unsafe {
			// Always restore the textured material before drawing the Moon; C++ glUniform1i.
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 0);
			// Send a column-major projection without transposing; C++ glUniformMatrix4fv.
			gl.uniform_matrix_4_f32_slice(
				// Look up an active uniform, returning None if absent; C++ glGetUniformLocation returns -1.
				gl.get_uniform_location(program, "u_projection").as_ref(),
				false,
				&view.projection(),
			);
			// Share one column-major rigid model matrix with geometry, normals, and CPU marker projection; C++ glUniformMatrix3fv.
			gl.uniform_matrix_3_f32_slice(
				gl.get_uniform_location(program, "u_model").as_ref(),
				false,
				&view.model(),
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
		}
	}
}
```

### 4.21. globe/src/renderer/setup.rs

```rust
use super::{Renderer, assets, glow, math};
use glow::HasContext as _;

impl Renderer
{
	// Construct and validate owned state, returning an error instead of throwing; C++ uses a factory returning expected.
	pub(crate) fn new(
		gl: &glow::Context,
		[albedo, normals]: [assets::TextureImage; 2],
	) -> Result<Self, String>
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
		let mut renderer = Self::empty(albedo.width() / 1024);
		let initialized = renderer.initialize(gl, version, albedo, normals);
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

	fn empty(tier: u32) -> Self
	{
		Self {
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
			tier,
			visible_announced: false,
			pending: None,
		}
	}

	fn initialize(
		&mut self,
		gl: &glow::Context,
		version: egui_glow::ShaderVersion,
		albedo: assets::TextureImage,
		normals: assets::TextureImage,
	) -> Result<(), String>
	{
		// Run fallible GPU construction in a local closure so partial resources can be cleaned; C++ try/catch is analogous.
		(|| -> Result<(), String> {
			// SAFETY: eframe calls the constructor with its GL context current. Every handle is
			// registered immediately so the error branch can clean partial construction.
			// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
			unsafe {
				self.build_program(gl, version)?;
				self.build_sphere(gl)?;
				self.build_ray_layout(gl)?;
			}
			// Upload each image once; the browser keeps decoded pixels in its native image source.
			for (image, srgb) in [(albedo, true), (normals, false)]
			{
				#[cfg(not(target_arch = "wasm32"))]
				self.upload_texture(gl, image.width(), image.height(), image.as_raw(), srgb)?;
				#[cfg(target_arch = "wasm32")]
				self.upload_browser_texture(gl, &image, srgb)?;
			}
			// Wrap a successful result without throwing; C++ expected stores its value.
			Ok(())
		})()
	}

	unsafe fn build_program(
		&mut self,
		gl: &glow::Context,
		version: egui_glow::ShaderVersion,
	) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// Allocate a linkable program; C++ glCreateProgram returns a GLuint, while glow can return an error.
			let program = gl.create_program()?;
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			self.program = Some(program);
			self.compile_stages(gl, program, version)?;
			// Fix the input location to match the vertex layout; C++ glBindAttribLocation before linking.
			gl.bind_attrib_location(program, 0, "a_position");
			// Fix the input location to match the vertex layout; C++ glBindAttribLocation before linking.
			gl.bind_attrib_location(program, 1, "a_uv");
			gl.bind_attrib_location(program, 2, "a_coverage");
			// Connect vertex outputs to fragment inputs; C++ glLinkProgram.
			gl.link_program(program);
			// Reject incompatible stages; C++ glGetProgramiv with GL_LINK_STATUS.
			if !gl.get_program_link_status(program)
			{
				// Recover the linker's diagnostic; C++ glGetProgramInfoLog.
				return Err(gl.get_program_info_log(program));
			}
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for shader in self.shaders.drain(..)
			{
				// Release the program's stage attachment after a successful link; C++ glDetachShader.
				gl.detach_shader(program, shader);
				// Delete the no-longer-needed shader object; C++ glDeleteShader.
				gl.delete_shader(shader);
			}
		}
		Ok(())
	}

	unsafe fn compile_stages(
		&mut self,
		gl: &glow::Context,
		program: glow::Program,
		version: egui_glow::ShaderVersion,
	) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for (kind, source) in [
				// Embed only the vertex shader's small UTF-8 source, never image tiers; C++ embeds a string literal.
				(glow::VERTEX_SHADER, include_str!("../moon.vert")),
				// Embed the matching fragment shader; C++ embeds another shader source string.
				(glow::FRAGMENT_SHADER, include_str!("../moon.frag")),
			]
			{
				// Allocate one shader stage; C++ glCreateShader takes the same stage enum.
				let shader = gl.create_shader(kind)?;
				// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
				self.shaders.push(shader);
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
		}
		Ok(())
	}

	unsafe fn build_sphere(&mut self, gl: &glow::Context) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// Own interleaved position/UV values on the CPU; C++ vector<float>.
			let (vertices, indices) = math::sphere(256, 128);
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			self.index_count = indices.len() as i32;
			// Allocate vertex-layout state; C++ glGenVertexArrays.
			self.vao = Some(gl.create_vertex_array()?);
			// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
			gl.bind_vertex_array(self.vao);
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
				self.buffers.push(buffer);
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
		}
		Ok(())
	}

	unsafe fn build_ray_layout(&mut self, gl: &glow::Context) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// Allocate a second VAO without disturbing the Moon's indexed mesh; C++ glGenVertexArrays.
			self.ray_vao = Some(gl.create_vertex_array()?);
			// Configure only this layout for position triples; C++ glBindVertexArray.
			gl.bind_vertex_array(self.ray_vao);
			// Allocate a tiny dynamic buffer for four ray vertices; C++ glGenBuffers.
			let ray_buffer = gl.create_buffer()?;
			// Register ownership immediately for cleanup on later failure; C++ owned handle vector append.
			self.buffers.push(ray_buffer);
			// Retain a nonowning handle for subsequent uploads; C++ handle copy.
			self.ray_buffer = Some(ray_buffer);
			// Make the new buffer the source of position attribute zero; C++ glBindBuffer.
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(ray_buffer));
			// Enable only position; unused UV defaults to zero for the constant-color branch; C++ glEnableVertexAttribArray.
			gl.enable_vertex_attrib_array(0);
			// Three tightly packed floats describe each ray vertex; C++ glVertexAttribPointer.
			gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 28, 0);
			gl.enable_vertex_attrib_array(1);
			gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 28, 12);
			gl.enable_vertex_attrib_array(2);
			gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 28, 20);
			// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
			gl.bind_vertex_array(None);
			// Select the buffer receiving the next operation; C++ glBindBuffer(target, handle or 0).
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
		Ok(())
	}
}
```

### 4.22. globe/src/renderer/textures.rs

```rust
use super::{Renderer, glow};
use glow::HasContext as _;

impl Renderer
{
	// Upload RGB bytes and configure mipmapped sampling; C++ receives a const byte span plus dimensions.
	#[cfg(not(target_arch = "wasm32"))]
	pub(super) fn upload_texture(
		&mut self,
		gl: &glow::Context,
		width: u32,
		height: u32,
		pixels: &[u8],
		srgb: bool,
	) -> Result<(), String>
	{
		// Pixel storage is converted only after allocation.
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
			Self::native_pixels(gl, width, height, pixels, srgb);
			Self::native_filter(gl);
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

	// Upload browser-decoded pixels directly, without a WASM RGB buffer; C++ DOM image upload.
	#[cfg(target_arch = "wasm32")]
	pub(super) fn upload_browser_texture(
		&mut self,
		gl: &glow::Context,
		image: &web_sys::HtmlImageElement,
		srgb: bool,
	) -> Result<(), String>
	{
		// SAFETY: callback runs on the current GL context and the image's creation thread.
		unsafe {
			// Register ownership before uploading, so construction failure cleans it; C++ owned GLuint.
			let texture = gl.create_texture()?;
			// Register cleanup ownership before using the handle; C++ vector::push_back.
			self.textures.push(texture);
			// Select the upload unit explicitly; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Bind this replacement without modifying the visible texture pair; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Normal maps are data, not browser color-corrected display images; C++ pixel-store policy.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				glow::NONE as i32,
			);
			// Keep the original top-row order, matching the native byte upload; C++ pixel-store policy.
			gl.pixel_store_i32(web_sys::WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
				0,
			);
			// Upload the browser-owned decoded source without copying through WASM; C++ DOM texture source.
			gl.tex_image_2d_with_html_image(
				glow::TEXTURE_2D,
				0,
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				image,
			);
			Self::browser_filter(gl);
			// Restore the WebGL default expected by other eframe painters; C++ state restoration.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				web_sys::WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
			);
			// Leave no upload binding behind; C++ glBindTexture with zero.
			gl.bind_texture(glow::TEXTURE_2D, None);
			// Convert driver failures into the normal recoverable loader error; C++ glGetError.
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Browser texture upload GL error {error}"));
			}
		}
		Ok(())
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn native_pixels(gl: &glow::Context, width: u32, height: u32, pixels: &[u8], srgb: bool)
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

		// SAFETY: the caller has bound an owned texture on the current context.
		unsafe {
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
		}
	}

	#[cfg(not(target_arch = "wasm32"))]
	unsafe fn native_filter(gl: &glow::Context)
	{
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
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
		}
	}

	#[cfg(target_arch = "wasm32")]
	unsafe fn browser_filter(gl: &glow::Context)
	{
		// SAFETY: called only while a texture is bound in the renderer's current context.
		unsafe {
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
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
			gl.generate_mipmap(glow::TEXTURE_2D);
		}
	}
}
```

### 4.23. globe/src/view.rs

```rust
use super::*;

impl Globe
{
	/// Fill the available UI rectangle with interactive controls and optional one-minute rotation.
	// Queue a paint callback for this frame and request the next; C++ an immediate-mode UI calls this each frame.
	pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize>
	{
		self.loading_status(ui);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let time = ui.input(|i| i.time);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dt = (time - self.last_time.replace(time).unwrap_or(time)).clamp(0.0, 0.1) as f32;
		// Bind this intermediate value for the current operation; C++ local initialization.
		let _ = time;
		let lighting = self.scene_lighting(ui);
		self.animate_camera(time, dt);
		self.pending_label(ui);
		// Reserve the available paint rectangle; C++ would receive a rectangle/response pair.
		let (rect, response) =
			ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if !rect.is_positive()
		{
			// Stop this frame's work without a value; C++ return exits a void function.
			return None;
		}
		self.orbit_input(ui, &response);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let scale = ui.ctx().pixels_per_point();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let view = math::View {
			yaw: self.yaw,
			tilt: self.tilt,
			zoom: self.zoom,
			aspect: rect.aspect_ratio(),
			pixels: [rect.width() * scale, rect.height() * scale],
			scale,
		};
		self.queue_gl(ui, rect, view, lighting);
		let clicked = self.paint_markers(ui, &response, rect, view);
		Self::legend(ui, rect);
		// Request another frame so loading and animation progress; C++ asks the event loop to redraw.
		ui.ctx().request_repaint();
		clicked
	}
}

impl Globe
{
	fn loading_status(&mut self, ui: &mut egui::Ui)
	{
		// Keep direct manipulation discoverable when the settings group is closed; C++ explanatory heading row.
		ui.horizontal(|ui| {
			ui.small("Moon overview");
			help::question(ui, "Drag to orbit the Moon; wheel or pinch to zoom. Magnification stops at 8 times to avoid misleading polar texture streaks. Markers are published points; click one or use Site. This globe is an illustration, not the detailed terrain silhouette shown in the panorama.");
		});
		// Display 2k before starting an optional upgrade; never blank the existing renderer while waiting.
		if self
			.renderer
			.lock()
			.as_ref()
			.is_some_and(|r| r.visible_announced)
			&& self.tier != self.target_tier
		{
			// Start one pair per explicit target change, with UI wakeups on completion; C++ state transition.
			self.tier = self.target_tier;
			self.loading = assets::load(self.tier, Some(ui.ctx().clone()));
		}
		// A failed upgrade leaves the last successful Moon on screen; C++ optional diagnostic display.
		if let Some(error) = self.loading.lock().error.as_ref()
		{
			help::annotated(
				ui,
				format!("Texture detail unavailable: {error}"),
				"The last successful Moon remains visible if higher detail fails. Try another detail level in View / camera after an upgrade fails, or reload the page if the first images fail. Scientific data is independent of globe textures.",
			);
		}
	}

	fn scene_lighting(&mut self, ui: &mut egui::Ui) -> Option<Lighting>
	{
		// Copy the selected site's actual directions, never infer illumination from availability flags; C++ optional transform.
		let directions = self
			.scene
			.iter()
			.find(|s| s.id == self.selected)
			.and_then(|s| {
				// Missing or invalid tracks have no real illumination fallback; C++ optional propagation.
				let [sun, earth] = s.directions?;
				// Propagate either invalid sky direction without inventing a light; C++ optional-return aggregate.
				Some([
					moon_fixed(s.lat_deg, s.lon_deg, sun)?,
					moon_fixed(s.lat_deg, s.lon_deg, earth)?,
				])
			});
		// Holding this mode reveals real libration rather than artificial animation; C++ conditional camera update.
		if self.earth_view
			&& let Some([_, earth]) = directions
		{
			// Cancel longitude so the Earth direction projects onto the camera's central meridian; C++ atan2.
			self.yaw = -earth[0].atan2(earth[2]);
			// Match camera latitude to sub-Earth latitude without a fixed presentation tilt; C++ asin.
			self.tilt = earth[1].clamp(-1.0, 1.0).asin();
		}
		// Pass an explicit absent light state while the selected track is unavailable; C++ optional snapshot.
		let lighting = directions.map(|[sun, earth]| Lighting {
			sun,
			earth,
			ratio: math::earthshine_ratio(sun, earth),
			boosted: self.boosted,
			fill: self.fill,
		});
		// Explain unavailable data rather than silently restoring a fixed presentation Sun; C++ conditional label.
		if lighting.is_none()
		{
			// State why the Moon has no physical illumination instead of fabricating data; C++ immediate-mode text.
			ui.label("Real lighting unavailable: selected track not loaded");
		}
		lighting
	}

	fn animate_camera(&mut self, time: f64, dt: f32)
	{
		// Check this state or input before acting; C++ conditional branch.
		if let Some((start, from, to)) = self.focus
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let t = ((time - start) / 0.8).clamp(0.0, 1.0) as f32;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let blend = t * t * (3.0 - 2.0 * t);
			// Update this persistent member explicitly; C++ member assignment.
			self.yaw = from[0] + (to[0] - from[0]) * blend;
			// Update this persistent member explicitly; C++ member assignment.
			self.tilt = from[1] + (to[1] - from[1]) * blend;
			// Update this persistent member explicitly; C++ member assignment.
			self.zoom = (from[2].ln() + (to[2].ln() - from[2].ln()) * blend).exp();
			// Check this state or input before acting; C++ conditional branch.
			if t >= 1.0
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.focus = None;
			}
		}
		else if !self.paused
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.yaw += dt * std::f32::consts::TAU / 60.0;
		}
	}

	fn pending_label(&self, ui: &mut egui::Ui)
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
	}

	fn orbit_input(&mut self, ui: &mut egui::Ui, response: &egui::Response)
	{
		// Wheel and touch pinch operate only inside the globe, not the panorama scroll area; C++ scoped event handling.
		if response.hovered()
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let (wheel, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
			// Check this state or input before acting; C++ conditional branch.
			if wheel != 0.0 || pinch != 1.0
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.zoom = (self.zoom * (wheel * 0.003).exp() * pinch).clamp(1.0, MAX_ZOOM);
				// Update this persistent member explicitly; C++ member assignment.
				self.focus = None;
				// Consume globe wheel input before the surrounding page scroll area; C++ event consumption.
				ui.input_mut(|i| i.smooth_scroll_delta.y = 0.0);
			}
		}
		// Check this state or input before acting; C++ conditional branch.
		if response.dragged()
		{
			// Direct orbit manipulation exits Earth lock while preserving the current orientation; C++ mode switch.
			self.earth_view = false;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let delta = ui.input(|i| i.pointer.delta());
			// Update this persistent member explicitly; C++ member assignment.
			self.yaw += delta.x * 0.006 / self.zoom;
			// Update this persistent member explicitly; C++ member assignment.
			self.tilt = (self.tilt + delta.y * 0.006 / self.zoom)
				.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
			// Update this persistent member explicitly; C++ member assignment.
			self.paused = true;
			// Update this persistent member explicitly; C++ member assignment.
			self.focus = None;
		}
	}

	fn queue_gl(
		&self,
		ui: &mut egui::Ui,
		rect: egui::Rect,
		view: math::View,
		lighting: Option<Lighting>,
	)
	{
		// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
		let renderer = Arc::clone(&self.renderer);
		// Keep the loader alive until this callback finishes; C++ copies its shared_ptr.
		let loading = Arc::clone(&self.loading);
		let rays = self.scene_rays();
		// Capture shared state for egui's later current-context draw; C++ stores a value-capturing std::function.
		let surface_rays = self.surface_rays;
		let ctx = ui.ctx().clone();
		let callback = egui_glow::CallbackFn::new(move |info, painter| {
			// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
			let mut renderer = renderer.lock();
			Self::process_textures(&mut renderer, &loading, &ctx, painter.gl());
			// Convert egui's logical rectangle to actual pixel dimensions; C++ receives integer width/height.
			let viewport = info.viewport_in_pixels();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if viewport.width_px > 0 && viewport.height_px > 0
			{
				// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
				if let Some(renderer) = renderer.as_mut()
				{
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.paint(painter.gl(), view, &rays, lighting, surface_rays);
					Self::announce(renderer, lighting, &ctx);
				}
			}
		});
		// Append the custom GL callback to egui's paint list; C++ queues an owned rendering command.
		ui.painter().add(egui::PaintCallback {
			rect,
			callback: Arc::new(callback),
		});
	}

	fn paint_markers(
		&self,
		ui: &mut egui::Ui,
		response: &egui::Response,
		rect: egui::Rect,
		view: math::View,
	) -> Option<usize>
	{
		// Retain the nearest hit in the same marker order; C++ optional identifier.
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut clicked = None;
		// Selected markers are larger; every marker has a contrasting outline and nearest-hit selection; C++ projected hit testing.
		let painter = ui.painter_at(rect);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let pointer = response.interact_pointer_pos();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut nearest: f32 = f32::INFINITY;
		// Visit each value without transferring the owner's resources; C++ range-for loop.
		for site in self
			.scene
			.iter()
			.filter(|s| s.id != self.selected)
			.chain(self.scene.iter().filter(|s| s.id == self.selected))
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let point = math::direction(
				(site.lat_deg as f32).to_radians(),
				(site.lon_deg as f32).to_radians(),
			);
			// Check this state or input before acting; C++ conditional branch.
			if let Some(ndc) = view.project(point, true)
			{
				// Bind this intermediate value for the current operation; C++ local initialization.
				let position = rect.center()
					+ egui::vec2(ndc[0] * rect.width() * 0.5, -ndc[1] * rect.height() * 0.5);
				if !rect.expand(15.0).contains(position)
				{
					continue;
				}
				// Bind this intermediate value for the current operation; C++ local initialization.
				let selected = site.id == self.selected;
				Self::draw_marker(&painter, position, site, selected);
				// Check this state or input before acting; C++ conditional branch.
				if response.clicked()
					&& let Some(p) = pointer
				{
					// Bind this intermediate value for the current operation; C++ local initialization.
					let distance = p.distance_sq(position);
					// Check this state or input before acting; C++ conditional branch.
					if distance < 14.0_f32.powi(2) && distance < nearest
					{
						nearest = distance;
						clicked = Some(site.id);
					}
				}
			}
		}
		clicked
	}

	fn legend(ui: &mut egui::Ui, rect: egui::Rect)
	{
		let painter = ui.painter_at(rect);
		// Keep the legend readable when the optional field crosses it; C++ translucent overlay rectangle.
		painter.rect_filled(
			egui::Rect::from_min_size(
				rect.left_top() + egui::vec2(5.0, 5.0),
				egui::vec2(258.0, 59.0),
			),
			3.0,
			egui::Color32::from_black_alpha(225),
		);
		painter.text(rect.left_top()+egui::vec2(8.0,8.0),egui::Align2::LEFT_TOP,"Drag to orbit; wheel / pinch to zoom\nYellow Sun | blue Earth | dim = blocked\nReal light; fill is a visual aid; rays not to scale\nPolar cap schematic; terrain detail in panorama",egui::FontId::proportional(11.0),egui::Color32::LIGHT_GRAY);
	}
}

impl Globe
{
	fn scene_rays(&self) -> Vec<([f32; 3], [Ray; 2])>
	{
		// Snapshot the selected sample for the deferred GL callback; C++ copy small optional values into a lambda.
		let rays: Vec<_> = self
			.scene
			.iter()
			.filter_map(|site| {
				// Bind this intermediate value for the current operation; C++ local initialization.
				let origin = math::direction(
					(site.lat_deg as f32).to_radians(),
					(site.lon_deg as f32).to_radians(),
				);
				// Bind this intermediate value for the current operation; C++ local initialization.
				let bodies = site.directions?;
				// Bind this intermediate value for the current operation; C++ local initialization.
				let convert = |body: BodyDirection| {
					math::local_direction(
						(site.lat_deg as f32).to_radians(),
						(site.lon_deg as f32).to_radians(),
						body.az_deg.to_radians(),
						body.el_deg.to_radians(),
					)
					.map(|direction| Ray {
						direction,
						visible: body.visible,
					})
				};
				Some((origin, [convert(bodies[0])?, convert(bodies[1])?]))
			})
			.collect();
		rays
	}

	fn process_textures(
		renderer: &mut Option<Renderer>,
		loading: &Mutex<assets::Loading>,
		ctx: &egui::Context,
		gl: &glow::Context,
	)
	{
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if loading.lock().error.is_none()
		{
			// Consume the completed pair without rebuilding an existing mesh or shader; C++ optional result.
			let result = loading.lock().take();
			match result
			{
				Ok(Some(images)) =>
				{
					if let Some(current) = renderer.as_mut()
					{
						// Keep old textures until every replacement is uploaded; C++ staged owner transfer.
						current.begin_upgrade(gl, images);
					}
					else
					{
						match Renderer::new(gl, images)
						{
							Ok(ready) => *renderer = Some(ready),
							Err(error) => loading.lock().error = Some(error),
						}
					}
				}
				Ok(None) =>
				{}
				Err(error) => loading.lock().error = Some(error),
			}
		}
		// Upload at most one replacement image per frame, leaving the last Moon usable; C++ incremental task.
		if let Some(current) = renderer.as_mut()
		{
			match current.advance_upgrade(gl)
			{
				Ok(true) => ctx.request_repaint(),
				Ok(false) =>
				{}
				Err(error) => loading.lock().error = Some(error),
			}
		}
	}

	fn announce(renderer: &mut Renderer, lighting: Option<Lighting>, ctx: &egui::Context)
	{
		// Announce a real lit draw, not merely decoded textures with pending sky data; C++ frame diagnostic.
		if lighting.is_some() && !renderer.visible_announced
		{
			let status = format!("Moon visible: {}k", renderer.tier);
			#[cfg(target_arch = "wasm32")]
			web_sys::console::log_1(&status.into());
			#[cfg(not(target_arch = "wasm32"))]
			eprintln!("{status}");
			renderer.visible_announced = true;
			// Give the first visible frame priority before requesting an upgrade; C++ event-loop wake.
			ctx.request_repaint();
		}
	}

	fn draw_marker(painter: &egui::Painter, position: egui::Pos2, site: &Marker, selected: bool)
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let radius = if selected { 9.0 } else { 6.0 };
		painter.circle_filled(
			position,
			radius,
			if selected
			{
				egui::Color32::LIGHT_RED
			}
			else
			{
				egui::Color32::from_rgb(70, 230, 160)
			},
		);
		painter.circle_stroke(
			position,
			radius,
			egui::Stroke::new(2.0, egui::Color32::WHITE),
		);
		if selected
		{
			painter.text(
				position + egui::vec2(12.0, -12.0),
				egui::Align2::LEFT_BOTTOM,
				&site.name,
				egui::FontId::proportional(13.0),
				egui::Color32::WHITE,
			);
		}
	}
}
```

### 4.24. globe/src/camera.rs

```rust
use super::*;

impl Globe
{
	// Draw camera controls independently of the illustration; C++ member function.
	pub fn camera_controls(&mut self, ui: &mut egui::Ui)
	{
		// Apply a texture-resolution limit instead of inventing polar detail; C++ bounded edits.
		ui.horizontal_wrapped(|ui| {
			if ui.button("- Zoom").clicked() { self.zoom = (self.zoom / 1.8).max(1.0); self.focus = None; }
			help::question(ui, "Zoom out to see more of the Moon. This changes the camera, not the landing site.");
			if ui.button("+ Zoom").clicked() { self.zoom = (self.zoom * 1.8).min(MAX_ZOOM); self.focus = None; }
			help::question(ui, "Zoom in, up to 8 times. Higher magnification enlarges misleading polar texture streaks. Choose close neighbors in Site; the panorama uses the detailed terrain horizon.");
		});
		self.site_camera(ui);
		// Require real Earth directions before entering Earth-following mode; C++ optional guard.
		let ready = self
			.scene
			.iter()
			.any(|s| s.id == self.selected && s.directions.is_some());
		ui.horizontal(|ui| {
			if ui.add_enabled(ready, egui::Button::new("View from Earth")).clicked()
			{ self.earth_view = true; self.paused = true; self.zoom = 1.0; self.focus = None; }
			help::question(ui, "Look along the current Earth direction. The apparent rocking changes with time. This is a direction-correct illustration, not a camera at Earth's true distance.");
		});
		// Keep synthetic camera spin optional; C++ bool edit.
		let mut spin = !self.paused;
		ui.horizontal(|ui| {
			if ui.checkbox(&mut spin, "Auto-rotate").changed() { self.paused = !spin; self.earth_view = false; self.focus = None; }
			help::question(ui, "Turn the camera slowly for inspection. This artificial one-minute spin is not a real orbital period and does not change the scientific date.");
		});
		// Identify the mode and magnification without hidden controls; C++ label.
		ui.label(format!(
			"{:.1}x | {}",
			self.zoom,
			if self.earth_view
			{
				"Earth-following view"
			}
			else
			{
				"Free orbit"
			}
		));
		self.texture_controls(ui);
	}

	// Draw physical-light display aids separately from camera actions; C++ member function.
	pub fn lighting_controls(&mut self, ui: &mut egui::Ui)
	{
		// Explain that increased exposure is not increased power; C++ checkbox row.
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.boosted, "Earthshine boosted (1000x)");
			help::question(ui, "Earthshine is sunlight reflected by Earth onto the Moon. It is normally thousands of times weaker than sunlight. This exposure boost changes only the illustration, not solar power or communication.");
		});
		// Keep the nonphysical aid explicitly labeled; C++ checkbox row.
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.fill, "Far side fill light");
			help::question(ui, "Soft gray-blue light helps you inspect the mean far side after dragging around the Moon. It is a visual aid, not real light, and changes no scientific result.");
		});
	}

	// Expose the independent surface field without requesting more site tracks; C++ checkbox.
	pub fn surface_ray_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.surface_rays, "Rays across the whole Moon");
			help::question(ui, "Parallel yellow rays arrive from the real Sun direction at this date and end on the sunlit sphere. About 256 of 512 evenly distributed points are lit. Lengths are illustrative; this is not terrain shadow tracing. The Moon hides rays behind its surface.");
		});
	}
}

impl Globe
{
	fn site_camera(&mut self, ui: &mut egui::Ui)
	{
		// Request an eased transition at the real UI clock time; C++ event flag.
		let go = ui.horizontal(|ui| {
			let clicked = ui.button("Go to site").clicked();
			help::question(ui, "Center the selected point and zoom in. Close published points can overlap on this overview; select them individually in Site.");
			clicked
		}).inner;
		// Animation time is independent of the scientific date; C++ double local.
		let time = ui.input(|i| i.time);
		if go && let Some(site) = self.scene.iter().find(|s| s.id == self.selected)
		{
			// Site-centered viewing is a free camera, not an Earth lock; C++ camera mode switch.
			self.earth_view = false;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let target = -(site.lon_deg as f32).to_radians();
			// Bind this intermediate value for the current operation; C++ local initialization.
			let delta = (target - self.yaw + std::f32::consts::PI)
				.rem_euclid(std::f32::consts::TAU)
				- std::f32::consts::PI;
			// Update this persistent member explicitly; C++ member assignment.
			self.focus = Some((
				time,
				[self.yaw, self.tilt, self.zoom],
				[
					self.yaw + delta,
					(site.lat_deg as f32).to_radians(),
					MAX_ZOOM,
				],
			));
			// Update this persistent member explicitly; C++ member assignment.
			self.paused = true;
		}
	}

	fn texture_controls(&mut self, ui: &mut egui::Ui)
	{
		// Let desktop and phone users explicitly trade download cost for map detail; C++ combo-box row.
		ui.horizontal_wrapped(|ui| {
			ui.label("Texture detail");
			egui::ComboBox::from_id_salt("moon-quality").selected_text(format!("{}k", self.target_tier)).show_ui(ui, |ui| {
				for tier in [2,4,8].into_iter().filter(|t| *t <= self.max_tier) { ui.selectable_value(&mut self.target_tier, tier, format!("{tier}k")); }
			});
			help::question(ui, "The Moon always appears in 2k first. Desktop browsers normally improve to 4k; phones keep 2k. Choose 8k only if you want a larger download and have enough memory. Higher detail changes only the globe map, not scientific data. The old Moon remains visible if an upgrade fails.");
		});
		// Distinguish displayed quality from a download still in progress; C++ optional snapshot.
		let displayed = self.renderer.lock().as_ref().map(|r| r.tier);
		help::annotated(
			ui,
			format!(
				"Showing {} | requested {}k",
				displayed.map_or("loading".into(), |t| format!("{t}k")),
				self.target_tier
			),
			"The first number is the currently visible map. The requested detail replaces it only after both images have decoded and uploaded successfully.",
		);
	}
}
```

### 4.25. app/src/layout.rs

```rust
use super::*;
use eframe::egui;

impl MoonApp
{
	pub(super) fn update_sites(&mut self, ui: &mut egui::Ui)
	{
		// Load only the selected site, or the remaining sites when all-rays is explicitly requested; C++ lazy cache initialization.
		for i in 0..self.catalog.len()
		{
			if (i == self.selected || self.all_rays) && self.panoramas[i].is_none()
			{
				self.panoramas[i] = Some(panorama::Panorama::new(
					ui.ctx().clone(),
					filename(&self.catalog[i].name),
				));
			}
			// Check this state or input before acting; C++ conditional branch.
			if let Some(p) = &mut self.panoramas[i]
			{
				p.update();
			}
		}
	}

	pub(super) fn synchronize_scene(&mut self)
	{
		// Keep one UTC epoch across switches and all-site rays; C++ optional epoch restoration.
		if let (Some(t), Some(p)) = (self.time, &mut self.panoramas[self.selected])
		{
			p.set_time(t);
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let margin = self.panoramas[self.selected]
			.as_ref()
			.map_or(0.5, |p| p.margin());
		// Bind this intermediate value for the current operation; C++ local initialization.
		let epoch = self.time.or_else(|| {
			self.panoramas[self.selected]
				.as_ref()
				.and_then(|p| p.time())
		});
		// Bind this intermediate value for the current operation; C++ local initialization.
		let scene = self
			.catalog
			.iter()
			.enumerate()
			.map(|(i, s)| {
				// Bind this intermediate value for the current operation; C++ local initialization.
				let directions = if i == self.selected || self.all_rays
				{
					self.panoramas[i]
						.as_mut()
						.and_then(|p| epoch.and_then(|t| p.directions_at(t, margin)))
				}
				else
				{
					None
				};
				globe::Marker {
					id: i,
					name: s.name.clone(),
					lat_deg: s.lat_deg,
					lon_deg: s.lon_deg,
					directions,
				}
			})
			.collect();
		self.globe.set_scene(scene, self.selected);
	}

	pub(super) fn central_view(&mut self, ui: &mut egui::Ui)
	{
		// Preserve black behind the Moon in both layouts; C++ immediate-mode central container.
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK))
			.show(ui, |ui| {
				// Capture finite viewport dimensions before entering the unbounded scroll axis; C++ copy a size vector.
				let available = ui.available_size();
				// Scroll only if a short screen cannot fit both views; C++ immediate-mode scroll container.
				eframe::egui::ScrollArea::vertical().show(ui, |ui| {
					self.main_views(ui, available);
				});
			});
	}
}

impl MoonApp
{
	fn main_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		// Stack on phones rather than crushing a 360-degree plot; C++ viewport-width branch.
		if available.x < 700.0
		{
			self.phone_views(ui, available);
		}
		// Use equal-width columns on larger windows; C++ else with two child containers.
		else
		{
			self.desktop_views(ui, available);
		}
	}
}

impl MoonApp
{
	fn phone_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		// Reserve room for phone camera and lighting controls; C++ clamp(height*0.32,220,290).
		let size = eframe::egui::vec2(available.x, (available.y * 0.32).clamp(220.0, 290.0));
		// Constrain the globe callback to the top rectangle; C++ child container with explicit size.
		ui.allocate_ui(size, |ui| {
			// Check this state or input before acting; C++ conditional branch.
			if let Some(id) = self.globe.show(ui)
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.selected = id;
			}
		});
		// Draw the scientific view below it; C++ member draw call.
		ui.allocate_ui(
			eframe::egui::vec2(available.x, (available.y - size.y - 8.0).max(460.0)),
			|ui| self.show_panorama(ui),
		);
	}

	fn desktop_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		// Rebuild columns without retaining widget handles; C++ immediate-mode callback.
		ui.allocate_ui(available, |ui| {
			// Divide only the bounded native/desktop viewport, not the unbounded scroll axis; C++ column container.
			ui.columns(2, |columns| {
				// Keep the rotating Moon on black at left; C++ first child draw.
				if let Some(id) = self.globe.show(&mut columns[0])
				{
					// Update this persistent member explicitly; C++ member assignment.
					self.selected = id;
				}
				// Disable columns' cross-axis justification so the plot can reserve tick-label space; C++ child layout override.
				columns[1].with_layout(
					eframe::egui::Layout::top_down(eframe::egui::Align::Min),
					|ui| self.show_panorama(ui),
				);
			})
		});
	}
}
```

### 4.26. app/src/panorama/view.rs

```rust
use super::*;

impl Panorama
{
	// Present a plain-language scene first, with optional numerical detail; C++ immediate-mode member function.
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		// Scrollable child UIs must wrap prose instead of widening the entire panorama; C++ local text-layout policy.
		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
		globe::help::annotated(
			ui,
			egui::RichText::new("Standing on the Moon").heading(),
			"Imagine standing at the selected point. The strip shows every direction around you; gray is terrain, yellow is Sun, blue is Earth. Hills hide the parts below their edge. Height is stretched and labeled. Faint paths cover the menu time window; this is not a landing-safety or thermal model.",
		);
		// Never draw invented tracks while waiting or after an error; C++ optional extraction.
		let Some(data) = &mut self.data
		else
		{
			// Show the real loader state; C++ optional value_or.
			globe::help::annotated(
				ui,
				self.error
					.as_deref()
					.unwrap_or("Loading site and sky data..."),
				"Results are unavailable until the selected track has loaded and passed validation. No substitute directions or statistics are shown for missing or invalid data.",
			);
			// Stop before indexing arrays; C++ early return.
			return;
		};
		// Identify the published observation point; C++ borrowed text.
		globe::help::annotated(
			ui,
			&data.site.name,
			"The selected published observation point. Use Site in the menu to change it; nearby coordinates can have very different horizons.",
		);
		solar_status(ui, data, self.index);
		earth_status(ui, data, self.index);
		// Keep UTC explicit, independent of the computer's time zone; C++ time formatting.
		globe::help::annotated(
			ui,
			utc_label(data.sun.t[self.index]),
			"The current shared UTC date and time. Open Time in the menu to change it. Hourly sampling can miss short events.",
		);
		// Show the same scientific data in either presentation; C++ view-mode branch.
		if self.technical
		{
			technical(ui, data, self.index, self.hours);
		}
		else
		{
			// All disks are angular-size correct before the labeled vertical stretch; C++ renderer call.
			crate::sky::show(
				ui,
				data,
				self.index,
				window(&data.sun.t, self.index, self.hours),
			);
		}
		summary_cards(ui, data, &self.summary);
	}
}

// Retain the original numerical plot independently of the novice scene; C++ free drawing function.
fn technical(ui: &mut egui::Ui, data: &SiteData, index: usize, hours: i64)
{
	// Select by UTC duration, allowing irregular future timestamps; C++ binary-search interval.
	let range = window(&data.sun.t, index, hours);
	// Explain endpoint clipping without crowding a phone with duplicate timestamps; C++ formatted count label.
	ui.small(format!(
		"Window: {} samples (centered, clipped)",
		range.len()
	));
	// Keep a readable graph while leaving space for summary cards below it; C++ fixed logical height.
	let height = 260.0;
	// Fix azimuth bounds while allowing elevation to fit terrain/track extrema; C++ plotting builder.
	Plot::new("panorama")
		// Leave room for the right endpoint's centered tick label inside the panel clip; C++ explicit plot width.
		.width((ui.available_width() - 20.0).max(100.0))
		.height(height)
		.default_x_bounds(0.0, 360.0)
		.include_y(-8.0)
		.include_y(8.0)
		.set_margin_fraction(egui::vec2(0.0, 0.05))
		.allow_zoom(false)
		.allow_drag(false)
		.allow_scroll(false)
		.allow_double_click_reset(false)
		.x_axis_label("Azimuth (deg, north clockwise)")
		.y_axis_label("Elevation (deg)")
		// Force cardinal ticks rather than accepting automatic 100-degree spacing; C++ generate five grid marks.
		.x_grid_spacer(|_| {
			(0..=4)
				.map(|i| GridMark {
					value: f64::from(i) * 90.0,
					step_size: 90.0,
				})
				.collect()
		})
		// Name both north endpoints of the unwrapped plot; C++ formatter lambda.
		.x_axis_formatter(|mark, _| {
			match mark.value as i32
			{
				0 => "0 N",
				90 => "90 E",
				180 => "180 S",
				270 => "270 W",
				_ => "360 N",
			}
			.into()
		})
		.legend(Legend::default())
		.show(ui, |plot| {
			technical_items(plot, data, index, range.clone());
		});
}

fn solar_status(ui: &mut egui::Ui, data: &SiteData, index: usize)
{
	// Delegate the solar area to the scientific core; C++ pure function call.
	let fraction = sun_fraction(
		data.sun.el_deg[index],
		data.horizon_at(data.sun.az_deg[index]),
	);
	// Green indicates any available sunlight, not full disk exposure; C++ conditional text/color pair.
	let (sun_text, sun_color) = if fraction > 0.0
	{
		(
			format!(
				"Sun visible: solar power available ({:.1}% of the disk)",
				fraction * 100.0
			),
			Color32::LIGHT_GREEN,
		)
	}
	else
	{
		(
			"Sun hidden behind terrain: no solar power".into(),
			Color32::LIGHT_RED,
		)
	};
	// Use large wrapping text so the whole status fits a phone; C++ rich-text widget.
	globe::help::annotated(
		ui,
		egui::RichText::new(sun_text)
			.size(18.0)
			.strong()
			.color(sun_color),
		"Sunlight can reach the site when some of the yellow disk clears the terrain. The percentage is the visible area of the disk, not guaranteed electrical output. Panel direction, hardware, dust and eclipses are not modeled.",
	);
}

fn earth_status(ui: &mut egui::Ui, data: &SiteData, index: usize)
{
	// Include the selected radio clearance, not just visible blue pixels; C++ cached bool.
	let earth_visible = data.earth_above[index];
	// Keep the full plain-language status visible without interpreting a graph; C++ conditional string.
	globe::help::annotated(
		ui,
		egui::RichText::new(
			if earth_visible
			{
				"Earth visible: direct communication possible"
			}
			else
			{
				"Earth hidden: no direct communication"
			},
		)
		.size(18.0)
		.strong()
		.color(
			if earth_visible
			{
				Color32::LIGHT_GREEN
			}
			else
			{
				Color32::LIGHT_RED
			},
		),
		"A direct radio signal could reach Earth without terrain blocking it, including the chosen safety margin. This is a geometric opportunity, not guaranteed service; relays, antennas and Earth stations are not modeled.",
	);
	// Qualify a blocked radio link that may still have a partly visible disk; C++ explanatory label.
	ui.small("Radio status includes the safety margin, not just the visible blue disk.");
}

fn summary_cards(ui: &mut egui::Ui, data: &SiteData, summary: &lunar_core::Stats)
{
	// Anchor summary cards to the full period, never the selected path window; C++ subheading.
	ui.strong("Across the whole data period");
	// Show exact dates rather than an unexplained average; C++ formatted label.
	ui.small(format!(
		"{} to {}",
		utc_label(data.sun.t[0]),
		utc_label(*data.sun.t.last().unwrap())
	));
	// Reserve frame padding and the gutter before dividing the available width; C++ bounded card size.
	let card_width = ((ui.available_width() - 40.0) / 2.0).clamp(90.0, 180.0);
	card_grid(ui, summary, card_width);
	// Avoid implying that hourly held-sample statistics are continuous event solutions; C++ disclaimer.
	ui.small("Hourly estimates. Sun time means at least part of the disk is visible. Power and radio service are not guaranteed.");
}

fn card_grid(ui: &mut egui::Ui, summary: &lunar_core::Stats, card_width: f32)
{
	// Two columns wrap well on both phone and desktop panels; C++ grid of framed cards.
	egui::Grid::new("site_summary")
		.num_columns(2)
		.spacing([8.0, 6.0])
		.show(ui, |ui| {
			// Include units and the definition of sunlight in the visible labels; C++ range-for over values.
			for (i, (title, value)) in [
				("Time with any Sun", format!("{:.1}%", summary.pct_sun)),
				("Time with Earth link", format!("{:.1}%", summary.pct_earth)),
				("Time with both", format!("{:.1}%", summary.pct_both)),
				(
					"Longest night",
					format!("{:.2} days", summary.longest_night_h / 24.0),
				),
			]
			.iter()
			.enumerate()
			{
				summary_card(ui, title, value, i, card_width);
				// Close each pair explicitly; C++ grid row break.
				if i % 2 == 1
				{
					ui.end_row();
				}
			}
		});
}

fn summary_card(ui: &mut egui::Ui, title: &str, value: &str, i: usize, card_width: f32)
{
	// A filled frame separates each summary from controls; C++ styled card container.
	egui::Frame::group(ui.style())
						.fill(Color32::from_gray(28))
						.show(ui, |ui| {
							// Stack each number above its caption instead of inheriting the grid's row layout; C++ column container.
							ui.vertical(|ui| {
								// Give both cards equal widths inside the parent column; C++ width constraint.
								ui.set_width(card_width);
								// Render the number prominently; C++ large-text label.
								ui.label(egui::RichText::new(value).size(21.0).strong());
								// Explain the number in ordinary language; C++ small label.
								ui.horizontal_wrapped(|ui| { ui.small(title); globe::help::question(ui, match i { 0 => "Fraction of the entire data period with at least some Sun visible. Partial disks count; this is not energy yield.", 1 => "Fraction of the entire period with direct Earth clearance at your chosen radio margin, not guaranteed radio service.", 2 => "Fraction of the entire period when sunlight and direct Earth clearance are both available.", _ => "Longest continuous sampled interval without any Sun, measured in days. A day here is 24 hours; short events between hourly samples can be missed." }); });
							});
						});
}

fn technical_items(
	plot: &mut egui_plot::PlotUi<'_>,
	data: &SiteData,
	index: usize,
	range: Range<usize>,
)
{
	// Always show the geometric flat reference, even if terrain is supplied; C++ submit a horizontal line.
	plot.hline(HLine::new("Flat 0 deg", 0.0).color(Color32::GRAY));
	// Add terrain only when present; this is profile interpolation, not DEM ingestion; C++ if(optional).
	if let Some(horizon) = &data.horizon
	{
		// Include every original uniform bin, with extra points for coarse profiles; C++ max(size,360).
		let bins = horizon.h_deg.len().max(360);
		// Close the cyclic line at 360 using Horizon::at, including all bin boundaries; C++ transform a counted range.
		let points: Vec<_> = (0..=bins)
			.map(|i| {
				// Convert this uniform bin boundary to degrees; C++ float arithmetic.
				let az = i as f32 * 360.0 / bins as f32;
				// Ask the core for the interpolated elevation, then widen plot coordinates; C++ array<double,2>.
				[f64::from(az), f64::from(horizon.at(az))]
			})
			.collect();
		// Keep the terrain visually distinct from the flat reference and both bodies; C++ submit a colored polyline.
		plot.line(Line::new("Terrain", points).color(Color32::LIGHT_RED));
	}
	// Apply identical window/wrap rules to both tracks; C++ range-for over named borrowed tracks.
	for (name, track, color) in [
		("Sun", &data.sun, Color32::YELLOW),
		("Earth", &data.earth, Color32::LIGHT_BLUE),
	]
	{
		// Never connect segments separated by a north wrap; C++ range-for over polyline pieces.
		for segment in segments(track, range.clone())
		{
			// Same body name groups pieces under a single legend entry; C++ submit each independent line.
			plot.line(Line::new(name, segment).color(color));
		}
		// Mark the current direction even for a one-sample interval; C++ one-point scatter series.
		plot.points(
			Points::new(
				format!("{name} now"),
				vec![[
					f64::from(track.az_deg[index].rem_euclid(360.0)),
					f64::from(track.el_deg[index]),
				]],
			)
			.color(color)
			.radius(4.0),
		);
	}
}
```

## 5. Alphabetical API and method index

GLSL smoothstep and mix are explained in [section 2.3](#23-glsl-apis); the Rust/OpenGL call index follows.

Each entry points to the complete annotated quotation. Sections 1 and 2 define the language and API contracts before those quotations.

- `abs`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `acos`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `active_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `add`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `add_enabled`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `add_triangle`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `advance_upgrade`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `all`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `allocate_exact_size`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `allocate_painter`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `allocate_ui`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `allow_double_click_reset`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `allow_drag`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `allow_scroll`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `allow_zoom`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `and_then`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `angular_disks_and_projection`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `animate_camera`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `announce`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `any`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `app::MoonApp::new`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `Arc::clone`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `Arc::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `as_chunks`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_deref`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_f64`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `as_mut`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_raw`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_ref`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `asin`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `aspect_ratio`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `assert!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `assert_eq!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `assets::load`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `assets::preferences`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `assets::select`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `at`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `atan2`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `attach_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `availability`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `available_height`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `available_size`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `available_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `begin_frame`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `begin_upgrade`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_attrib_location`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_buffer`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_vertex_array`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `blend_func`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bodies`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `BodyDirection`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bottom`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `Box::new`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `browser_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `buffer_data_u8_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `build_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_ray_layout`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `bundled_three_years_are_aligned_hourly_and_have_terrain`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `button`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `bytemuck::cast_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `camera_controls`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `card_grid`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `center`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `centered_site_and_billboard_width`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `central_view`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `chain`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `changed`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `checkbox`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `circle_filled`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `circle_stroke`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `clamp`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `clear`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `clear_depth_f32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `clicked`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `clone`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `collect`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `color`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Color32::from_gray`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Color32::from_rgb`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `colored_label`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `colored_vertex`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `columns`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `compile_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `compile_stages`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `contains`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `cos`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `crate::sky::show`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `create_buffer`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `create_program`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `create_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `create_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `create_vertex_array`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `ctx`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `cull_face`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `data::load`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Default`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `Default::default`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `default_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `default_x_bounds`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `delete_buffer`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `delete_program`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `delete_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `delete_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `delete_vertex_array`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `delta`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `depth_func`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `depth_mask`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `desktop_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `destroy`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `detach_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `direction`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `directions`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `directions_at`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `disable`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `disk`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `display`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `distance_sq`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `document`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `dragged`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `drain`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `draw_arrays`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `draw_elements`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `draw_marker`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `dyn_into`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `earth_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `earthshine_phase_and_scale`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `earthshine_ratio`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `eframe::egui::CentralPanel::default`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::Frame`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::Layout::top_down`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::ScrollArea::vertical`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::vec2`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::run_native`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `eframe::WebRunner::new`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `egui::Button::new`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Color32::from_black_alpha`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Color32::from_rgb`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::ComboBox::from_id_salt`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::FontId::proportional`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Frame::group`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::Grid::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::Mesh::default`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::pos2`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::Rect::from_min_size`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::RichText::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::Sense::click_and_drag`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Sense::hover`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Shape::convex_polygon`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::Shape::line`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::Shape::mesh`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::Slider::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::Stroke::new`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::vec2`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Window::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui_glow::CallbackFn::new`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui_glow::ShaderVersion::get`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `ehttp::fetch`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `ehttp::Request::get`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `empty`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `enable`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `enable_vertex_attrib_array`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `end_row`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `endpoint`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `enumerate`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `env!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `env_logger::init`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `eprintln!`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `exp`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `expand`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `expect`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `extend`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `extend_from_slice`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `f64::from`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `field::illuminated_points`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `fill`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `filter`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `filter_map`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `find`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `fixed_decimals`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `fixture`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `flat_and_interpolated_terrain_visibility`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `flat_map`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `flatten`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `format`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `format!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `frame`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `front_face`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `generate_mipmap`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `get_element_by_id`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `get_error`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_or_insert`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_parameter_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_program_info_log`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_program_link_status`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_shader_compile_status`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_shader_info_log`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `get_uniform_location`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `gl`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `Globe`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `globe::Globe::new`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `globe::help::annotated`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `globe::help::question`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `glow::PixelUnpackData::Slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `guides`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `heading`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `height`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `help::annotated`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `help::question`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `hline`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `HLine::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `horizon_at`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `horizontal`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `horizontal_wrapped`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `hovered`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `hypot`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `image::load_from_memory`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `include_bytes!`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `include_str!`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `include_y`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `initialize`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `inner_width`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `input`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `input_mut`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `interact_pointer_pos`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `into`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `into_iter`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `is_dir`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `is_empty`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `is_err`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `is_finite`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `is_new_shader_interface`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `is_none`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `is_none_or`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `is_positive`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `is_some`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `is_some_and`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `iter`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `join`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `json!`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `label`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `last`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `last_mut`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `left`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `left_top`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `legend`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Legend::default`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `len`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `light_uniforms`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `lighting_controls`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `line`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Line::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `line_segment`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `link_program`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `ln`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `load`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `load_error`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `loading_status`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `local_direction`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `local_sky_frame`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `location`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `lock`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `lunar_core::stats`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `lunar_core::Stats::default`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `lunar_core::sun_fraction`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `main_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `map`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `map_err`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `map_or`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `map_or_else`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `margin`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `marker`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `Marker`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::direction`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::earthshine_ratio`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::local_direction`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::marker`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::perspective`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::rotation`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::sphere`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `max`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `mesh_orientation_and_seam`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `min`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `model`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `moon_fixed`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `Mutex::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `native_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `native_pixels`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `navigator`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `next`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `num_columns`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `ok`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `ok_or`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `on_hover_text`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `open`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `or_else`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `orbit_input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `paint_markers`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint_opaque`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_site_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_surface_rays`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `painter`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `painter_at`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `panic!`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `panorama::Panorama::new`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `parent`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `parse`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `partition_point`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Pending::default`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `pending_label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `perspective`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `phone_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `pixel_store_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `pixels_per_point`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `Plot::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `pointer_mut`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `points`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Points::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `powi`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `project`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `projection`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `projection_depth_and_aspect`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `push`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `queue_gl`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `radio_controls`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `radius`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Rect::from_min_max`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `rect_filled`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `rejects_invalid_inputs`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `rem_euclid`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `remove`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `Renderer::new`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `replace`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `request_repaint`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `RgbImage::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `ribbon`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `right`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `rotation`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `saturating_sub`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `scene_identity_empty_and_invalid`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `scene_lighting`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `scene_rays`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `search`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `segments`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `selectable_value`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `selected_text`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
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
- `serde_json::from_slice`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `serde_json::to_vec`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `set_margin_fraction`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `set_max_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `set_scene`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `set_text_content`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `set_theme`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `set_time`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `set_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `shader_source`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `show`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `show_panorama`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `show_ui`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `show_value`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `sin`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `sin_cos`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `site`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `site_camera`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `site_projection_and_south_tilt`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `SiteData::parse`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `size`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `small`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `solar_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `spacing`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `spacing_mut`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `sphere`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `sqrt`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `start`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `std::array::from_fn`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `std::env::current_exe`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `std::env::var`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `std::env::var_os`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `std::fs::read`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `std::path::Path::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `std::thread::spawn`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `Stroke::new`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `strong`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `style`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `style_mut`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `suffix`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `sum`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `summary_card`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `summary_cards`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `summary_uses_full_duration_and_current_clearance`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `super::panorama::segments`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `surface_ray_controls`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `synchronize_scene`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `take`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `tan`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `technical`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `technical_controls`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `technical_items`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `terrain`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `tex_image_2d`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `tex_image_2d_with_html_image`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `tex_parameter_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `text`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `texture_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `then`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `time`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `time_controls`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `to_degrees`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `to_lowercase`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `to_radians`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `to_rgb8`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `to_string`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `transpose`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `trim`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `truncate`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_1_f32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_1_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_3_f32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `unwrap`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_err`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_or`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_or_default`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_or_else`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `update`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `update_earth_margin`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `update_sites`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `upload_browser_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `upload_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `use_program`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `user_agent`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `utc_label`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `vec!`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `Vec::from`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `Vec::new`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `Vec::with_capacity`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `version_declaration`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `vertex_attrib_pointer_f32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `vertical`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `viewport_in_pixels`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `vscroll`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `wasm_bindgen_futures::spawn_local`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `web_sys::console::log_1`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `web_sys::UrlSearchParams::new_with_str`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `web_sys::window`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `width`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `window`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `window_label`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `windows`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `windows_are_utc_intervals_not_sample_counts`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `with_clip_rect`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `with_layout`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `wraps_do_not_draw_chords`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `x_axis_formatter`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `x_axis_label`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `x_grid_spacer`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `y_axis_label`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `zip`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `zoom_delta`: [annotated globe/src/lib.rs](#44-globesrclibrs)
