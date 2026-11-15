// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui::mutex::Mutex;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
#[cfg(not(target_arch = "wasm32"))]
use image::RgbImage;

// Native workers return owned RGB bytes; C++ image value alias.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type TextureImage = RgbImage;
// Keep DOM images on their creation thread even across Send-bound UI callbacks; C++ checked thread-affine handle.
#[cfg(target_arch = "wasm32")]
pub(crate) type TextureImage = send_wrapper::SendWrapper<web_sys::HtmlImageElement>;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::sync::Arc;

// Keep asynchronous CPU results separate from current-context GPU state; C++ defines a similar aggregate.
pub(crate) struct Loading
{
	// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
	pub images: [Option<Result<TextureImage, String>>; 2],
	// An optional terminal loading/upload diagnostic; C++ optional<string>.
	pub error: Option<String>,
}

// Keep native overview quality while reducing the browser's critical-path bytes; C++ target policy.
pub(crate) fn first_tier() -> u32
{
	if cfg!(target_arch = "wasm32") { 0 } else { 2 }
}

// Zero denotes a private 512-pixel preview, never a selectable zero-resolution texture; C++ label helper.
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

// Select one supported tier or return a diagnostic; C++ expected<unsigned,string> replaces Result.
pub(crate) fn select(max: i32, mobile: bool, forced: Option<&str>) -> Result<u32, String>
{
	// Choose a requested width before applying the GPU limit; C++ switch selects an integer.
	let desired = match forced
	{
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some("2" | "2k") => 2048,
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some("4" | "4k") => 4096,
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some("8" | "8k") => 8192,
		// Match or construct the present optional alternative; C++ optional holds this value.
		Some(_) => return Err("MOON_TIER must be 2, 4 or 8".into()),
		// Represent an absent optional alternative; C++ nullopt.
		None =>
		{
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if mobile || cfg!(target_arch = "wasm32")
			{
				2048
			}
			// Handle the complementary case; C++ uses the same else branch.
			else
			{
				8192
			}
		}
	};
	// Search widths largest first so clamping preserves the highest allowed tier; C++ scans an ordered array.
	[8192, 4096, 2048]
		// Consume this array into an iterator; C++ iterates its owned local elements.
		.into_iter()
		// Return the first tier satisfying both bounds; C++ ranges::find_if plus optional conversion.
		.find(|&n| n <= max && n <= desired)
		// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
		.map(|n| n as u32 / 1024)
		// Convert absence into the given error value; C++ constructs expected from optional after a presence test.
		.ok_or("Moon needs MAX_TEXTURE_SIZE >= 2048".into())
}

// Read platform-specific quality hints without doing I/O; C++ conditional compilation selects the implementation.
pub(crate) fn preferences() -> (bool, Option<String>)
{
	// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
	#[cfg(not(target_arch = "wasm32"))]
	{
		// Native machines default to desktop policy; C++ getenv plus optional<string> supplies an override.
		(false, std::env::var("MOON_TIER").ok())
	}
	// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
	#[cfg(target_arch = "wasm32")]
	{
		// Obtain the browser Window object; C++ native builds do not have this DOM API.
		let window = web_sys::window().expect("Browser window");
		// Normalize the user-agent string for case-insensitive matching; C++ transforms a string to lowercase.
		let ua = window
			// Access the browser's Navigator for device hints; this is a web platform API.
			.navigator()
			// Read the user-agent string, which is a heuristic and not a hardware guarantee.
			.user_agent()
			// Use an empty string if the browser provides no user agent; C++ optional::value_or(string{}).
			.unwrap_or_default()
			// Allocate a lowercase Unicode string for matching; C++ transforms characters into a new string.
			.to_lowercase();
		// Record whether the browser looks mobile; C++ stores the corresponding bool.
		let mobile = ["android", "iphone", "ipad", "mobile"]
			// Borrow elements without moving them; C++ cbegin/cend or a const range.
			.iter()
			// Return true at the first matching heuristic/extension; C++ ranges::any_of short-circuits.
			.any(|s| ua.contains(s))
			|| window
				// Read browser viewport width in CSS pixels; high-density screens still use logical CSS units.
				.inner_width()
				// Discard a platform error and retain only success as an Option; C++ converts expected to optional.
				.ok()
				// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
				.and_then(|n| n.as_f64())
				// Substitute the stated default if absent; C++ optional::value_or.
				.unwrap_or(0.0)
				<= 600.0;
		// Read an optional user tier override; C++ would use optional<string>.
		let forced = window
			// Access the current browser URL; this is a web platform API.
			.location()
			// Read its query string, including the leading question mark if present.
			.search()
			// Discard a platform error and retain only success as an Option; C++ converts expected to optional.
			.ok()
			// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
			.and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok())
			// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
			.and_then(|p| p.get("moon_tier"));
		// Return both browser hints without allocating another aggregate; C++ returns pair<bool,optional<string>>.
		(mobile, forced.or_else(|| (!mobile).then(|| "4".into())))
	}
}

