use super::{Renderer, glow};
use glow::HasContext as _;

impl Renderer
{
	#[cfg(not(target_arch = "wasm32"))]
	pub(super) fn upload_texture(
		&mut self,
		gl: &glow::Context,
		width: u32,
		height: u32,
		pixels: &[u8],
		srgb: bool,
	) -> Result<(), String>
	{
		// SAFETY: the context is current during initialization; decoded RGB storage is
		// width * height * 3 bytes. SRGB8_ALPHA8 decodes albedo before filtering; RGB8 stores normals.
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
			// RGB rows may not be four-byte aligned; restore egui's default afterward.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			Self::native_pixels(gl, width, height, pixels, srgb);
			Self::native_filter(gl);
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

	/// Upload browser-decoded pixels directly, without a WASM RGB buffer.
	#[cfg(target_arch = "wasm32")]
	pub(super) fn upload_browser_texture(
		&mut self,
		gl: &glow::Context,
		image: &web_sys::HtmlImageElement,
		srgb: bool,
	) -> Result<(), String>
	{
		// SAFETY: callback runs on the current GL context and the image's creation thread.
		unsafe {
			let texture = gl.create_texture()?;
			self.textures.push(texture);
			gl.active_texture(glow::TEXTURE0);
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Normal maps are data, not browser color-corrected display images.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				glow::NONE as i32,
			);
			// Match the top-row order of native uploads.
			gl.pixel_store_i32(web_sys::WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
				0,
			);
			gl.tex_image_2d_with_html_image(
				glow::TEXTURE_2D,
				0,
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				image,
			);
			Self::browser_filter(gl);
			// Restore the WebGL default expected by other eframe painters.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				web_sys::WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
			);
			gl.bind_texture(glow::TEXTURE_2D, None);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Browser texture upload GL error {error}"));
			}
		}
		Ok(())
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn native_pixels(gl: &glow::Context, width: u32, height: u32, pixels: &[u8], srgb: bool)
	{
		// WebGL2 can generate mipmaps for SRGB8_ALPHA8, but not for bare SRGB8.
		// Expand only albedo to opaque RGBA.
		let rgba: Option<Vec<u8>> = srgb.then(|| {
			pixels
				.as_chunks::<3>()
				.0
				.iter()
				.flat_map(|p| [p[0], p[1], p[2], 255])
				.collect()
		});
		let upload = rgba.as_deref().unwrap_or(pixels);

		// SAFETY: the caller has bound an owned texture on the current context.
		unsafe {
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				width as i32,
				height as i32,
				0,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(upload)),
			);
		}
	}

	#[cfg(not(target_arch = "wasm32"))]
	unsafe fn native_filter(gl: &glow::Context)
	{
		// SAFETY: the caller keeps this renderer's GL context current.
		unsafe {
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
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			gl.generate_mipmap(glow::TEXTURE_2D);
			// Keep the texture's default anisotropy ratio of one: textureGrad with anisotropy made dotted limb arcs on llvmpipe.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_WRAP_T,
				glow::CLAMP_TO_EDGE as i32,
			);
		}
	}

	#[cfg(target_arch = "wasm32")]
	unsafe fn browser_filter(gl: &glow::Context)
	{
		// SAFETY: called only while a texture is bound in the renderer's current context.
		unsafe {
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
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_WRAP_T,
				glow::CLAMP_TO_EDGE as i32,
			);
			gl.generate_mipmap(glow::TEXTURE_2D);
		}
	}
}
