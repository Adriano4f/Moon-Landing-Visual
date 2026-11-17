use super::{Renderer, assets, glow};
use egui::mutex::Mutex;
use glow::HasContext as _;

pub(super) struct Mask
{
	texture: glow::Texture,
	framebuffer: glow::Framebuffer,
	size: [i32; 2],
	key: Vec<u32>,
}

impl Mask
{
	unsafe fn new(gl: &glow::Context, size: [i32; 2]) -> Result<Self, String>
	{
		unsafe {
			let texture = gl.create_texture()?;
			let framebuffer = match gl.create_framebuffer()
			{
				Ok(f) => f,
				Err(e) =>
				{
					gl.delete_texture(texture);
					return Err(e);
				}
			};
			gl.active_texture(glow::TEXTURE8);
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA8 as i32,
				size[0],
				size[1],
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(None),
			);
			for p in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER]
			{
				gl.tex_parameter_i32(glow::TEXTURE_2D, p, glow::LINEAR as i32);
			}
			for p in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T]
			{
				gl.tex_parameter_i32(glow::TEXTURE_2D, p, glow::CLAMP_TO_EDGE as i32);
			}
			gl.bind_texture(glow::TEXTURE_2D, None);
			gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
			gl.framebuffer_texture_2d(
				glow::FRAMEBUFFER,
				glow::COLOR_ATTACHMENT0,
				glow::TEXTURE_2D,
				Some(texture),
				0,
			);
			let mask = Self {
				texture,
				framebuffer,
				size,
				key: Vec::new(),
			};
			if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE
			{
				mask.destroy(gl);
				return Err(
					"Shadow framebuffer is incomplete; using full-resolution tracing".into(),
				);
			}
			Ok(mask)
		}
	}

	pub(super) unsafe fn destroy(self, gl: &glow::Context)
	{
		unsafe {
			gl.delete_framebuffer(self.framebuffer);
			gl.delete_texture(self.texture);
		}
	}
}

