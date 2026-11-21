use super::{Program, Renderer, glow};
use crate::{detail, math};
use glow::HasContext as _;

unsafe fn upload(
	gl: &glow::Context,
	rgba: &[u8],
	side: i32,
	photo: bool,
	mipmaps: bool,
) -> Result<glow::Texture, String>
{
	unsafe {
		let texture = gl.create_texture()?;
		gl.bind_texture(glow::TEXTURE_2D, Some(texture));
		gl.tex_image_2d(
			glow::TEXTURE_2D,
			0,
			if photo
			{
				glow::SRGB8_ALPHA8
			}
			else
			{
				glow::RGBA8
			} as i32,
			side,
			side,
			0,
			glow::RGBA,
			glow::UNSIGNED_BYTE,
			glow::PixelUnpackData::Slice(Some(rgba)),
		);
		if mipmaps
		{
			gl.generate_mipmap(glow::TEXTURE_2D);
		}
		gl.tex_parameter_i32(
			glow::TEXTURE_2D,
			glow::TEXTURE_MIN_FILTER,
			if mipmaps
			{
				glow::LINEAR_MIPMAP_LINEAR
			}
			else
			{
				glow::LINEAR
			} as i32,
		);
		gl.tex_parameter_i32(
			glow::TEXTURE_2D,
			glow::TEXTURE_MAG_FILTER,
			glow::LINEAR as i32,
		);
		for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T]
		{
			gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::CLAMP_TO_EDGE as i32);
		}
		if gl.get_error() != glow::NO_ERROR
		{
			gl.delete_texture(texture);
			return Err("Detail texture upload failed".into());
		}
		Ok(texture)
	}
}

impl Renderer
{
	pub(crate) fn update_detail(&mut self, gl: &glow::Context, shared: &detail::Shared)
	{
		let Some((spec, pixels)) = shared.lock().pending.take()
		else
		{
			return;
		};
		let side = if spec.photo { 1024 } else { 512 };
		let count = side as usize * side as usize;
		if pixels.len() != count * if spec.photo { 3 } else { 7 }
		{
			return;
		}
		unsafe {
			gl.active_texture(glow::TEXTURE9);
			let rgba: Vec<u8> = pixels[..count * 3]
				.as_chunks::<3>()
				.0
				.iter()
				.flat_map(|p| [p[0], p[1], p[2], 255])
				.collect();
			let Ok(texture) = upload(gl, &rgba, side, spec.photo, true)
			else
			{
				return;
			};
			Self::anisotropy(gl, glow::TEXTURE_2D);
			let height = if spec.photo
			{
				None
			}
			else
			{
				gl.active_texture(glow::TEXTURE10);
				match upload(gl, &pixels[count * 3..], side, false, false)
				{
					Ok(height) => Some(height),
					Err(_) =>
					{
						gl.delete_texture(texture);
						return;
					}
				}
			};
			if let Some((_, old, old_height)) = self.detail.replace((spec, texture, height))
			{
				gl.delete_texture(old);
				if let Some(old_height) = old_height
				{
					gl.delete_texture(old_height);
				}
			}
			self.invalidate_shadow_mask();
		}
	}

	pub(super) unsafe fn bind_detail(&self, gl: &glow::Context, program: &Program, view: math::View)
	{
		unsafe {
			gl.uniform_1_i32(program.uniform("u_detail_kind"), 0);
			gl.active_texture(glow::TEXTURE9);
			gl.bind_texture(
				glow::TEXTURE_2D,
				Some(
					self.detail
						.as_ref()
						.map_or(self.textures[0], |(_, t, _)| *t),
				),
			);
			gl.active_texture(glow::TEXTURE10);
			gl.bind_texture(
				glow::TEXTURE_2D,
				Some(
					self.detail
						.as_ref()
						.and_then(|(_, _, h)| *h)
						.unwrap_or(self.textures[0]),
				),
			);
			if view.zoom < 16.0
			{
				return;
			}
			if let Some((spec, _, _)) = &self.detail
				&& spec.photo != self.live_ready()
			{
				let [a, b, c, d] = spec.bbox;
				gl.uniform_1_i32(
					program.uniform("u_detail_kind"),
					if spec.photo { 1 } else { 2 },
				);
				gl.uniform_1_i32(program.uniform("u_detail_pole"), spec.pole);
				gl.uniform_4_f32(
					program.uniform("u_detail_bounds"),
					a as f32,
					b as f32,
					(c - a) as f32,
					(d - b) as f32,
				);
			}
		}
	}
}
