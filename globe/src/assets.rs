use egui::mutex::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use image::RgbImage;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type TextureImage = RgbImage;
#[cfg(target_arch = "wasm32")]
pub(crate) type TextureImage = send_wrapper::SendWrapper<web_sys::HtmlImageElement>;
use std::sync::Arc;

/// Keep asynchronous CPU results separate from current-context GPU state.
pub(crate) struct Loading
{
	pub images: [Option<Result<TextureImage, String>>; 2],
	pub error: Option<String>,
}

pub(crate) fn first_tier() -> u32
{
	if cfg!(target_arch = "wasm32") { 0 } else { 2 }
}

/// Zero denotes a private 512-pixel preview, never a selectable zero-resolution texture.
pub(crate) fn label(tier: u32) -> String
{
	if tier == 0
	{
		"512px".into()
	}
	else
	{
		format!("{tier}k")
	}
}

pub(crate) fn select(max: i32, mobile: bool, forced: Option<&str>) -> Result<u32, String>
{
	let desired = match forced
	{
		Some("2" | "2k") => 2048,
		Some("4" | "4k") => 4096,
		Some("8" | "8k") => 8192,
		Some(_) => return Err("MOON_TIER must be 2, 4 or 8".into()),
		None =>
		{
			if mobile || cfg!(target_arch = "wasm32")
			{
				2048
			}
			else
			{
				8192
			}
		}
	};
	[8192, 4096, 2048]
		.into_iter()
		.find(|&n| n <= max && n <= desired)
		.map(|n| n as u32 / 1024)
		.ok_or("Moon needs MAX_TEXTURE_SIZE >= 2048".into())
}

pub(crate) fn preferences() -> (bool, Option<String>)
{
	#[cfg(not(target_arch = "wasm32"))]
	{
		(false, std::env::var("MOON_TIER").ok())
	}
	#[cfg(target_arch = "wasm32")]
	{
		let window = web_sys::window().expect("Browser window");
		let ua = window
			.navigator()
			// Read the user-agent string, which is a heuristic and not a hardware guarantee.
			.user_agent()
			.unwrap_or_default()
			.to_lowercase();
		let mobile = ["android", "iphone", "ipad", "mobile"]
			.iter()
			.any(|s| ua.contains(s))
			|| window
				// Read browser viewport width in CSS pixels; high-density screens still use logical CSS units.
				.inner_width()
				.ok()
				.and_then(|n| n.as_f64())
				.unwrap_or(0.0)
				<= 600.0;
		let forced = window
			.location()
			.search()
			.ok()
			.and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
			.and_then(|p| p.get("moon_tier"));
		(mobile, forced)
	}
}

pub(crate) fn load(tier: u32, ctx: Option<egui::Context>) -> Arc<Mutex<Loading>>
{
	let state = Arc::new(Mutex::new(Loading {
		images: [None, None],
		error: None,
	}));
	let suffix = if tier == 0
	{
		"preview".into()
	}
	else
	{
		format!("{tier}k")
	};
	let normal = if cfg!(target_arch = "wasm32")
	{
		format!("normal_{suffix}.webp")
	}
	else
	{
		format!("normal_{suffix}.png")
	};
	for (index, filename) in [format!("albedo_{suffix}.jpg"), normal]
		.into_iter()
		.enumerate()
	{
		let state = Arc::clone(&state);
		let ctx = ctx.clone();
		// Decode only native images in Rust; browser images stay in the native browser decoder.
		#[cfg(not(target_arch = "wasm32"))]
		let done = move |bytes: Result<Vec<u8>, String>| {
			let result = bytes.and_then(|b| {
				// Detect JPEG/PNG and decode only this downloaded image.
				image::load_from_memory(&b)
					.map(|i| i.to_rgb8())
					.map_err(|e| e.to_string())
			});
			state.lock().images[index] = Some(result);
			if let Some(ctx) = ctx
			{
				ctx.request_repaint();
			}
		};
		#[cfg(not(target_arch = "wasm32"))]
		read_native(filename, done);
		#[cfg(target_arch = "wasm32")]
		// Browser-native decoding runs asynchronously rather than in the WASM frame.
		read_browser(filename, state, index, ctx);
	}
	state
}

