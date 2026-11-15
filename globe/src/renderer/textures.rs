use super::{Renderer, glow};
use glow::HasContext as _;

impl Renderer
{
	// Upload RGB bytes and configure mipmapped sampling; C++ receives a const byte span plus dimensions.
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
		// Pixel storage is converted only after allocation.
		// SAFETY: the context is current during initialization; decoded RGB storage is
		// width * height * 3 bytes. SRGB8_ALPHA8 decodes albedo before filtering; RGB8 stores normals.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Read an integer capability into Rust; C++ glGetIntegerv writes through an output pointer.
			let max_side = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32;
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if width > max_side || height > max_side
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err("Moon texture exceeds the device texture-size limit".into());
			}
			// Select which texture unit subsequent binds affect; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Allocate a texture handle owned by this renderer; C++ glGenTextures.
			let texture = gl.create_texture()?;
			// Register ownership before any later upload operation can fail; C++ vector::push_back records the GLuint.
			self.textures.push(texture);
			// Bind or unbind an image at the active unit; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Set row-byte alignment for RGB upload, then restore egui's default; C++ glPixelStorei.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			Self::native_pixels(gl, width, height, pixels, srgb);
			Self::native_filter(gl);
			// Set row-byte alignment for RGB upload, then restore egui's default; C++ glPixelStorei.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
			// Bind or unbind an image at the active unit; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, None);
			// Retrieve and clear a GL error flag after upload; C++ glGetError.
			let error = gl.get_error();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if error != glow::NO_ERROR
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err(format!("Moon texture upload GL error: {error:#x}"));
			}
		}
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(())
	}

	// Upload browser-decoded pixels directly, without a WASM RGB buffer; C++ DOM image upload.
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
			// Register ownership before uploading, so construction failure cleans it; C++ owned GLuint.
			let texture = gl.create_texture()?;
			// Register cleanup ownership before using the handle; C++ vector::push_back.
			self.textures.push(texture);
			// Select the upload unit explicitly; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Bind this replacement without modifying the visible texture pair; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Normal maps are data, not browser color-corrected display images; C++ pixel-store policy.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				glow::NONE as i32,
			);
			// Keep the original top-row order, matching the native byte upload; C++ pixel-store policy.
			gl.pixel_store_i32(web_sys::WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
				0,
			);
			// Upload the browser-owned decoded source without copying through WASM; C++ DOM texture source.
			gl.tex_image_2d_with_html_image(
				glow::TEXTURE_2D,
				0,
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				image,
			);
			Self::browser_filter(gl);
			// Restore the WebGL default expected by other eframe painters; C++ state restoration.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				web_sys::WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
			);
			// Leave no upload binding behind; C++ glBindTexture with zero.
			gl.bind_texture(glow::TEXTURE_2D, None);
			// Convert driver failures into the normal recoverable loader error; C++ glGetError.
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
		// Expand only albedo to opaque RGBA; C++ builds the same vector from RGB triples.
		let rgba: Option<Vec<u8>> = srgb.then(|| {
			pixels
				.as_chunks::<3>()
				.0
				.iter()
				.flat_map(|p| [p[0], p[1], p[2], 255])
				.collect()
		});
		// Borrow expanded bytes when present, otherwise the original normal-map bytes; C++ span.
		let upload = rgba.as_deref().unwrap_or(pixels);

		// SAFETY: the caller has bound an owned texture on the current context.
		unsafe {
			// Upload level zero with explicit color-space storage; C++ glTexImage2D receives the pixel pointer.
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
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
		// SAFETY: the caller keeps this renderer's GL context current; C++ has the same driver contract.
		unsafe {
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as i32,
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			// Build all lower-resolution levels from level zero; C++ glGenerateMipmap.
			gl.generate_mipmap(glow::TEXTURE_2D);
			// Keep the texture's default anisotropy ratio of one: textureGrad with anisotropy made dotted limb arcs on llvmpipe.
			// Trilinear mipmaps and MSAA remain enabled; C++ likewise leaves GL_TEXTURE_MAX_ANISOTROPY_EXT at its default.
			// Set integer sampling/wrapping state; C++ glTexParameteri.
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
