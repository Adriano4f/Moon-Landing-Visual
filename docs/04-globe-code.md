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

globe/Cargo.toml makes this an independent library depending only on egui, egui_glow, bytemuck, and JPEG/PNG image decoding. WASM-only dependencies add ehttp and browser bindings. No lunar_core dependency is introduced. app depends on globe by path. index.html copies the six static assets for Trunk, leaves the title Karaya Explore unchanged, and sets a black page background.

assets.rs defines Loading with two optional results and a terminal diagnostic. select parses a requested label, chooses a default from mobile status, then searches descending supported widths under both limits. preferences obtains runtime platform hints. load starts two disk workers or two HTTP completions, decodes each image, and writes its fixed slot under the mutex. Loading::take first returns any existing error; only when both slots exist does it move both out. Otherwise it returns Ok(None). tiers_and_loading covers default/forced/clamped quality, invalid quality/GPU size, incomplete data, a failed slot, successful transfer, and the emptied state.

math.rs defines direction, the trigonometric convention. sphere asserts dimensions, emits interleaved vertices row by row, makes exact poles/seam copies, and appends six indices per cell. perspective constructs the projective matrix derived in the rendering guide. rotation composes geographic-north animation with a 30-degree south presentation tilt; marker projects and sphere-occludes a unit site direction. The [panorama guide](05-panorama.md) derives the shared CPU/GPU transformation. Tests check mesh dimensions, index bounds, exact seams, orientation, winding, depth endpoints, portrait scaling, and the example marker through a full turn.

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
web-sys = { version = "0.3", features = ["Window", "Navigator", "Location", "UrlSearchParams", "console"] }
```

### 4.2. globe/src/assets.rs

```rust
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui::mutex::Mutex;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use image::RgbImage;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::sync::Arc;

// Keep asynchronous CPU results separate from current-context GPU state; C++ defines a similar aggregate.
pub(crate) struct Loading
{
	// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
	pub images: [Option<Result<RgbImage, String>>; 2],
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
		(mobile, forced)
	}
}

// Start exactly two asynchronous image reads for one tier; C++ shared_ptr keeps completion state alive.
pub(crate) fn load(tier: u32) -> Arc<Mutex<Loading>>
{
	// Share two pending image results with asynchronous producers; C++ shared_ptr protects a mutex-backed aggregate.
	let state = Arc::new(Mutex::new(Loading {
		// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
		images: [None, None],
		// An optional terminal loading/upload diagnostic; C++ optional<string>.
		error: None,
	}));
	// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
	for (index, filename) in [format!("albedo_{tier}k.jpg"), format!("normal_{tier}k.png")]
		// Consume this array into an iterator; C++ iterates its owned local elements.
		.into_iter()
		// Attach each element's zero-based index; C++ an indexed range or explicit counter.
		.enumerate()
	{
		// Share two pending image results with asynchronous producers; C++ shared_ptr protects a mutex-backed aggregate.
		let state = Arc::clone(&state);
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
		};
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Move disk I/O and image decoding to a native worker thread; C++ launches a thread with a value capture.
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
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Start a browser HTTP fetch without blocking rendering; C++ native HTTP libraries have no direct browser DOM equivalent.
		ehttp::fetch(
			// Create a GET request for only this selected tier's relative URL; C++ constructs the corresponding HTTP request.
			ehttp::Request::get(format!("assets/globe/{filename}")),
			// Own this fetch completion so it outlives the current frame; C++ captures required state by value.
			move |response| {
				// Complete this asset exactly once with bytes or a diagnostic; C++ invokes the captured completion callable.
				done(response.and_then(|r| {
					// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
					if r.ok
					{
						// Wrap a successful result without throwing; C++ expected stores its value.
						Ok(r.bytes)
					}
					// Handle the complementary case; C++ uses the same else branch.
					else
					{
						// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
						Err(format!("HTTP {} loading {filename}", r.status))
					}
				}));
			},
		);
	}
	state
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Loading
{
	// Move both completed images out only when neither is missing or erroneous; C++ expected<optional<array<Image,2>>,string>.
	pub fn take(&mut self) -> Result<Option<[RgbImage; 2]>, String>
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
```

### 4.3. globe/src/math.rs

```rust
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// Convert latitude/longitude in radians to a unit object-space direction; C++ returns array<float,3>.
pub(crate) fn direction(lat: f32, lon: f32) -> [f32; 3]
{
	// Return x=cos(lat)sin(lon), y=sin(lat), z=cos(lat)cos(lon); C++ returns an aggregate array.
	[lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
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

### 4.4. globe/src/lib.rs

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
		})
	}

	// Accept only finite geographic coordinates; absent/invalid input clears the marker; C++ optional setter.
	pub fn set_site(&mut self, site: Option<(f64, f64)>)
	{
		// Convert degrees to the renderer's object-space convention, independent of scientific data types; C++ optional transform.
		self.site = site
			.filter(|&(lat, lon)| {
				lat.is_finite() && lon.is_finite() && (-90.0..=90.0).contains(&lat)
			})
			.map(|(lat, lon)| {
				math::direction(
					lat.to_radians() as f32,
					lon.rem_euclid(360.0).to_radians() as f32,
				)
			});
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
	fn paint(&self, gl: &glow::Context, aspect: f32, angle: f32)
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

### 4.5. globe/src/moon.vert

```glsl
// Request high floating-point precision in WebGL; desktop drivers accept this GLSL precision declaration.
precision highp float;
// Read one unit object-space sphere vertex from attribute location zero.
in vec3 a_position;
// Read the mesh UV from location one; fragment shading reconstructs UV from direction for seam-safe derivatives.
in vec2 a_uv;
// Receive the column-major view-to-clip perspective matrix from Rust.
uniform mat4 u_projection;
// Receive Rx(-30 degrees)*Ry(animation), shared with normals and the CPU site marker; C++ glUniformMatrix3fv.
uniform mat3 u_model;
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

### 4.7. app/src/lib.rs

```rust
// Compile the validated data loader; C++ includes a project module's declarations.
mod data;
// Compile the immediate-mode panorama view; C++ includes a project module's declarations.
mod panorama;
// Compile the angular sky-strip painter; C++ project drawing module.
mod sky;

// Own persistent renderer and scientific-view state; C++ application class members.
pub struct MoonApp
{
	// Keep presentation independent of the scientific core; C++ renderer value member.
	globe: globe::Globe,
	// Own data, selected sample and plot window; C++ view-model value member.
	panorama: panorama::Panorama,
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
		// Move independently owned models into the app; C++ aggregate initialization.
		Ok(Self {
			// Propagate renderer initialization errors; C++ expected error forwarding.
			globe: globe::Globe::new(gl)?,
			// Start one asynchronous data request, not one per frame; C++ copy the shared UI context handle.
			panorama: panorama::Panorama::new(cc.egui_ctx.clone()),
		})
	}
}

