mod deep;
mod polar;
mod setup;
mod textures;

use super::glow;
use super::{Lighting, Ray, assets, field, math};
use glow::HasContext as _;

pub(super) struct Renderer
{
	program: Option<glow::Program>,
	shaders: Vec<glow::Shader>,
	vao: Option<glow::VertexArray>,
	/// Own a separate position-only layout for the two direction lines.
	ray_vao: Option<glow::VertexArray>,
	ray_buffer: Option<glow::Buffer>,
	buffers: Vec<glow::Buffer>,
	/// Own the albedo and normal textures in sampler order.
	textures: Vec<glow::Texture>,
	pub(super) tier: u32,
	/// Signal only after the renderer has drawn with real sky directions.
	pub(super) visible_announced: bool,
	/// Retain decoded upgrades between bounded GPU-upload frames.
	pending: Option<(u32, std::vec::IntoIter<assets::TextureImage>)>,
	pub(crate) polar: Option<polar::Atlas>,
	deep: Option<deep::Atlas>,
}

impl Renderer
{
	pub(super) fn begin_upgrade(&mut self, gl: &glow::Context, images: [assets::TextureImage; 2])
	{
		// Discard only abandoned replacement textures, never the visible pair.
		for texture in self.textures.drain(2..)
		{
			unsafe {
				gl.delete_texture(texture);
			}
		}
		self.pending = Some((images[0].width() / 1024, Vec::from(images).into_iter()));
	}

	pub(super) fn advance_upgrade(&mut self, gl: &glow::Context) -> Result<bool, String>
	{
		// Temporarily own the in-progress upload, restoring it only after success.
		let Some((tier, mut images)) = self.pending.take()
		else
		{
			return Ok(false);
		};
		// Process only one image in this frame.
		if let Some(image) = images.next()
		{
			// The first replacement is albedo, the second is directional data.
			let srgb = self.textures.len() == 2;
			#[cfg(not(target_arch = "wasm32"))]
			let result =
				self.upload_texture(gl, image.width(), image.height(), image.as_raw(), srgb);
			#[cfg(target_arch = "wasm32")]
			let result = self.upload_browser_texture(gl, &image, srgb);
			if let Err(error) = result
			{
				// SAFETY: this callback owns the current GL context.
				for texture in self.textures.drain(2..)
				{
					unsafe {
						gl.delete_texture(texture);
					}
				}
				return Err(error);
			}
			self.pending = Some((tier, images));
			return Ok(true);
		}
		// SAFETY: old textures are no longer sampled after this callback-local swap.
		for texture in self.textures.drain(..2)
		{
			unsafe {
				gl.delete_texture(texture);
			}
		}
		self.tier = tier;
		self.visible_announced = false;
		Ok(false)
	}

