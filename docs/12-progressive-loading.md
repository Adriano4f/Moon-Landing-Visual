# Progressive Moon loading

## 1. What changed

The browser first displays a 512 by 256 texture pair. Native startup retains the 2k pair. Only after a real, lit draw do the selected site's JSON and optional higher-quality textures start. Desktop browsers default to 4k, phones to 2k, and native to 8k. View / camera contains a Texture detail selector with its own question-mark explanation. `?moon_tier=8` or native `MOON_TIER=8` explicitly requests 8k; hardware texture-size limits still apply.

The preview uses an embedded 56-byte, one-record ephemeris for 2027-01-01 00:00 UTC, derived reproducibly from the validated complete series. Its date is shown explicitly. It supplies real Moon-fixed Sun and Earth directions, not terrain visibility or invented horizon results. The selected track permanently replaces this initial state when ready. A later missing track cannot revive the initial epoch. The complete eight-year ephemeris and DEMs remain absent from startup traffic.

`globe/assets/preview.py` averages the existing 2k albedo in linear light before JPEG encoding. It averages object-space normal vectors and renormalizes them before lossless WebP encoding. The files contain 29,555 and 163,536 bytes, respectively, totaling 193,091 bytes instead of the first 2k pair's 2,975,280 bytes. This is a coarse first picture, not additional terrain information. Selected 2k, 4k and 8k assets are unchanged.

The browser's `web-release` Cargo profile uses size optimization, LTO, one codegen unit and aborting panics; `lunar_core` retains optimization level 3. Trunk runs `wasm-opt -Oz`, stripping names and producer metadata only after wasm-bindgen has consumed target-feature metadata. Native release settings are unchanged. Browser font subsets retain Latin text, punctuation, geometric symbols and interface metrics; native fonts remain complete. See `app/assets/fonts/README.md` for source fonts, licenses and deterministic regeneration.

Native PNG/JPEG decoding remains on worker threads. Browser image requests instead use `HtmlImageElement.decode()`: the browser performs decoding, not the Rust image decoder in the WASM event callback. `glow` 0.17 uploads the decoded DOM image directly. `SendWrapper` checks creation-thread access without an unsafe Send implementation; no worker may dereference these DOM images.

The first pair initializes the renderer. An upgrade retains its shader, geometry and old textures, uploads at most one replacement image per frame, and exposes the new pair only after both uploads succeed. HTTP, decoding and GL failures retain the last successful Moon and display a diagnostic. A failed initial pair requires reloading the page, but releases the data-loading gate so the independent terrain graph and scientific results remain usable. Camera, time and scientific data are not reset by an upgrade.

## 2. Lossless browser normals

Run `python3 globe/assets/web_normals.py` after preparing the native PNGs. Pillow with WebP support is required. The generated files preserve every RGB8 channel exactly; they introduce no additional normal-vector quantization, slope error or photometric error. Native distributions still use PNG. Trunk deploys WebP normals instead of the redundant PNG copies.

| Tier | PNG bytes | WebP bytes | Reduction |
|:--|--:|--:|--:|
| 2k | 2,703,533 | 2,250,260 | 16.8% |
| 4k | 10,637,334 | 8,878,268 | 16.5% |
| 8k | 36,756,786 | 29,849,182 | 18.8% |

All three lossless round trips changed zero channels, so the additional angular error is exactly zero. A separate Chrome canvas comparison of the complete 2k PNG/WebP pair also found zero differences across 8,388,608 RGBA channels. Albedo JPEGs and the original native PNGs are unchanged. Browser color-space conversion is disabled during upload; normals remain linear data and albedo still uses sRGB GPU storage.

## 3. Historical 2k-first measurements

The earlier comparison used the same local HTTP server, 1440 by 1000 viewport, device scale 1 and software WebGL before and after. Caches were disabled. The throttled connection used 1,600,000 download bytes/s, 750,000 upload bytes/s and 40 ms latency. This server did not gzip WASM or scientific JSON. Its 12.8 Mbps connection must not be confused with the stricter 4 Mbps case below.

Readiness means a real lit draw, followed by two animation frames, not merely an image download. Before the change, first and final quality were both 8k. Afterward, first quality is 2k and the default final quality is 4k, so the final-quality columns do not compare equal resolution. Bytes include JS, WASM, the selected site's full scientific JSON and all requested textures, including the initial pair.