impl Loading
{
	/// Move both completed images out only when neither is missing or erroneous.
	pub fn take(&mut self) -> Result<Option<[TextureImage; 2]>, String>
	{
		for result in self.images.iter().flatten()
		{
			if let Err(error) = result
			{
				return Err(error.clone());
			}
		}
		if self.images.iter().all(Option::is_some)
		{
			Ok(Some([
				self.images[0].take().unwrap()?,
				self.images[1].take().unwrap()?,
			]))
		}
		else
		{
			Ok(None)
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn initial_and_selected_tier_labels()
	{
		assert_eq!(first_tier(), 2);
		assert_eq!(label(0), "512px");
		for tier in [2, 4, 8]
		{
			assert_eq!(label(tier), format!("{tier}k"));
		}
		assert!(select(8192, false, Some("0")).is_err());
	}
	#[test]
	fn tiers_and_loading()
	{
		assert_eq!(select(16384, false, None).unwrap(), 8);
		assert_eq!(select(16384, true, None).unwrap(), 2);
		assert_eq!(select(4096, false, Some("8")).unwrap(), 4);
		assert_eq!(select(8192, false, Some("2")).unwrap(), 2);
		assert!(select(1024, true, None).is_err());
		assert!(select(8192, false, Some("bad")).is_err());
		let mut s = Loading {
			images: [None, None],
			error: None,
		};
		assert!(s.take().unwrap().is_none());
		s.images[0] = Some(Ok(RgbImage::new(2, 1)));
		// Simulate an HTTP/disk/decoder failure for the second image.
		s.images[1] = Some(Err("missing file".into()));
		assert_eq!(s.take().unwrap_err(), "missing file");
		s.images[1] = Some(Ok(RgbImage::new(2, 1)));
		assert!(s.take().unwrap().is_some());
		assert!(s.take().unwrap().is_none());
	}
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_native(
	filename: String,
	done: impl FnOnce(Result<Vec<u8>, String>) + Send + 'static,
)
{
	std::thread::spawn(move || {
		let directory = std::env::var_os("MOON_ASSET_DIR")
			.map(std::path::PathBuf::from)
			.unwrap_or_else(|| {
				// Try assets/globe beside the executable before the development fallback.
				let beside = std::env::current_exe()
					.ok()
					.and_then(|p| p.parent().map(|p| p.join("assets/globe")));
				// Use the executable-relative directory only if it exists.
				beside.filter(|p| p.is_dir()).unwrap_or_else(|| {
					std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
				})
			});
		done(std::fs::read(directory.join(&filename)).map_err(|e| format!("{filename}: {e}")));
	});
}

#[cfg(target_arch = "wasm32")]
fn read_browser(
	filename: String,
	state: Arc<Mutex<Loading>>,
	index: usize,
	ctx: Option<egui::Context>,
)
{
	wasm_bindgen_futures::spawn_local(async move {
		let result = match web_sys::HtmlImageElement::new()
		{
			Ok(image) =>
			{
				image.set_src(&format!("assets/globe/{filename}"));
				// Await the browser's own decoder and retain its native upload source.
				wasm_bindgen_futures::JsFuture::from(image.decode())
					.await
					.map(|_| send_wrapper::SendWrapper::new(image))
					.map_err(|e| format!("{filename}: {e:?}"))
			}
			Err(e) => Err(format!("{filename}: {e:?}")),
		};
		// Publish a complete image or a genuine error, never a half-decoded buffer.
		state.lock().images[index] = Some(result);
		// Background completion must repaint even when the first Moon is already visible.
		if let Some(ctx) = ctx
		{
			ctx.request_repaint();
		}
	});
}
