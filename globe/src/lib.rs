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
		})
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
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if gl
				// Inspect advertised optional GL features before using anisotropy; C++ enumerates extension strings.
				.supported_extensions()
				// Borrow elements without moving them; C++ cbegin/cend or a const range.
				.iter()
				// Return true at the first matching heuristic/extension; C++ ranges::any_of short-circuits.
				.any(|name| name.ends_with("texture_filter_anisotropic"))
			{
				// Read the maximum supported anisotropy; C++ glGetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT).
				let max = gl.get_parameter_f32(0x84FF);
				// Cap anisotropic sampling at an anisotropy ratio of eight; C++ glTexParameterf.
				gl.tex_parameter_f32(glow::TEXTURE_2D, 0x84FE, max.min(8.0));
			}
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
			// Send the rotation angle in radians; C++ glUniform1f.
			gl.uniform_1_f32(gl.get_uniform_location(program, "u_angle").as_ref(), angle);
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
