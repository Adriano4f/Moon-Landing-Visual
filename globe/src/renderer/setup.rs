use super::{Program, Renderer, assets, glow};
use glow::HasContext as _;

impl Renderer
{
	pub(crate) fn new(
		gl: &glow::Context,
		[albedo, normals]: [assets::TextureImage; 2],
	) -> Result<Self, String>
	{
		// Inspect GLSL capabilities rather than assuming desktop syntax.
		let version = egui_glow::ShaderVersion::get(gl);
		if !version.is_new_shader_interface()
		{
			return Err("The Moon requires modern OpenGL or WebGL2".into());
		}
		let mut renderer = Self::empty(albedo.width() / 1024);
		let initialized = renderer.initialize(gl, version, albedo, normals);
		if let Err(error) = initialized
		{
			renderer.destroy(gl);
			return Err(error);
		}
		#[cfg(not(target_arch = "wasm32"))]
		eprintln!("Moon textures ready");
		#[cfg(target_arch = "wasm32")]
		web_sys::console::log_1(&"Moon textures ready".into());
		Ok(renderer)
	}

	fn empty(tier: u32) -> Self
	{
		Self {
			program: None,
			fast_program: None,
			shaders: Vec::new(),
			vao: None,
			ray_vao: None,
			ray_buffer: None,
			buffers: Vec::new(),
			// Own the albedo and normal textures in sampler order.
			textures: Vec::new(),
			tier,
			visible_announced: false,
			pending: None,
			polar: None,
			deep: None,
			detail: None,
			heights: Vec::new(),
			height_tier: 0,
			surface_tier: None,
			live_normal: None,
			surface_upload: None,
			deep_ready: false,
			live_shadows: true,
			mask: None,
			mask_failed: false,
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
		(|| -> Result<(), String> {
			// SAFETY: eframe calls the constructor with its GL context current. Every handle is
			// registered immediately so the error branch can clean partial construction.
			unsafe {
				self.build_program(gl, version, false)?;
				self.build_program(gl, version, true)?;
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
			self.deep = Some(super::deep::Atlas::new(gl, 1)?);
			Ok(())
		})()
	}

	unsafe fn build_program(
		&mut self,
		gl: &glow::Context,
		version: egui_glow::ShaderVersion,
		fast: bool,
	) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current.
		unsafe {
			let program = gl.create_program()?;
			let state = std::sync::Arc::new(Program::new(program));
			if fast
			{
				self.fast_program = Some(state);
			}
			else
			{
				self.program = Some(state);
			}
			self.compile_stages(gl, program, version, fast)?;
			gl.bind_attrib_location(program, 0, "a_position");
			gl.bind_attrib_location(program, 1, "a_uv");
			gl.bind_attrib_location(program, 2, "a_coverage");
			gl.link_program(program);
			if !gl.get_program_link_status(program)
			{
				return Err(gl.get_program_info_log(program));
			}
			for shader in self.shaders.drain(..)
			{
				gl.detach_shader(program, shader);
				gl.delete_shader(shader);
			}
			let state = if fast
			{
				&mut self.fast_program
			}
			else
			{
				&mut self.program
			};
			let state = std::sync::Arc::get_mut(state.as_mut().unwrap()).unwrap();
			for index in 0..gl.get_active_uniforms(program)
			{
				if let Some(info) = gl.get_active_uniform(program, index)
					&& let Some(location) = gl.get_uniform_location(program, &info.name)
				{
					state.uniforms.insert(info.name, location);
				}
			}
			gl.use_program(Some(program));
			for (unit, name) in [
				"u_albedo",
				"u_normal",
				"u_polar_color",
				"u_polar_normal",
				"u_deep_color",
				"u_deep_pages",
				"u_height",
				"u_polar_height",
				"u_shadow_mask",
				"u_detail",
				"u_detail_height",
			]
			.iter()
			.enumerate()
			{
				gl.uniform_1_i32(state.uniform(name), unit as i32);
			}
			gl.use_program(None);
		}
		Ok(())
	}

	unsafe fn compile_stages(
		&mut self,
		gl: &glow::Context,
		program: glow::Program,
		version: egui_glow::ShaderVersion,
		fast: bool,
	) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current.
		unsafe {
			for (kind, source) in [
				(glow::VERTEX_SHADER, include_str!("../moon.vert")),
				(glow::FRAGMENT_SHADER, include_str!("../moon.frag")),
			]
			{
				let shader = gl.create_shader(kind)?;
				self.shaders.push(shader);
				gl.shader_source(
					shader,
					&format!(
						"{}\n#define FAST_MODE {}\n{source}", version.version_declaration(),
						u8::from(fast)),
				);
				gl.compile_shader(shader);
				if !gl.get_shader_compile_status(shader)
				{
					return Err(gl.get_shader_info_log(shader));
				}
				gl.attach_shader(program, shader);
			}
		}
		Ok(())
	}

	unsafe fn build_sphere(&mut self, gl: &glow::Context) -> Result<(), String>
	{
		// SAFETY: the analytic globe uses a bounded quad generated by gl_VertexID.
		unsafe {
			self.vao = Some(gl.create_vertex_array()?);
		}
		Ok(())
	}

	unsafe fn build_ray_layout(&mut self, gl: &glow::Context) -> Result<(), String>
	{
		// SAFETY: the caller keeps this renderer's GL context current.
		unsafe {
			// Allocate a second VAO without disturbing the Moon's indexed mesh.
			self.ray_vao = Some(gl.create_vertex_array()?);
			gl.bind_vertex_array(self.ray_vao);
			let ray_buffer = gl.create_buffer()?;
			self.buffers.push(ray_buffer);
			self.ray_buffer = Some(ray_buffer);
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(ray_buffer));
			// Enable only position; unused UV defaults to zero for the constant-color branch.
			gl.enable_vertex_attrib_array(0);
			gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 28, 0);
			gl.enable_vertex_attrib_array(1);
			gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 28, 12);
			gl.enable_vertex_attrib_array(2);
			gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 28, 20);
			gl.bind_vertex_array(None);
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn live_lighting_requires_a_committed_matching_surface()
	{
		let mut renderer = Renderer::empty(0);
		assert!(!renderer.live_ready());
		renderer.surface_tier = Some(2);
		assert!(!renderer.live_ready());
		renderer.tier = 2;
		assert!(renderer.live_ready());
		renderer.tier = 4;
		assert!(!renderer.live_ready());
		renderer.surface_tier = Some(4);
		assert!(renderer.live_ready());
		renderer.live_shadows = false;
		assert!(!renderer.live_ready());
	}
}
