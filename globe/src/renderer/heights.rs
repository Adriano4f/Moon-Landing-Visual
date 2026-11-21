use super::{Program, Renderer, assets, glow};
use egui::mutex::Mutex;
use glow::HasContext as _;

pub(super) struct SurfaceUpload
{
	tier: u32,
	images: std::vec::IntoIter<assets::TextureImage>,
	pub textures: Vec<glow::Texture>,
}

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
					"Shadow framebuffer is incomplete; keeping overview lighting".into(),
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
	pub(super) fn invalidate_shadow_mask(&mut self)
	{
		if let Some(mask) = &mut self.mask
		{
			mask.key.clear();
		}
	}

	pub(super) unsafe fn shadow_mask(
		&mut self,
		gl: &glow::Context,
		program: &Program,
		view: crate::math::View,
		sun: [f32; 3],
		framebuffer: Option<glow::Framebuffer>,
	) -> bool
	{
		unsafe {
			gl.uniform_1_i32(program.uniform("u_shadow_pass"),
				0,
			);
			gl.uniform_1_i32(program.uniform("u_mask_ready"), 0);
			if !self.live_ready() || self.mask_failed
			{
				return false;
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
						return false;
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
				self.live_normal.is_some() as u32,
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
				gl.uniform_1_i32(program.uniform("u_shadow_pass"),
					1);
				gl.disable(glow::BLEND);
				gl.draw_arrays(glow::TRIANGLES, 0, 6);
				gl.enable(glow::BLEND);
				gl.uniform_1_i32(program.uniform("u_shadow_pass"),
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
			gl.uniform_1_i32(program.uniform("u_mask_ready"), 1);
			gl.uniform_4_f32(
				program.uniform("u_shadow_viewport"),
				viewport[0] as f32,
				viewport[1] as f32,
				viewport[2] as f32,
				viewport[3] as f32,
			);
			true
		}
	}

	pub(crate) fn live_ready(&self) -> bool
	{
		self.live_shadows && self.surface_tier == Some(self.tier) && !self.mask_failed
	}

	pub(crate) fn update_surface(
		&mut self,
		gl: &glow::Context,
		tier: u32,
		shared: &Mutex<assets::Loading<5>>,
	) -> bool
	{
		let mut state = shared.lock();
		if !self.live_shadows || state.error.is_some()
		{
			return false;
		}
		if self
			.surface_upload
			.as_ref()
			.is_some_and(|upload| upload.tier != tier)
		{
			for texture in self.surface_upload.take().unwrap().textures
			{
				unsafe {
					gl.delete_texture(texture);
				}
			}
		}
		if self.surface_upload.is_none()
		{
			match state.take()
			{
				Ok(Some(images)) =>
				{
					self.surface_upload = Some(SurfaceUpload {
						tier,
						images: Vec::from(images).into_iter(),
						textures: Vec::new(),
					})
				}
				Ok(None) => return false,
				Err(error) =>
				{
					state.error = Some(error);
					return false;
				}
			}
		}
		let mut upload = self.surface_upload.take().unwrap();
		let result = (|| {
			if let Some(image) = upload.images.next()
			{
				let i = upload.textures.len();
				let height_tier = tier.min(4);
				let dimensions = [
					(tier * 1024, tier * 512),
					(tier * 1024, tier * 512),
					(height_tier * 1024, height_tier * 512),
					(height_tier * 512, height_tier * 512),
					(height_tier * 512, height_tier * 512),
				];
				if (image.width(), image.height()) != dimensions[i]
				{
					return Err("Unexpected lighting texture dimensions".into());
				}
				let previous = self.textures.len();
				#[cfg(not(target_arch = "wasm32"))]
				let uploaded =
					self.upload_texture(gl, image.width(), image.height(), image.as_raw(), i == 0);
				#[cfg(target_arch = "wasm32")]
				let uploaded = self.upload_browser_texture(gl, &image, i == 0);
				if self.textures.len() > previous
				{
					upload.textures.push(self.textures.pop().unwrap());
				}
				uploaded?;
				if i == 2 || i == 3
				{
					unsafe {
						gl.bind_texture(glow::TEXTURE_2D, upload.textures.last().copied());
						gl.tex_parameter_i32(
							glow::TEXTURE_2D,
							glow::TEXTURE_MIN_FILTER,
							glow::LINEAR as i32,
						);
						gl.tex_parameter_i32(
							glow::TEXTURE_2D,
							glow::TEXTURE_WRAP_S,
							if i == 2
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
			}
			Ok::<_, String>(())
		})();
		if let Err(error) = result
		{
			state.error = Some(error);
			for texture in upload.textures
			{
				unsafe {
					gl.delete_texture(texture);
				}
			}
			return false;
		}
		if upload.textures.len() != 5
		{
			self.surface_upload = Some(upload);
			return true;
		}
		let normal = upload.textures.pop().unwrap();
		let heights = upload.textures.split_off(2);
		let old_textures = std::mem::replace(&mut self.textures, upload.textures);
		let old_heights = std::mem::replace(&mut self.heights, heights);
		let old_normal = self.live_normal.replace(normal);
		self.pending = None;
		self.tier = tier;
		self.height_tier = tier.min(4);
		self.surface_tier = Some(tier);
		self.visible_announced = false;
		unsafe {
			for texture in old_textures
				.into_iter()
				.chain(old_heights)
				.chain(old_normal)
			{
				gl.delete_texture(texture);
			}
		}
		false
	}

	pub(super) unsafe fn bind_heights(&self, gl: &glow::Context, program: &Program, live: bool)
	{
		unsafe {
			gl.uniform_1_i32(program.uniform("u_live_shadows"),
				live as i32,
			);
			gl.uniform_1_i32(
				program.uniform("u_height_ready"),
				(self.heights.len() == 2) as i32,
			);
			gl.uniform_1_i32(
				program.uniform("u_shadow_steps"),
				if self.height_tier <= 2 { 32 } else { 48 },
			);
			for (i, texture) in self.heights.iter().enumerate()
			{
				gl.active_texture(glow::TEXTURE6 + i as u32);
				gl.bind_texture(glow::TEXTURE_2D, Some(*texture));
			}
		}
	}
}
