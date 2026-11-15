use crate::assets::TextureImage;

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn image(url: String, done: impl FnOnce(Result<TextureImage, String>) + Send + 'static)
{
	let decode = move |bytes: Result<Vec<u8>, String>| {
		done(bytes.and_then(|b| {
			image::load_from_memory(&b)
				.map(|i| i.to_rgb8())
				.map_err(|e| e.to_string())
		}));
	};
	if url.starts_with("https://")
	{
		ehttp::fetch(ehttp::Request::get(url), move |response| {
			decode(response.and_then(|r| {
				if r.ok
				{
					Ok(r.bytes)
				}
				else
				{
					Err(format!("Polar HTTP {}", r.status))
				}
			}));
		});
	}
	else
	{
		crate::assets::read_native(url, decode);
	}
}

#[cfg(target_arch = "wasm32")]
pub(super) fn image(url: String, done: impl FnOnce(Result<TextureImage, String>) + 'static)
{
	wasm_bindgen_futures::spawn_local(async move {
		let result = match web_sys::HtmlImageElement::new()
		{
			Ok(image) =>
			{
				image.set_cross_origin(Some("anonymous"));
				image.set_src(&if url.starts_with("https://")
				{
					url
				}
				else
				{
					format!("assets/globe/{url}")
				});
				wasm_bindgen_futures::JsFuture::from(image.decode())
					.await
					.map(|_| send_wrapper::SendWrapper::new(image))
					.map_err(|e| format!("Polar image decode: {e:?}"))
			}
			Err(e) => Err(format!("Polar image construction: {e:?}")),
		};
		done(result);
	});
}