// Start exactly two asynchronous image reads for one tier; C++ shared_ptr keeps completion state alive.
pub(crate) fn load(tier: u32, ctx: Option<egui::Context>) -> Arc<Mutex<Loading>>
{
	// Share two pending image results with asynchronous producers; C++ shared_ptr protects a mutex-backed aggregate.
	let state = Arc::new(Mutex::new(Loading {
		// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
		images: [None, None],
		// An optional terminal loading/upload diagnostic; C++ optional<string>.
		error: None,
	}));
	// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
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
		// Consume this array into an iterator; C++ iterates its owned local elements.
		.into_iter()
		// Attach each element's zero-based index; C++ an indexed range or explicit counter.
		.enumerate()
	{
		// Share two pending image results with asynchronous producers; C++ shared_ptr protects a mutex-backed aggregate.
		let state = Arc::clone(&state);
		// Wake a paused globe when either upgrade image completes; C++ optional UI event-loop handle.
		let ctx = ctx.clone();
		// Decode only native images in Rust; browser images stay in the native browser decoder.
		#[cfg(not(target_arch = "wasm32"))]
		// Capture the completion action by ownership; C++ uses a value-capturing callable.
		let done = move |bytes: Result<Vec<u8>, String>| {
			// Carry either the next successful value or a diagnostic; C++ could use expected<T,string>.
			let result = bytes.and_then(|b| {
				// Detect JPEG/PNG and decode only this downloaded image; C++ would use an image decoder on a byte span.
				image::load_from_memory(&b)
					// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
					.map(|i| i.to_rgb8())
					// Transform only an error into a printable diagnostic; C++ maps the error alternative of expected.
					.map_err(|e| e.to_string())
			});
			// Access an image-completion slot under exclusive synchronization; C++ locks its mutex before accessing the slot.
			state.lock().images[index] = Some(result);
			if let Some(ctx) = ctx
			{
				ctx.request_repaint();
			}
		};
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Move disk I/O and image decoding to a native worker thread; C++ launches a thread with a value capture.
		read_native(filename, done);
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Browser-native decoding runs asynchronously rather than in the WASM frame; C++ platform decoder task.
		read_browser(filename, state, index, ctx);
	}
	state
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Loading
{
	// Move both completed images out only when neither is missing or erroneous; C++ expected<optional<array<Image,2>>,string>.
	pub fn take(&mut self) -> Result<Option<[TextureImage; 2]>, String>
	{
		// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
		for result in self.images.iter().flatten()
		{
			// Extract a completed error before consuming either image; C++ tests expected::has_value and reads its error.
			if let Err(error) = result
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err(error.clone());
			}
		}
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if self.images.iter().all(Option::is_some)
		{
			// Wrap a successful result without throwing; C++ expected stores its value.
			Ok(Some([
				// Access an image-completion slot under exclusive synchronization; C++ locks its mutex before accessing the slot.
				self.images[0].take().unwrap()?,
				// Access an image-completion slot under exclusive synchronization; C++ locks its mutex before accessing the slot.
				self.images[1].take().unwrap()?,
			]))
		}
		// Handle the complementary case; C++ uses the same else branch.
		else
		{
			// Report that construction is still pending; C++ optional remains empty without signaling failure.
			Ok(None)
		}
	}
}

// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
#[cfg(test)]
// Group tests in a nested module; C++ puts test helpers in a namespace.
mod tests
{
	// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
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
	// Register the following function with Rust's test runner; C++ needs a test framework registration.
	#[test]
	// Exercise normal, clamped, invalid, pending, failed, and completed loader states; C++ would use test assertions.
	fn tiers_and_loading()
	{
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(16384, false, None).unwrap(), 8);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(16384, true, None).unwrap(), 2);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(4096, false, Some("8")).unwrap(), 4);
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(select(8192, false, Some("2")).unwrap(), 2);
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(select(1024, true, None).is_err());
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(select(8192, false, Some("bad")).is_err());
		// Create a loader fixture with neither image complete; C++ initializes a local aggregate.
		let mut s = Loading {
			// Two optional completion results, albedo first and normals second; C++ array<optional<expected<Image,string>>,2>.
			images: [None, None],
			// An optional terminal loading/upload diagnostic; C++ optional<string>.
			error: None,
		};
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(s.take().unwrap().is_none());
		// Simulate completion of the first image; C++ assigns an expected<Image,string> into an optional slot.
		s.images[0] = Some(Ok(RgbImage::new(2, 1)));
		// Simulate an HTTP/disk/decoder failure for the second image; C++ stores unexpected<string>.
		s.images[1] = Some(Err("missing file".into()));
		// Assert exact equality for the expected invariant; C++ assert compares the same values but Rust also prints both.
		assert_eq!(s.take().unwrap_err(), "missing file");
		// Replace the failure with a successful completion and verify both images can be consumed.
		s.images[1] = Some(Ok(RgbImage::new(2, 1)));
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
		assert!(s.take().unwrap().is_some());
		// Panic if this geometric or state invariant is false; C++ assert is the nearest equivalent, but Rust tests keep assertions enabled.
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
		// Resolve the native asset directory once in this worker; C++ filesystem::path owns its path.
		let directory = std::env::var_os("MOON_ASSET_DIR")
			// Transform only a present/successful value; C++ applies a callable after checking optional/expected.
			.map(std::path::PathBuf::from)
			// Compute the fallback only when absent; C++ checks optional before calling a fallback lambda.
			.unwrap_or_else(|| {
				// Try assets/globe beside the executable before the development fallback; C++ optional<path>.
				let beside = std::env::current_exe()
					// Discard a platform error and retain only success as an Option; C++ converts expected to optional.
					.ok()
					// Chain a fallible/optional operation without nesting wrappers; C++ checks then invokes the next callable.
					.and_then(|p| p.parent().map(|p| p.join("assets/globe")));
				// Use the executable-relative directory only if it exists; C++ filesystem::is_directory guards a path.
				beside.filter(|p| p.is_dir()).unwrap_or_else(|| {
					// Fall back to the compile-time crate directory for workspace runs; C++ a build-defined source path is similar.
					std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
				})
			});
		// Complete this asset exactly once with bytes or a diagnostic; C++ invokes the captured completion callable.
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
		// Resolve DOM construction failures through the ordinary loader diagnostic; C++ expected-returning factory.
		let result = match web_sys::HtmlImageElement::new()
		{
			Ok(image) =>
			{
				// Same-origin assets need no cross-origin permission or JS byte copies; C++ URL assignment.
				image.set_src(&format!("assets/globe/{filename}"));
				// Await the browser's own decoder and retain its native upload source; C++ future continuation.
				wasm_bindgen_futures::JsFuture::from(image.decode())
					.await
					.map(|_| send_wrapper::SendWrapper::new(image))
					.map_err(|e| format!("{filename}: {e:?}"))
			}
			Err(e) => Err(format!("{filename}: {e:?}")),
		};
		// Publish a complete image or a genuine error, never a half-decoded buffer; C++ synchronized slot.
		state.lock().images[index] = Some(result);
		// Background completion must repaint even when the first Moon is already visible; C++ event-loop wake.
		if let Some(ctx) = ctx
		{
			ctx.request_repaint();
		}
	});
}