// Implement the framework's application trait; C++ override a virtual application interface.
impl eframe::App for MoonApp
{
	// Rebuild responsive UI each frame; C++ virtual draw callback.
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		// Take completed data before deriving the globe marker; C++ poll an optional completion.
		self.panorama.update();
		// Share only geographic coordinates with the renderer; C++ optional<pair<double,double>> setter.
		self.globe.set_site(self.panorama.site());
		// Preserve black behind the Moon in both layouts; C++ immediate-mode central container.
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK))
			.show(ui, |ui| {
				// Capture finite viewport dimensions before entering the unbounded scroll axis; C++ copy a size vector.
				let available = ui.available_size();
				// Scroll only if a short screen cannot fit both views; C++ immediate-mode scroll container.
				eframe::egui::ScrollArea::vertical().show(ui, |ui| {
					// Stack on phones rather than crushing a 360-degree plot; C++ viewport-width branch.
					if available.x < 700.0
					{
						// Reserve a compact globe so phone controls and both axes remain visible; C++ clamp(height*0.24,140,220).
						let size = eframe::egui::vec2(
							available.x,
							(available.y * 0.24).clamp(140.0, 220.0),
						);
						// Constrain the globe callback to the top rectangle; C++ child container with explicit size.
						ui.allocate_ui(size, |ui| self.globe.show(ui));
						// Draw the scientific view below it; C++ member draw call.
						ui.allocate_ui(
							eframe::egui::vec2(
								available.x,
								(available.y - size.y - 8.0).max(460.0),
							),
							|ui| self.panorama.show(ui),
						);
					}
					// Use equal-width columns on larger windows; C++ else with two child containers.
					else
					{
						// Rebuild columns without retaining widget handles; C++ immediate-mode callback.
						ui.allocate_ui(available, |ui| {
							// Divide only the bounded native/desktop viewport, not the unbounded scroll axis; C++ column container.
							ui.columns(2, |columns| {
								// Keep the rotating Moon on black at left; C++ first child draw.
								self.globe.show(&mut columns[0]);
								// Disable columns' cross-axis justification so the plot can reserve tick-label space; C++ child layout override.
								columns[1].with_layout(
									eframe::egui::Layout::top_down(eframe::egui::Align::Min),
									|ui| self.panorama.show(ui),
								);
							})
						});
					}
				});
			});
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
	<title>Karaya Explore</title>
	<link data-trunk rel="rust">
	<link data-trunk rel="copy-dir" href="assets">
	<link data-trunk rel="copy-file" href="../globe/assets/albedo_2k.jpg" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/normal_2k.png" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/albedo_4k.jpg" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/normal_4k.png" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/albedo_8k.jpg" data-target-path="assets/globe">
	<link data-trunk rel="copy-file" href="../globe/assets/normal_8k.png" data-target-path="assets/globe">
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

