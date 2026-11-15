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
			polar: None,
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