| Network | Before first/final | After first 2k | After final 4k | Before/after total bytes |
|:--|--:|--:|--:|--:|
| Unthrottled | 4.41 s | 1.87 s | 4.13 s | 54,821,519 / 22,003,623 |
| 12.8 Mbps, 40 ms | 37.89 s | 7.82 s | 17.26 s | 54,821,519 / 22,003,623 |

The first-Moon target of about one to two seconds was met only without throttling. Uncompressed WASM, scientific JSON and the first texture pair exceeded 10 MB together. Main-thread WASM image decoding was eliminated, but GPU upload/mipmap generation remained synchronous: the largest animation-frame gap was approximately 0.70 s on software WebGL.

### 3.1. Measured critical path and current preview

Cold HTTP-cache Chrome measurements use a 1440 by 1000 viewport, device scale 2 and software WebGL. Fast 4G is explicitly 500,000 download bytes/s, 750,000 upload bytes/s and 20 ms latency. A separate 12.8 Mbps, 40 ms case preserves the earlier connection for comparison. Readiness is the real lit draw followed by two animation frames. Total bytes below sum Resource Timing transfer sizes.

The deployed 2k-first baseline requested a 1,787,505-byte encoded WASM module, then the 616,257-byte encoded selected JSON and 2,975,280-byte first texture pair together. No ephemeris or DEM request occurred. The 4k pair added 11,673,441 bytes. Pages served compressed WASM and JSON, as their encoded and decoded lengths differed: 1,787,505 / 4,577,844 and 616,257 / 2,718,617 bytes.

| Transport | Network | First Moon, s | Full 4k, s | Total bytes |
|:--|:--|--:|--:|--:|
| Deployed 2k-first baseline | Unthrottled | 1.919 | 5.020 | 17,068,424 |
| Deployed 2k-first baseline | Fast 4G | 11.800 | 37.222 | 17,068,424 |
| Deployed 2k-first baseline | 12.8 Mbps, 40 ms | 4.942 | 14.347 | 17,068,424 |
| Current local gzip transport | Unthrottled | 0.944 | 3.010 | 13,786,859 |
| Current local gzip transport | Fast 4G | 3.724 | 30.099 | 13,786,859 |
| Current local gzip transport | 12.8 Mbps, 40 ms | 1.742 | 11.330 | 13,786,859 |

The current rows use a local HTTP server with gzip level 6 for JS, WASM and JSON, not a second Pages deployment. They characterize the new critical path but are not a same-server before/after experiment. The final WASM is 3,823,377 decoded bytes and 1,292,561 gzip bytes. Startup site requests begin after the first real draw; complete ephemeris and DEM requests remain zero.

The strict first-Moon budget below 3 s is not met in this software-WebGL run. JS, WASM and preview textures alone nearly exhaust its 1.5 MB three-second transfer budget, before initialization and rendering. GPU uploads and mipmap generation also remain synchronous: maximum measured frame gaps were 583.3 to 666.6 ms. These results establish faster startup, not stall-free rendering or attainment of the stricter target.

## 4. Checks and limitations

Browser checks cover preview-before-site-data ordering, desktop 4k and Pixel 7 2k targets, browser-native decoding, real illuminated draws, and missing-upgrade handling. Pixel 7 means emulation, not physical hardware. The 8k option remains a large explicit download and requires considerable decoder/GPU memory. Progressive replacement temporarily retains both old and new texture pairs. A texture-size limit is not a memory guarantee.

Scientific terrain silhouettes, Horizons time samples, visibility predicates, light directions and site selection are unchanged. [South-polar detail](15-polar-detail.md) now loads independently after a real overview draw and only when zoomed into a visible south pole; it is not part of the startup critical path.

## 5. Annotated loading source

### 5.1. globe/src/assets.rs

```rust
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
			if mobile
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
```

### 5.2. globe/assets/web_normals.py

