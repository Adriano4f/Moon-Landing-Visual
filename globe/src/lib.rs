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
