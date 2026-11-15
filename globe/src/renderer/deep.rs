//! Fixed-size virtual texture: one page lookup and one imagery sample per pixel.
use super::{Renderer, assets, glow};
use crate::deep::{Shared, Tile};
use glow::HasContext as _;

const SLOTS: usize = 96;
const WIDTH: usize = 576;
const HEIGHT: usize = 256;
const UPLOADS_PER_FRAME: usize = 4;

struct Entry
{
	tile: Tile,
	used: u64,
}

pub(super) struct Atlas
{
	color: glow::Texture,
	pages: glow::Texture,
	entries: Vec<Option<Entry>>,
	clock: u64,
	failed: bool,
}

impl Atlas
{
	pub(super) fn new(gl: &glow::Context, slots: usize) -> Result<Self, String>
	{
		// SAFETY: the paint callback owns a current WebGL2/OpenGL context.
		unsafe {
			let color = gl.create_texture()?;
			let pages = match gl.create_texture()
			{
				Ok(pages) => pages,
				Err(error) =>
				{
					gl.delete_texture(color);
					return Err(error);
				}
			};
			gl.active_texture(glow::TEXTURE4);
			gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(color));
			gl.tex_storage_3d(
				glow::TEXTURE_2D_ARRAY,
				1,
				glow::SRGB8_ALPHA8,
				256,
				256,
				slots as i32,
			);
			Self::filter(gl, glow::TEXTURE_2D_ARRAY, glow::LINEAR);
			gl.active_texture(glow::TEXTURE5);
			gl.bind_texture(glow::TEXTURE_2D, Some(pages));
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA8 as i32,
				WIDTH as i32,
				HEIGHT as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(&vec![0; WIDTH * HEIGHT * 4])),
			);
			Self::filter(gl, glow::TEXTURE_2D, glow::NEAREST);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				gl.delete_texture(color);
				gl.delete_texture(pages);
				return Err(format!("Deep zoom allocation GL {error}"));
			}
			Ok(Self {
				color,
				pages,
				entries: (0..slots).map(|_| None).collect(),
				clock: 0,
				failed: false,
			})
		}
	}

	unsafe fn filter(gl: &glow::Context, target: u32, filter: u32)
	{
		// SAFETY: target has a bound, complete texture owned by this atlas.
		unsafe {
			for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T]
			{
				gl.tex_parameter_i32(target, parameter, glow::CLAMP_TO_EDGE as i32);
			}
			gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, filter as i32);
			gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, filter as i32);
		}
	}

	fn upload(
		&self,
		gl: &glow::Context,
		slot: usize,
		image: &assets::TextureImage,
	) -> Result<(), String>
	{
		if image.width() != 256 || image.height() != 256
		{
			return Err("Unexpected deep zoom tile dimensions".into());
		}
		// SAFETY: the validated image fits exactly one allocated array layer.
		unsafe {
			gl.active_texture(glow::TEXTURE4);
			gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.color));
			#[cfg(not(target_arch = "wasm32"))]
			{
				let rgba: Vec<u8> = image
					.as_raw()
					.as_chunks::<3>()
					.0
					.iter()
					.flat_map(|p| [p[0], p[1], p[2], 255])
					.collect();
				gl.tex_sub_image_3d(
					glow::TEXTURE_2D_ARRAY,
					0,
					0,
					0,
					slot as i32,
					256,
					256,
					1,
					glow::RGBA,
					glow::UNSIGNED_BYTE,
					glow::PixelUnpackData::Slice(Some(&rgba)),
				);
			}
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
				gl.tex_sub_image_3d_with_html_image_element(
					glow::TEXTURE_2D_ARRAY,
					0,
					0,
					0,
					slot as i32,
					256,
					256,
					1,
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
				return Err(format!("Deep zoom upload GL {error}"));
			}
		}
		Ok(())
	}

	fn page_pixels(entries: &[Option<Entry>]) -> Vec<u8>
	{
		let mut pixels = vec![0_u8; WIDTH * HEIGHT * 4];
		let mut sorted: Vec<_> = entries
			.iter()
			.enumerate()
			.filter_map(|(slot, entry)| entry.as_ref().map(|e| (slot, e.tile)))
			.collect();
		sorted.sort_by_key(|(_, tile)| tile.z);
		for (slot, tile) in sorted
		{
			let shift = if tile.pole != 0 { 5 } else { 8 } - tile.z;
			let size = 1_usize << shift;
			let start_x = tile.col as usize * size
				+ match tile.pole
				{
					-1 => 512,
					1 => 544,
					_ => 0,
				};
			let start_y = tile.row as usize * size;
			for y in start_y..start_y + size
			{
				for x in start_x..start_x + size
				{
					let offset = (y * WIDTH + x) * 4;
					pixels[offset] = slot as u8 + 1;
					pixels[offset + 1] = tile.z as u8;
				}
			}
		}
		pixels
	}

	fn publish(&self, gl: &glow::Context) -> Result<(), String>
	{
		let pixels = Self::page_pixels(&self.entries);
		// SAFETY: page dimensions are fixed and all entries address allocated layers.
		unsafe {
			gl.active_texture(glow::TEXTURE5);
			gl.bind_texture(glow::TEXTURE_2D, Some(self.pages));
			gl.tex_sub_image_2d(
				glow::TEXTURE_2D,
				0,
				0,
				0,
				WIDTH as i32,
				HEIGHT as i32,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(&pixels)),
			);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Deep zoom page table GL {error}"));
			}
		}
		Ok(())
	}

	pub(super) fn destroy(self, gl: &glow::Context)
	{
		// SAFETY: teardown runs with the owning context current.
		unsafe {
			gl.delete_texture(self.color);
			gl.delete_texture(self.pages);
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn finer_pages_override_parents_and_polar_maps_are_disjoint()
	{
		let entry = |z, row, col, pole| {
			Some(Entry {
				tile: Tile { z, row, col, pole },
				used: 0,
			})
		};
		let entries = vec![
			entry(8, 255, 511, 0),
			entry(0, 0, 0, -1),
			entry(5, 31, 31, 1),
			entry(0, 0, 1, 0),
		];
		let pixels = Atlas::page_pixels(&entries);
		let page =
			|x: usize, y: usize| pixels[(y * WIDTH + x) * 4..(y * WIDTH + x) * 4 + 2].to_vec();
		assert_eq!(page(511, 255), [1, 8]);
		assert_eq!(page(510, 255), [4, 0]);
		assert_eq!(page(256, 0), [4, 0]);
		assert_eq!(page(255, 0), [0, 0]);
		assert_eq!(page(512, 0), [2, 0]);
		assert_eq!(page(543, 31), [2, 0]);
		assert_eq!(page(575, 31), [3, 5]);
		assert_eq!(page(544, 0), [0, 0]);
		assert_eq!(page(575, 255), [0, 0]);
	}
}

impl Renderer
{
	pub(crate) fn update_deep(&mut self, gl: &glow::Context, shared: &Shared)
	{
		let mut state = shared.lock();
		if self.deep.as_ref().is_some_and(|atlas| atlas.failed)
		{
			state.pending.clear();
			return;
		}
		if state.pending.is_empty()
		{
			return;
		}
		if self
			.deep
			.as_ref()
			.is_none_or(|atlas| atlas.entries.len() < SLOTS)
		{
			match Atlas::new(gl, SLOTS)
			{
				Ok(atlas) =>
				{
					if let Some(previous) = self.deep.replace(atlas)
					{
						previous.destroy(gl);
					}
				}
				Err(error) =>
				{
					if let Some(atlas) = self.deep.as_mut()
					{
						atlas.failed = true;
					}
					state.disable(error);
					return;
				}
			}
		}
		let atlas = self.deep.as_mut().unwrap();
		atlas.clock += 1;
		for entry in atlas.entries.iter_mut().flatten()
		{
			if state.wanted.contains(&entry.tile)
			{
				entry.used = atlas.clock;
			}
		}
		let mut changed = false;
		for _ in 0..UPLOADS_PER_FRAME
		{
			let Some((tile, image)) = state.pending.pop_front()
			else
			{
				break;
			};
			if !state.wanted.contains(&tile) || state.resident.contains(&tile)
			{
				continue;
			}
			let slot = atlas.entries.iter().position(Option::is_none).or_else(|| {
				atlas
					.entries
					.iter()
					.enumerate()
					.filter_map(|(slot, entry)| {
						let entry = entry.as_ref()?;
						(!state.wanted.contains(&entry.tile)).then_some((slot, entry.used))
					})
					.min_by_key(|(_, used)| *used)
					.map(|(slot, _)| slot)
			});
			let Some(slot) = slot
			else
			{
				break;
			};
			if let Some(old) = atlas.entries[slot].take()
			{
				state.resident.remove(&old.tile);
			}
			changed = true;
			match atlas.upload(gl, slot, &image)
			{
				Ok(()) =>
				{
					atlas.entries[slot] = Some(Entry {
						tile,
						used: atlas.clock,
					});
					state.resident.insert(tile);
				}
				Err(error) => state.error = Some(error),
			}
		}
		if changed && let Err(error) = atlas.publish(gl)
		{
			atlas.failed = true;
			state.disable(error);
		}
	}

	pub(super) fn bind_deep(&self, gl: &glow::Context, program: glow::Program)
	{
		// SAFETY: every active sampler has a complete texture of its declared type.
		unsafe {
			gl.uniform_1_i32(
				gl.get_uniform_location(program, "u_deep_ready").as_ref(),
				i32::from(self.deep.as_ref().is_some_and(|a| !a.failed)),
			);
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_deep_color").as_ref(), 4);
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_deep_pages").as_ref(), 5);
			gl.active_texture(glow::TEXTURE4);
			gl.bind_texture(glow::TEXTURE_2D_ARRAY, self.deep.as_ref().map(|a| a.color));
			gl.active_texture(glow::TEXTURE5);
			gl.bind_texture(
				glow::TEXTURE_2D,
				Some(self.deep.as_ref().map_or(self.textures[0], |a| a.pages)),
			);
		}
	}
}
