use crate::polar::{Shared, Spec};
use crate::{assets, glow, renderer::Renderer};
use glow::HasContext as _;

pub(crate) struct Atlas
{
	color: glow::Texture,
	normal: Option<glow::Texture>,
	spec: Spec,
}

impl Atlas
{
	fn new(gl: &glow::Context, spec: Spec) -> Result<Self, String>
	{
		// SAFETY: the renderer's deferred callback owns the current context.
		unsafe {
			let color = gl.create_texture()?;
			gl.active_texture(glow::TEXTURE2);
			gl.bind_texture(glow::TEXTURE_2D, Some(color));
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::SRGB8_ALPHA8 as i32,
				spec.side as i32,
				spec.side as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(None),
			);
			Self::filter(gl);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				gl.delete_texture(color);
				return Err(format!("Polar allocation GL {error}"));
			}
			Ok(Self {
				color,
				normal: None,
				spec,
			})
		}
	}

	fn filter(gl: &glow::Context)
	{
		// SAFETY: the caller has bound the atlas or its complete normal texture.
		unsafe {
			for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T]
			{
				gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::CLAMP_TO_EDGE as i32);
			}
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
		}
	}

	fn tile(
		&self,
		gl: &glow::Context,
		job: usize,
		image: &assets::TextureImage,
	) -> Result<(), String>
	{
		if job >= self.spec.count() || image.width() != 256 || image.height() != 256
		{
			return Err("Unexpected polar tile geometry".into());
		}
		let (x, y) = self.spec.offset(job);
		// SAFETY: dimensions and atlas bounds are validated before this current-context upload.
		unsafe {
			gl.active_texture(glow::TEXTURE2);
			gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
			#[cfg(not(target_arch = "wasm32"))]
			gl.tex_sub_image_2d(
				glow::TEXTURE_2D,
				0,
				x as i32,
				y as i32,
				256,
				256,
				glow::RGB,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(image.as_raw())),
			);
			#[cfg(target_arch = "wasm32")]
			{
				gl.pixel_store_i32(
					web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
					glow::NONE as i32,
				);
				gl.pixel_store_i32(web_sys::WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
				gl.pixel_store_i32(
					web_sys::WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
					0,
				);
				gl.tex_sub_image_2d_with_html_image(
					glow::TEXTURE_2D,
					0,
					x as i32,
					y as i32,
					glow::RGBA,
					glow::UNSIGNED_BYTE,
					image,
				);
				gl.pixel_store_i32(
					web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
					web_sys::WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
				);
			}
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Polar tile GL {error}"));
			}
		}
		Ok(())
	}

	pub(crate) fn destroy(self, gl: &glow::Context)
	{
		// SAFETY: the owning renderer is destroyed with its context current.
		unsafe {
			gl.delete_texture(self.color);
			if let Some(n) = self.normal
			{
				gl.delete_texture(n);
			}
		}
	}

	fn finish(&self, gl: &glow::Context) -> Result<(), String>
	{
		// SAFETY: all level-zero pixels are uploaded before mipmaps become visible.
		unsafe {
			gl.active_texture(glow::TEXTURE2);
			gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
			gl.generate_mipmap(glow::TEXTURE_2D);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
			);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Polar mipmap GL {error}"));
			}
		}
		Ok(())
	}
}

impl Renderer
{
	pub(crate) fn update_polar(&mut self, gl: &glow::Context, shared: &Shared)
	{
		let (spec, work) = {
			let mut state = shared.lock();
			if state.error.is_some() || state.ready
			{
				return;
			}
			let Some(spec) = state.spec
			else
			{
				return;
			};
			(spec, state.pending.pop_front())
		};
		let result = (|| {
			if self.polar.is_none()
			{
				self.polar = Some(Atlas::new(gl, spec)?);
			}
			let Some((job, image)) = work
			else
			{
				return Ok(false);
			};
			if job == 0
			{
				self.upload_polar_normal(gl, image, spec)?;
			}
			else
			{
				self.polar.as_ref().unwrap().tile(gl, job, &image)?;
			}
			Ok(true)
		})();
		self.publish_polar(gl, shared, spec, result);
	}

	fn publish_polar(
		&self,
		gl: &glow::Context,
		shared: &Shared,
		spec: Spec,
		result: Result<bool, String>,
	)
	{
		let mut state = shared.lock();
		match result
		{
			Ok(true) =>
			{
				state.uploaded += 1;
				if state.uploaded == spec.count()
				{
					if let Err(error) = self.polar.as_ref().unwrap().finish(gl)
					{
						state.error = Some(error);
						return;
					}
					state.ready = true;
					#[cfg(target_arch = "wasm32")]
					web_sys::console::log_1(
						&format!("Polar detail visible: {}px", spec.side).into(),
					);
				}
			}
			Ok(false) =>
			{}
			Err(error) =>
			{
				state.error = Some(error);
				state.pending.clear();
			}
		}
	}

	fn upload_polar_normal(
		&mut self,
		gl: &glow::Context,
		image: assets::TextureImage,
		spec: Spec,
	) -> Result<(), String>
	{
		if image.width() != spec.side || image.height() != spec.side
		{
			return Err("Unexpected polar normal geometry".into());
		}
		let previous = self.textures.len();
		#[cfg(not(target_arch = "wasm32"))]
		let result = self.upload_texture(gl, image.width(), image.height(), image.as_raw(), false);
		#[cfg(target_arch = "wasm32")]
		let result = self.upload_browser_texture(gl, &image, false);
		let texture = if self.textures.len() > previous
		{
			self.textures.pop()
		}
		else
		{
			None
		};
		if result.is_err()
		{
			// SAFETY: discard only this failed upload's registered texture, not the visible pair.
			if let Some(t) = texture
			{
				unsafe {
					gl.delete_texture(t);
				}
			}
			return result;
		}
		self.polar.as_mut().unwrap().normal = texture;
		Ok(())
	}

	pub(crate) fn bind_polar(&self, gl: &glow::Context, program: glow::Program, ready: bool)
	{
		let color = self.polar.as_ref().map_or(self.textures[0], |p| p.color);
		let normal = self
			.polar
			.as_ref()
			.and_then(|p| p.normal)
			.unwrap_or(self.textures[1]);
		// SAFETY: all samplers have complete textures even before the optional atlas is ready.
		unsafe {
			for (unit, name, texture) in
				[(2, "u_polar_color", color), (3, "u_polar_normal", normal)]
			{
				gl.active_texture(glow::TEXTURE0 + unit);
				gl.bind_texture(glow::TEXTURE_2D, Some(texture));
				gl.uniform_1_i32(gl.get_uniform_location(program, name).as_ref(), unit as i32);
			}
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_polar_ready").as_ref(),
				if ready { 1.0 } else { 0.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_polar_normal_ready").as_ref(),
				if self.polar.as_ref().is_some_and(|p| p.normal.is_some()) { 1.0 } else { 0.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_polar_extent").as_ref(),
				crate::polar::EXTENT,
			);
		}
	}
}