## 5. Alphabetical API and method index

GLSL smoothstep and mix are explained in [section 2.3](#23-glsl-apis); the Rust/OpenGL call index follows.

Each entry points to the complete annotated quotation. Sections 1 and 2 define the language and API contracts before those quotations.

- `abs`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `acos`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `active_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `add`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `add_triangle`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
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
- `any`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `app::MoonApp::new`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `Arc::clone`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `Arc::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `as_chunks`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_deref`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_f64`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `as_raw`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `as_ref`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `aspect_ratio`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `assert!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `assert_eq!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `assets::load`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `assets::preferences`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `assets::select`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `at`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `attach_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `availability`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `available_height`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `available_size`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `available_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `bind_attrib_location`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_buffer`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bind_vertex_array`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bottom`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `Box::new`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `buffer_data_u8_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `bundled_three_years_are_aligned_hourly_and_have_terrain`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `button`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `bytemuck::cast_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `center`: [annotated globe/src/lib.rs](#44-globesrclibrs)
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
- `depth_func`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `depth_mask`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `destroy`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `detach_shader`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `disable`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `disk`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `display`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `document`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `drain`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `draw_elements`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `dyn_into`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `eframe::egui::CentralPanel::default`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::Frame`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::Layout::top_down`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::ScrollArea::vertical`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::egui::vec2`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `eframe::run_native`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `eframe::WebRunner::new`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `egui::ComboBox::from_id_salt`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::FontId::proportional`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `egui::Frame::group`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::Grid::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `egui::Mesh::default`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::pos2`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `egui::RichText::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
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
- `enable`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `enable_vertex_attrib_array`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `end_row`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `enumerate`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `env!`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `env_logger::init`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `eprintln!`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `expect`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `extend_from_slice`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `f64::from`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `fill`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `filter`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
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
- `globe::Globe::new`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `glow::PixelUnpackData::Slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `heading`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `height`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `hline`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `HLine::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `horizon_at`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `horizontal`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `horizontal_wrapped`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `image::load_from_memory`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `include_bytes!`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `include_str!`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `include_y`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `inner_width`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `input`: [annotated globe/src/lib.rs](#44-globesrclibrs)
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
- `line`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Line::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `line_segment`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `link_program`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `load`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `location`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `lock`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `lunar_core::stats`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `lunar_core::Stats::default`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `lunar_core::sun_fraction`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `map`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `map_err`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `map_or`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `map_or_else`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `math::direction`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::marker`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::perspective`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::rotation`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `math::sphere`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `max`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `min`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `Mutex::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `navigator`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `num_columns`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `ok`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `ok_or`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `on_hover_text`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `open`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `paint`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `painter`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `panic!`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `panorama::Panorama::new`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `parent`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `parse`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `partition_point`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `pixel_store_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `Plot::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `pointer_mut`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `points`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Points::new`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `project`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `push`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `radius`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `Rect::from_min_max`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `rect_filled`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `rejects_invalid_inputs`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `rem_euclid`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `remove`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `Renderer::new`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `request_repaint`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `RgbImage::new`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `right`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `saturating_sub`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `search`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `segments`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `selectable_value`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `selected_text`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `serde_json::from_slice`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `serde_json::to_vec`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `set_margin_fraction`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `set_max_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `set_site`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `set_text_content`: [annotated app/src/main.rs](#48-appsrcmainrs)
- `set_theme`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `set_width`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `shader_source`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `show`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `show_ui`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `show_value`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `sin`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `sin_cos`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `site`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `SiteData::parse`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `size`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `small`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `spacing`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `spacing_mut`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
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
- `summary_uses_full_duration_and_current_clearance`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `super::panorama::segments`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `take`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `tan`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `technical`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `tex_image_2d`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `tex_parameter_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `text`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `then`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `to_degrees`: [annotated app/src/sky.rs](07-ui.md#81-appsrcskyrs)
- `to_lowercase`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `to_radians`: [annotated globe/src/math.rs](#43-globesrcmathrs)
- `to_rgb8`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `to_string`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `transpose`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `trim`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `uniform_1_i32`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `unwrap`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_err`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_or`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_or_default`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `unwrap_or_else`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `update`: [annotated app/src/lib.rs](#47-appsrclibrs)
- `update_earth_margin`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `upload_texture`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `use_program`: [annotated globe/src/lib.rs](#44-globesrclibrs)
- `user_agent`: [annotated globe/src/assets.rs](#42-globesrcassetsrs)
- `utc_label`: [annotated app/src/panorama.rs](05-panorama.md#92-appsrcpanoramars)
- `vec!`: [annotated app/src/data.rs](05-panorama.md#91-appsrcdatars)
- `Vec::new`: [annotated globe/src/math.rs](#43-globesrcmathrs)
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
