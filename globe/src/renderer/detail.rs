use super::{Program, Renderer, glow};
use crate::{detail, math};
use glow::HasContext as _;

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
		unsafe {
			let Ok(texture) = gl.create_texture()
			else
			{
				return;
			};
			gl.active_texture(glow::TEXTURE9);
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			let rgba: Vec<u8> = pixels
				.as_chunks::<3>()
				.0
				.iter()
				.flat_map(|p| [p[0], p[1], p[2], 255])
				.collect();
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				if spec.photo
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
				glow::PixelUnpackData::Slice(Some(&rgba)),
			);
			gl.generate_mipmap(glow::TEXTURE_2D);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
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
			Self::anisotropy(gl, glow::TEXTURE_2D);
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
			if gl.get_error() != glow::NO_ERROR
			{
				gl.delete_texture(texture);
				return;
			}
			if let Some((_, old)) = self.detail.replace((spec, texture))
			{
				gl.delete_texture(old);
			}
		}
	}

	pub(super) unsafe fn bind_detail(&self, gl: &glow::Context, program: &Program, view: math::View)
	{
		unsafe {
			gl.uniform_1_i32(program.uniform("u_detail_kind"), 0);
			gl.active_texture(glow::TEXTURE9);
			gl.bind_texture(
				glow::TEXTURE_2D,
				Some(self.detail.as_ref().map_or(self.textures[0], |(_, t)| *t)),
			);
			if view.zoom < 16.0
			{
				return;
			}
			if let Some((spec, _)) = &self.detail
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