	/// Draw one opaque sphere inside egui's already configured viewport.
	pub(super) fn paint(
		&self,
		gl: &glow::Context,
		view: math::View,
		rays: &[([f32; 3], [Ray; 2])],
		lighting: Option<Lighting>,
		surface_rays: bool,
		polar_ready: bool,
	)
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
			Self::begin_frame(gl, program);
			Self::light_uniforms(gl, program, lighting);
			self.bind_polar(gl, program, polar_ready);
			self.bind_deep(gl, program);
			self.paint_opaque(gl, program, view);
			// Smooth alpha coverage needs blending on native and WebGL.
			gl.enable(glow::BLEND);
			gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
			gl.depth_mask(false);
			if surface_rays
			{
				self.paint_surface_rays(gl, program, view, lighting);
			}
			self.paint_site_rays(gl, program, view, rays);
			gl.depth_mask(true);
			gl.disable(glow::BLEND);
			gl.bind_vertex_array(None);
			for unit in 0..6
			{
				gl.active_texture(glow::TEXTURE0 + unit);
				gl.bind_texture(glow::TEXTURE_2D, None);
				if unit == 4
				{
					gl.bind_texture(glow::TEXTURE_2D_ARRAY, None);
				}
			}
			gl.active_texture(glow::TEXTURE0);
			gl.use_program(None);
			gl.disable(glow::DEPTH_TEST);
			gl.disable(glow::CULL_FACE);
		}
	}

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
			ribbon.extend(view.arrowhead(origin,point,0.0));
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

	/// Release registered GPU objects using the current context.
	pub(super) fn destroy(&mut self, gl: &glow::Context)
	{
		if let Some(deep) = self.deep.take()
		{
			deep.destroy(gl);
		}
		if let Some(polar) = self.polar.take()
		{
			polar.destroy(gl);
		}
		// SAFETY: called with the owning context current, either on_exit or failed construction.
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
			if let Some(vao) = self.ray_vao.take()
			{
				gl.delete_vertex_array(vao);
			}
			self.ray_buffer = None;
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

impl Renderer
{
	unsafe fn light_uniforms(gl: &glow::Context, program: glow::Program, lighting: Option<Lighting>)
	{
		// SAFETY: the caller keeps this renderer's GL context current.
		unsafe {
			let light = lighting.unwrap_or(Lighting {
				sun: [0.0, 0.0, 1.0],
				earth: [0.0, 0.0, 1.0],
				ratio: 0.0,
				boosted: false,
			});
			for (name, direction) in [("u_sun", light.sun), ("u_earth", light.earth)]
			{
				gl.uniform_3_f32(
					gl.get_uniform_location(program, name).as_ref(),
					direction[0],
					direction[1],
					direction[2],
				);
			}
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_sun_scale").as_ref(),
				if lighting.is_some() { 1.0 } else { 0.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_earthshine").as_ref(),
				light.ratio * if light.boosted { 1000.0 } else { 1.0 },
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
		// SAFETY: the caller keeps this renderer's GL context current.
		unsafe {
			// These are direction diagrams, not physical beams: retain blocked rays on the visible surface.
			gl.disable(glow::DEPTH_TEST);
			// Draw real object-space direction lines against the sphere's existing depth buffer.
			for &(origin, bodies) in rays
			{
				// Never expose diagrams from the back of the Moon.
				if view.project(origin, true).is_none()
				{
					continue;
				}
				gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 1);
				// Select the independent, position-only line layout.
				gl.bind_vertex_array(self.ray_vao);
				// Upload two endpoints per ray without modifying the sphere buffers.
				gl.bind_buffer(glow::ARRAY_BUFFER, self.ray_buffer);
				for (body, color) in bodies
					.into_iter()
					.zip([[1.0, 0.9, 0.05], [0.15, 0.65, 1.0]])
				{
					// Direction rays are 0.6 lunar radii, not body distances.
					let end: [f32; 3] =
						std::array::from_fn(|i| origin[i] + 0.6 / view.zoom * body.direction[i]);
					let mut vertices = view.ribbon(origin, end);
					vertices.extend(view.arrowhead(end,origin,11.0));
					// Stream the ribbon and endpoint triangles without persistent mapping.
					gl.buffer_data_u8_slice(
						glow::ARRAY_BUFFER,
						bytemuck::cast_slice(&vertices),
						glow::STREAM_DRAW,
					);
					// Dim failed terrain/radio predicates on the visible-side direction diagram.
					let strength = if body.visible { 1.0 } else { 0.35 };
					// Send an unlit display color rather than illuminating rays with the globe's illustration light.
					gl.uniform_3_f32(
						gl.get_uniform_location(program, "u_ray_color").as_ref(),
						color[0] * strength,
						color[1] * strength,
						color[2] * strength,
					);
					// Draw both ribbon faces and the endpoint fan as portable triangles.
					gl.disable(glow::CULL_FACE);
					gl.draw_arrays(glow::TRIANGLES, 0, (vertices.len() / 7) as i32);
				}
				// Do not leak a dynamic buffer binding into egui.
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
		}
	}
}

impl Renderer
{
	unsafe fn begin_frame(gl: &glow::Context, program: glow::Program)
	{
		// SAFETY: the calling paint callback owns the current context.
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
		}
	}
}

impl Renderer
{
	unsafe fn paint_opaque(&self, gl: &glow::Context, program: glow::Program, view: math::View)
	{
		// SAFETY: the calling paint callback owns the current context.
		unsafe {
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_camera_distance")
					.as_ref(),
				view.camera_distance(),
			);
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 0);
			// Send a column-major projection without transposing.
			gl.uniform_matrix_4_f32_slice(
				gl.get_uniform_location(program, "u_projection").as_ref(),
				false,
				&view.projection(),
			);
			// Share one column-major rigid model matrix with geometry, normals, and CPU marker projection.
			gl.uniform_matrix_3_f32_slice(
				gl.get_uniform_location(program, "u_model").as_ref(),
				false,
				&view.model(),
			);
			for (unit, name) in ["u_albedo", "u_normal"].iter().enumerate()
			{
				gl.active_texture(glow::TEXTURE0 + unit as u32);
				gl.bind_texture(glow::TEXTURE_2D, Some(self.textures[unit]));
				gl.uniform_1_i32(gl.get_uniform_location(program, name).as_ref(), unit as i32);
			}
			gl.bind_vertex_array(self.vao);
			gl.draw_arrays(glow::TRIANGLES, 0, 3);
		}
	}
}