impl Renderer
{
	pub(super) unsafe fn shadow_mask(
		&mut self,
		gl: &glow::Context,
		program: glow::Program,
		view: crate::math::View,
		sun: [f32; 3],
		framebuffer: Option<glow::Framebuffer>,
	)
	{
		unsafe {
			gl.uniform_1_i32(
				gl.get_uniform_location(program, "u_shadow_pass").as_ref(),
				0,
			);
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_mask_ready").as_ref(), 0);
			gl.uniform_1_i32(
				gl.get_uniform_location(program, "u_shadow_mask").as_ref(),
				8,
			);
			if !self.live_shadows || self.heights.len() != 2 || self.mask_failed
			{
				return;
			}
			let mut viewport = [0; 4];
			gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
			let limit = if self.height_tier <= 2 { 256 } else { 512 };
			let size = [
				(viewport[2] / 2).clamp(1, limit),
				(viewport[3] / 2).clamp(1, limit),
			];
			if self.mask.as_ref().is_none_or(|m| m.size != size)
			{
				match Mask::new(gl, size)
				{
					Ok(mask) =>
					{
						if let Some(old) = self.mask.replace(mask)
						{
							old.destroy(gl);
						}
					}
					Err(error) =>
					{
						self.mask_failed = true;
						#[cfg(not(target_arch = "wasm32"))]
						eprintln!("{error}");
						#[cfg(target_arch = "wasm32")]
						web_sys::console::warn_1(&error.into());
						gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer);
						return;
					}
				}
				gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer);
			}
			let mut key: Vec<u32> = view
				.model()
				.into_iter()
				.chain(view.projection())
				.map(f32::to_bits)
				.collect();
			key.extend(sun.into_iter().map(f32::to_bits));
			key.extend([
				view.camera_distance().to_bits(),
				self.height_tier,
				self.polar.as_ref().is_some_and(|p| p.normal_ready()) as u32,
			]);
			let mask = self.mask.as_mut().unwrap();
			gl.active_texture(glow::TEXTURE8);
			gl.bind_texture(glow::TEXTURE_2D, None);
			if mask.key != key
			{
				let scissor = gl.is_enabled(glow::SCISSOR_TEST);
				let mut color = [0.0; 4];
				gl.get_parameter_f32_slice(glow::COLOR_CLEAR_VALUE, &mut color);
				gl.bind_framebuffer(glow::FRAMEBUFFER, Some(mask.framebuffer));
				gl.viewport(0, 0, size[0], size[1]);
				gl.disable(glow::SCISSOR_TEST);
				gl.disable(glow::DEPTH_TEST);
				gl.clear_color(1.0, 1.0, 1.0, 1.0);
				gl.clear(glow::COLOR_BUFFER_BIT);
				gl.uniform_1_i32(
					gl.get_uniform_location(program, "u_shadow_pass").as_ref(),
					1,
				);
				gl.draw_arrays(glow::TRIANGLES, 0, 3);
				gl.uniform_1_i32(
					gl.get_uniform_location(program, "u_shadow_pass").as_ref(),
					0,
				);
				gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer);
				gl.viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
				if scissor
				{
					gl.enable(glow::SCISSOR_TEST);
				}
				gl.enable(glow::DEPTH_TEST);
				gl.clear_color(color[0], color[1], color[2], color[3]);
				mask.key = key;
			}
			gl.bind_texture(glow::TEXTURE_2D, Some(mask.texture));
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_mask_ready").as_ref(), 1);
			gl.uniform_4_f32(
				gl.get_uniform_location(program, "u_shadow_viewport")
					.as_ref(),
				viewport[0] as f32,
				viewport[1] as f32,
				viewport[2] as f32,
				viewport[3] as f32,
			);
		}
	}

	pub(crate) fn update_heights(
		&mut self,
		gl: &glow::Context,
		tier: u32,
		shared: &Mutex<assets::Loading>,
	)
	{
		let mut state = shared.lock();
		if state.error.is_some()
		{
			return;
		}
		let images = match state.take()
		{
			Ok(Some(images)) => images,
			Ok(None) => return,
			Err(error) =>
			{
				state.error = Some(error);
				return;
			}
		};
		let mut textures = Vec::new();
		let result = (|| {
			for (i, image) in images.iter().enumerate()
			{
				let width = if i == 0 { tier * 1024 } else { tier * 512 };
				if image.width() != width || image.height() != tier * 512
				{
					return Err("Unexpected height texture dimensions".into());
				}
				let previous = self.textures.len();
				#[cfg(not(target_arch = "wasm32"))]
				let uploaded =
					self.upload_texture(gl, image.width(), image.height(), image.as_raw(), false);
				#[cfg(target_arch = "wasm32")]
				let uploaded = self.upload_browser_texture(gl, image, false);
				if self.textures.len() > previous
				{
					textures.push(self.textures.pop().unwrap());
				}
				uploaded?;
				// Packed bytes must never be averaged in mipmaps.
				unsafe {
					gl.bind_texture(glow::TEXTURE_2D, textures.last().copied());
					gl.tex_parameter_i32(
						glow::TEXTURE_2D,
						glow::TEXTURE_MIN_FILTER,
						glow::LINEAR as i32,
					);
					gl.tex_parameter_i32(
						glow::TEXTURE_2D,
						glow::TEXTURE_WRAP_S,
						if i == 0
						{
							glow::REPEAT
						}
						else
						{
							glow::CLAMP_TO_EDGE
						} as i32,
					);
					gl.bind_texture(glow::TEXTURE_2D, None);
				}
			}
			Ok::<_, String>(())
		})();
		if let Err(error) = result
		{
			state.error = Some(error);
		}
		else
		{
			std::mem::swap(&mut self.heights, &mut textures);
			self.height_tier = tier;
		}
		unsafe {
			for texture in textures
			{
				gl.delete_texture(texture);
			}
		}
	}

	pub(super) unsafe fn bind_heights(&self, gl: &glow::Context, program: glow::Program, live: bool)
	{
		unsafe {
			gl.uniform_1_i32(
				gl.get_uniform_location(program, "u_live_shadows").as_ref(),
				live as i32,
			);
			gl.uniform_1_i32(
				gl.get_uniform_location(program, "u_height_ready").as_ref(),
				(self.heights.len() == 2) as i32,
			);
			gl.uniform_1_i32(
				gl.get_uniform_location(program, "u_shadow_steps").as_ref(),
				if self.height_tier <= 2 { 32 } else { 48 },
			);
			for (i, name) in ["u_height", "u_polar_height"].iter().enumerate()
			{
				gl.active_texture(glow::TEXTURE6 + i as u32);
				gl.bind_texture(glow::TEXTURE_2D, self.heights.get(i).copied());
				gl.uniform_1_i32(
					gl.get_uniform_location(program, name).as_ref(),
					6 + i as i32,
				);
			}
		}
	}
}
