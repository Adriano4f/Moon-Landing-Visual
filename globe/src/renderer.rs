mod polar;
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
	pub(crate) polar: Option<polar::Atlas>,
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
		polar_ready: bool,
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
			self.bind_polar(gl, program, polar_ready);
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
			for unit in 0..4
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

	// Release registered GPU objects using the current context; C++ explicit teardown avoids a late invalid-context destructor.
	pub(super) fn destroy(&mut self, gl: &glow::Context)
	{
		if let Some(polar) = self.polar.take()
		{
			polar.destroy(gl);
		}
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
					let mut vertices = view.ribbon(origin, end);
					vertices.extend(view.arrowhead(end,origin,11.0));
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
			// Share one camera position across vertex projection, shading and CPU picking; C++ glUniform1f.
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_camera_distance")
					.as_ref(),
				math::CAMERA_DISTANCE,
			);
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
