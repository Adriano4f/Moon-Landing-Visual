fn opaque_rgb(rgba: &[u8]) -> Result<Vec<u8>, String>
{
	if rgba.len() != 256 * 256 * 4 || rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255)
	{
		return Err("Zoom photograph has missing coverage".into());
	}
	Ok(rgba
		.as_chunks::<4>()
		.0
		.iter()
		.flat_map(|p| [p[0], p[1], p[2]])
		.collect())
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn photo(url: String, done: impl FnOnce(Result<Vec<u8>, String>) + Send + 'static)
{
	ehttp::fetch(ehttp::Request::get(url), move |response| {
		done(response.and_then(|r| {
			if !r.ok
			{
				return Err(format!("Zoom photograph HTTP {}", r.status));
			}
			let image = image::load_from_memory(&r.bytes).map_err(|e| e.to_string())?;
			if image.width() != 256 || image.height() != 256
			{
				return Err("Unexpected photograph dimensions".into());
			}
			opaque_rgb(image.to_rgba8().as_raw())
		}));
	});
}

#[cfg(target_arch = "wasm32")]
pub(super) fn photo(url: String, done: impl FnOnce(Result<Vec<u8>, String>) + 'static)
{
	use wasm_bindgen::JsCast as _;
	wasm_bindgen_futures::spawn_local(async move {
		let result = async {
			let image = web_sys::HtmlImageElement::new()?;
			image.set_cross_origin(Some("anonymous"));
			image.set_src(&url);
			wasm_bindgen_futures::JsFuture::from(image.decode()).await?;
			if image.natural_width() != 256 || image.natural_height() != 256
			{
				return Err("Unexpected photograph dimensions".into());
			}
			let canvas = web_sys::OffscreenCanvas::new(256, 256)?;
			let context = canvas
				.get_context("2d")?
				.ok_or_else(|| wasm_bindgen::JsValue::from_str("No photograph decoder"))?
				.dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()?;
			context.draw_image_with_html_image_element(&image, 0.0, 0.0)?;
			let rgba = context.get_image_data(0.0, 0.0, 256.0, 256.0)?.data();
			opaque_rgb(&rgba.0).map_err(|e| wasm_bindgen::JsValue::from_str(&e))
		}
		.await;
		done(result.map_err(|e| format!("Zoom photograph decode: {e:?}")));
	});
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn empty_and_transparent_photographs_are_not_published()
	{
		assert!(opaque_rgb(&[]).is_err());
		let mut rgba = vec![255; 256 * 256 * 4];
		assert_eq!(opaque_rgb(&rgba).unwrap().len(), 256 * 256 * 3);
		rgba[3] = 0;
		assert!(opaque_rgb(&rgba).is_err());
	}
}
