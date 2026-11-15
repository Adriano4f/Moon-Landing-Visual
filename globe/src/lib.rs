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