```python
"""Create browser-native, lossless normal textures; never quantize directional data."""
from pathlib import Path
import json
from PIL import Image

ROOT = Path(__file__).resolve().parent


def main():
    """Keep native PNGs and emit exactly equivalent browser WebP pixels."""
    results = []
    for tier in (2, 4, 8):
        source = ROOT / f"normal_{tier}k.png"
        target = ROOT / f"normal_{tier}k.webp"
        with Image.open(source) as image:
            rgb = image.convert("RGB")
            rgb.save(target, format="WEBP", lossless=True, quality=100, method=4)
            with Image.open(target) as decoded:
                if decoded.convert("RGB").tobytes() != rgb.tobytes():
                    raise ValueError(f"Normal pixels changed: {target}")
        results.append(dict(tier=tier, png_bytes=source.stat().st_size,
                            webp_bytes=target.stat().st_size,
                            changed_channels=0, angular_error_deg=0))
        print(json.dumps(results[-1]), flush=True)
    print("All normal textures are pixel-identical; angular error 0 degrees.")


if __name__ == "__main__":
    main()
```

### 5.3. app/src/fonts.rs

```rust
use eframe::egui::{Context, FontData, FontDefinitions, FontFamily, FontTweak};

// Preserve interface glyph metrics without embedding unused scripts; C++ font registry setup.
pub(super) fn install(ctx: &Context)
{
	let mut fonts = FontDefinitions::empty();
	for (name, bytes, scale) in [
		(
			"body",
			include_bytes!("../assets/fonts/karaya-ui.ttf").as_slice(),
			1.0,
		),
		(
			"mono",
			include_bytes!("../assets/fonts/karaya-mono.ttf").as_slice(),
			1.0,
		),
		(
			"egui-icons",
			include_bytes!("../assets/fonts/egui-icons.ttf").as_slice(),
			0.9,
		),
	]
	{
		fonts.font_data.insert(
			name.into(),
			FontData::from_static(bytes)
				.tweak(FontTweak {
					scale,
					..Default::default()
				})
				.into(),
		);
	}
	fonts.families.insert(
		FontFamily::Proportional,
		vec!["body".into(), "egui-icons".into(), "mono".into()],
	);
	fonts.families.insert(
		FontFamily::Monospace,
		vec!["mono".into(), "body".into(), "egui-icons".into()],
	);
	ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests
{
	use super::*;
	use eframe::egui::{FontId, RawInput};

	#[test]
	fn interface_symbols_empty_and_unknown_glyphs()
	{
		let ctx = Context::default();
		install(&ctx);
		let mut output = ctx.run_ui(RawInput::default(), |ui| {
			ui.fonts_mut(|fonts| {
				for family in [FontFamily::Proportional, FontFamily::Monospace]
				{
					let id = FontId::new(15.0, family);
					assert!(fonts.has_glyphs(&id, "Karaya Gläser 0° 50% ● ■ × ← → … − ·"));
					assert!(fonts.has_glyphs(&id, ""));
					assert!(!fonts.has_glyph(&id, '\u{10ffff}'));
				}
			});
		});
		output.textures_delta.clear();
	}
}
```

### 5.4. app/src/startup.rs

```rust
use lunar_core::ephemeris::Ephemeris;

// Decode one validated initial epoch without fetching the complete eight-year series; C++ checked tiny record.
pub(super) fn directions(bytes: &[u8]) -> Result<[[f32; 3]; 2], String>
{
	let initial = Ephemeris::decode(bytes)?;
	if initial.start_utc != 1798761600 || initial.step_s != 3600 || initial.samples.len() != 1
	{
		return Err("Initial sky must contain only 2027-01-01 00:00 UTC".into());
	}
	Ok(initial.samples[0].map(|body| body.direction))
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn initial_record_matches_complete_series_and_rejects_bad_data()
	{
		let bytes = include_bytes!("../assets/data/moon-ephemeris.initial.bin");
		assert_eq!(bytes.len(), 56);
		let full = Ephemeris::decode(include_bytes!("../assets/data/moon-ephemeris.bin")).unwrap();
		assert_eq!(
			directions(bytes).unwrap(),
			full.samples[0].map(|body| body.direction)
		);
		assert!(directions(&[]).is_err());
		assert!(directions(include_bytes!("../assets/data/moon-ephemeris.bin")).is_err());
		let mut wrong_epoch = *bytes;
		wrong_epoch[8..16].copy_from_slice(&(full.start_utc + 3600).to_le_bytes());
		assert!(directions(&wrong_epoch).is_err());
		let mut corrupt = *bytes;
		corrupt[24..28].copy_from_slice(&f32::NAN.to_le_bytes());
		assert!(directions(&corrupt).is_err());
	}
}
```

### 5.5. globe/assets/preview.py

```python
"""Derive a 512 by 256 first frame from the existing 2k scientific-display assets."""
from pathlib import Path
import numpy as np
from PIL import Image


def average(image):
    values = np.asarray(image, dtype=np.float64) / 255.0
    if values.shape != (1024, 2048, 3) or not np.isfinite(values).all():
        raise ValueError('Expected a finite 2k RGB image')
    return values.reshape(256, 4, 512, 4, 3).mean(axis=(1, 3))


def normal_preview(image):
    vectors = average(image) * 2.0 - 1.0
    lengths = np.linalg.norm(vectors, axis=2, keepdims=True)
    if np.any(lengths < 1e-6):
        raise ValueError('Degenerate mean normal')
    return Image.fromarray(np.rint((vectors / lengths + 1.0) * 127.5).astype(np.uint8))


def main():
    directory = Path(__file__).parent
    source = np.asarray(Image.open(directory/'albedo_2k.jpg'), dtype=np.float64)/255.0
    linear = np.where(source <= 0.04045, source/12.92, ((source+0.055)/1.055)**2.4)
    mean = linear.reshape(256, 4, 512, 4, 3).mean(axis=(1, 3))
    srgb = np.where(mean <= 0.0031308, mean*12.92, 1.055*mean**(1/2.4)-0.055)
    Image.fromarray(np.rint(srgb*255).astype(np.uint8)).save(directory/'albedo_preview.jpg', quality=87, subsampling=0, optimize=True)
    normal_preview(Image.open(directory/'normal_2k.png')).save(directory/'normal_preview.webp', lossless=True, method=6)
    for name in ['albedo_preview.jpg', 'normal_preview.webp']:
        print(name, (directory/name).stat().st_size)


if __name__ == '__main__':
    main()
```

### 5.6. app/assets/fonts/subset.py

```python
"""Build small, renamed interface fonts from epaint_default_fonts 0.37 originals."""
from pathlib import Path
import argparse
import shutil
from fontTools import subset
from fontTools.ttLib import TTFont


def build(source, target, name):
    font = TTFont(source, recalcTimestamp=False)
    options = subset.Options()
    options.recalc_timestamp = False
    options.drop_tables += ['TTFA']
    options.name_IDs = ['*']
    options.name_languages = ['*']
    worker = subset.Subsetter(options=options)
    worker.populate(unicodes=list(range(32, 256)) + list(range(0x2000, 0x2070)) + [ord(c) for c in '●■←→↑↓↔↕▶▼◀▲−'])
    worker.subset(font)
    for record in font['name'].names:
        if record.nameID in (1, 2, 3, 4, 6, 16, 17):
            value = 'Regular' if record.nameID in (2, 17) else name
            record.string = value.encode(record.getEncoding())
    font.save(target)
    print(target.name, target.stat().st_size)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path, help='epaint_default_fonts 0.37 fonts directory')
    source = parser.parse_args().source
    output = Path(__file__).parent
    build(source/'Ubuntu-Light.ttf', output/'karaya-ui.ttf', 'KarayaUI')
    build(source/'Hack-Regular.ttf', output/'karaya-mono.ttf', 'KarayaMono')
    for filename in ['egui-icons.ttf', 'egui-icons.txt', 'UFL.txt', 'Hack-Regular.txt']:
        shutil.copyfile(source/filename, output/filename)


if __name__ == '__main__':
    main()
```

## 6. API index

`preferences` and `select` choose the requested final tier. `load` schedules native workers or browser-native image decoding. `Loading::take` distinguishes pending, failed and completed pairs. `Renderer::new` owns initial GL construction. `begin_upgrade` preserves visible textures, `advance_upgrade` stages replacement uploads, and `upload_browser_texture` passes the browser image to GL. `Globe::camera_controls` exposes the requested detail and the actually displayed tier.

`first_tier` and `label` distinguish the private preview from selectable final tiers. `startup::directions` validates the one-record epoch. `Globe::set_startup_light`, `light_sources` and `has_visible_moon` maintain the one-shot initial light and post-draw loading gate. `fonts::install` registers browser subsets; `subset.py::build` preserves glyph data while renaming the fonts. `preview.py::average` and `normal_preview` derive coarse vectors. `prep::ephemeris::initial_sample` uses the existing checked binary encoder.
