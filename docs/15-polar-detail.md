# South-polar visualization detail

> **Historical fallback atlas.** This guide's projection, source provenance,
> measurements and annotated code describe the original south-polar fallback.
> Current main `5e4d5b7` also streams north/south WAC and optional polar NAC,
> publishes compatible global/polar resources atomically and uses DEM detail
> for Realistic normals/shadows. See [20](20-terrain-shadows.md),
> [21](21-atomic-loading-and-flicker.md) and [23](23-deep-zoom-windows.md).
> The 1k/2k tables below describe fallback spacing, not maximum zoom resolution.

## Data and provenance

The south-polar visualization combines the official LROC WAC south-polar mosaic with normals derived from the unrendered LOLA 30 m south-polar DEM. These are rendering inputs, not the detailed PGDA scientific horizons. Site statistics, ephemerides and the terrain silhouette are unchanged.

- Imagery: `LRO_WAC_Mosaic_SPole60_100m_v02`, credited to the LROC Team. Its [official metadata](https://trek.nasa.gov/moon/TrekWS/rest/cat/metadata/fgdc/html?label=LRO_WAC_Mosaic_SPole60_100m_v02) lists access constraints and use constraints as None, and Arizona State University as the contact.
- [WMTS capabilities](https://trek.nasa.gov/tiles/Moon/SP/LRO_WAC_Mosaic_SPole60_100m_v02/1.0.0/WMTSCapabilities.xml): style `default`, matrix set `default028mm`, levels 0 through 5, 256 by 256 PNG tiles, top-left projected corner (-1,095,930 m, 1,095,930 m). The capabilities CRS identifier is the unresolved `urn:ogc:def:crs:EPSG::0`, not a valid authoritative EPSG code; the projection must be identified from the product's south-polar service metadata rather than guessed from that identifier. Tile requests use `https://trek.nasa.gov/tiles/Moon/SP/LRO_WAC_Mosaic_SPole60_100m_v02/1.0.0/default/default028mm/{level}/{row}/{column}.png`.
- Normals: [LOLA south-polar 30 m ImageServer](https://trek.nasa.gov/moon/trekarcgis/rest/services/LRO_LOLA_DEM_SPole75_30mp_v04/ImageServer?f=pjson), exported as uncompressed, single-band F32 GeoTIFF with raster function None. `polar_normals.py` records the source hash, coordinate convention and exported spacing in `polar_normals.meta.json`. The source TIFF remains outside the deployed repository.

Capabilities and sampled imagery/elevation responses were checked with an Origin header and returned `Access-Control-Allow-Origin: *`. The Trek API documentation supports direct WMTS clients. This does not establish an unlimited quota or availability guarantee. Three requests run concurrently; there is no automatic retry loop.

## Projection and resolution

The [WAC ImageServer's spatial-reference WKT](https://trek.nasa.gov/moon/trekarcgis/rest/services/LRO_WAC_Mosaic_SPole60_100m_v02/ImageServer?f=pjson) explicitly specifies `Stereographic_South_Pole`, standard parallel -90, central meridian zero, zero false offsets and spherical radius 1,737,400 m. This resolves the WMTS document's invalid CRS identifier. The spherical south-polar stereographic projection has radius $R=1{,}737{,}400\ \mathrm{m}$, central longitude zero, true scale at the south pole and no false offsets. For Moon-fixed renderer direction $(p_x,p_y,p_z)$, projected east/north coordinates are

$$X=\frac{2Rp_x}{1-p_y},\qquad Y=\frac{2Rp_z}{1-p_y}.$$

The square atlas has half-width $E=547{,}965\ \mathrm{m}$ and texture coordinates

$$u=\frac12+\frac{X}{2E},\qquad v=\frac12-\frac{Y}{2E}.$$

Rows increase southward in the image, without a browser-only vertical flip. Longitude zero is toward positive projected north; positive longitude rotates toward projected east. The shader blends through latitudes 75 to 77 degrees south and completely replaces the schematic southern cap farther south when both imagery and normals are ready. The northern cap retains its existing fallback.

| Profile | WMTS level | Rows and columns | Mosaic and normal side | Spacing, m |
|:--|--:|:--|--:|--:|
| 2k overview quality, default phone | 3 | 2 through 5 | 1,024 | 1,070.244140625 |
| Higher overview quality, default desktop | 4 | 4 through 11 | 2,048 | 535.1220703125 |

The capabilities document reports a nominal matrix twice as wide as high; this implementation uses the central square of the actual polar extent, not an assumed geographic longitude grid. The selected square comfortably contains the 75 degree south circle. Resolution is sufficient for the existing 8 times overview zoom, not a satellite deep-zoom pyramid.

The normal bake reconstructs each pixel-center latitude and longitude from stereographic coordinates, then forms $\mathbf P=(R+h)\mathbf u$. Centered row/column differences produce a cross product, oriented outward and normalized. Missing or invalid heights use the radial geometric normal. RGB8 encoding rounds $127.5(\mathbf n+1)$; normals are renormalized after texture filtering. Phone heights are 2 by 2 averages of the exported desktop grid. Neither profile retains the original 30 m terrain resolution.

## Loading and display interpretation

No polar assets join initial loading. A real overview Moon must already have been drawn, zoom must be at least 2, and the south pole must be visible and near the viewport. Requests pause when that condition no longer holds. The application owns one atlas, one normal texture and at most four in-flight or completed images. One image uploads per frame; the previous complete material remains visible until all 17 or 65 jobs succeed. Failed HTTP, decoding or GL operations retain that material and show an unavailable caption. Completed polar textures are reused without further requests during the session.

The browser uses native image decoding and CORS-enabled DOM image uploads. Native builds use background HTTP/image decoding. At most one polar image uploads per frame, independently of existing global-tier uploads. Normal assets are lossless WebP in the browser and PNG natively:

| Profile | WebP bytes | PNG bytes |
|:--|--:|--:|
| 1k normals | 910,680 | 1,078,757 |
| 2k normals | 3,243,240 | 3,927,182 |

Two complete atlas textures are retained, with nominal RGB8 normals plus RGBA8 imagery and mipmaps requiring approximately 9.79 or 39.15 MB. Driver storage and browser-managed caches can differ. The selected global overview textures remain separately owned.

The historical photographic fallback contains recorded shadows, not this epoch's illumination and not calibrated albedo. Its bounded contrast divides linear values by 0.2, clamps to [0.65, 1.35] and mixes with neutral contrast at weight 0.25 when streamed imagery is not taking precedence. Current Fast uses photographic contrast with cheap geometric shading; Realistic excludes it and instead uses DEM normals and dynamic cast shadows. It would be incorrect to describe current Realistic as Hapke multiplied by this recorded-shadow mosaic. Neither path predicts thermal conditions or landing safety.

## Numeric elevation finding

Trek's similarly named DEM WMTS is an 8-bit display image, not numeric elevation. A sampled tile was grayscale plus alpha, with values 34 through 232. It must not be interpreted as meters. By contrast, the ImageServer export with `pixelType=F32`, `format=tiff`, `compression=none` and `renderingRule={"rasterFunction":"None"}` returned genuine floating-point heights. A 16 by 16 sample around the south pole ranged from 775.369 to 1,391.334 m and included correct lunar stereographic GeoKeys, pixel-is-area georeferencing and 125 m pixel spacing. This particular TIFF uses 128 by 128 tiles rather than strips. Runtime scientific terrain loading must handle that distinction and validate global/finer coverage separately.

## Checks and observed loading

Unit tests cover atlas geometry and URLs, device tiers, overview/hidden/offscreen request gates, stopped complete/error states, flat-sphere normal orientation, nodata and invalid normal-bake inputs. Browser checks exercise the actual CORS tile path. On local cold browser contexts without network throttling, atlas completion followed the first Trek request by approximately 25.721 s on desktop and 7.087 s on Pixel 7. These are lazy polar-detail timings, not first-Moon startup measurements, and do not establish a service latency guarantee.

Maximum-zoom before/after screenshots are `/home/ubuntu/plan/polar/before-desktop.png`, `after-desktop.png`, `before-pixel7.png` and `after-pixel7.png`. The phone captures use Pixel 7 emulation at 412 by 839 CSS pixels and device scale 2. The camera was positioned with existing desktop controls before resizing the phone context, preserving the real phone 2k asset profile.

## 5. API index

- `abs`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `active_texture`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `advance`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `all`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `and_then`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `arange`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `Arc::clone`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `Arc::new`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `arctan`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `arctan2`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `as_mut`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `as_raw`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `as_ref`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `asarray`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `assertEqual`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `assertGreater`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `assertLess`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `assertRaises`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `assertTrue`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `astype`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `Atlas::new`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `bind_polar`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `bind_texture`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `clear`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `clip`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `clone`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `cos`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `count`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `crate::assets::read_native`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `create_texture`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `cross`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `decode`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `delete_texture`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `destroy`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `dumps`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `egui::Context::default`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `ehttp::fetch`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `ehttp::Request::get`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `empty`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `exists`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `extents_tiers_offsets_and_urls`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `failed_and_complete_states_do_not_start_requests`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `fetch::image`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `filter`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `finish`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `fromarray`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `full`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `gate_rejects_overview_hidden_and_offscreen_poles`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `generate_mipmap`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `get`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `get_error`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `get_or_insert_with`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `get_uniform_location`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `glow::PixelUnpackData::Slice`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `gradient`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `height`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `hexdigest`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `hypot`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `image`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `image::load_from_memory`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `into`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `is_empty`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `is_err`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `is_none`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `is_some`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `is_some_and`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `isfinite`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `len`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `lock`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `main`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `map`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `map_err`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `map_or`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `mean`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `meshgrid`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `Mutex::new`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `new`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `norm`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `normals`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `offset`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `open`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `pixel_store_i32`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `pop`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `pop_front`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `pow`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `powi`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `prepare`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `project`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `publish_polar`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `push`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `push_back`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `raise_for_status`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `read_bytes`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `request_repaint`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `reshape`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `round`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `save`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `Self::filter`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `send_wrapper::SendWrapper::new`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `set_cross_origin`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `set_src`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `sha256`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `sin`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `Spec::new`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `stack`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `starts_with`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `stat`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `State::default`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `sum`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `test_bad_dimensions_and_extent`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `test_flat_sphere_pole_and_orientation`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `test_missing_cells_use_radial_not_invalid_normals`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `tex_image_2d`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `tex_parameter_i32`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `tex_sub_image_2d`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `tex_sub_image_2d_with_html_image`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `tile`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `to_rgb8`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `to_string`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `tolist`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).
- `uniform_1_f32`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `uniform_1_i32`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `unwrap`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `unwrap_or`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `update_polar`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `upload_browser_texture`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `upload_polar_normal`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `upload_texture`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `url`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `Vec::new`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `wanted`: [globe/src/polar.rs](#61-globesrcpolarrs).
- `wasm_bindgen_futures::JsFuture::from`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `wasm_bindgen_futures::spawn_local`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `web_sys::console::log_1`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `web_sys::HtmlImageElement::new`: [globe/src/polar/fetch.rs](#62-globesrcpolarfetchrs).
- `where`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `width`: [globe/src/renderer/polar.rs](#63-globesrcrendererpolarrs).
- `with_suffix`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `write_bytes`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `write_text`: [globe/assets/polar_normals.py](#64-globeassetspolarnormalspy).
- `zeros`: [globe/assets/test_polar_normals.py](#65-globeassetstestpolarnormalspy).

## 6. Complete implementation sources

### 6.1. globe/src/polar.rs

```rust
//! Bounded south-polar WMTS atlas, independent of scientific terrain and startup.
mod fetch;

use crate::{assets, math};
use egui::mutex::Mutex;
use std::{collections::VecDeque, sync::Arc};

pub(super) const EXTENT: f32 = 547965.0;
const WAC: &str = "https://trek.nasa.gov/tiles/Moon/SP/LRO_WAC_Mosaic_SPole60_100m_v02/1.0.0/default/default028mm";

#[derive(Clone, Copy, Debug)]
pub(super) struct Spec
{
	pub side: u32,
	pub level: u32,
}

impl Spec
{
	pub fn new(phone: bool) -> Self
	{
		Self {
			side: if phone { 1024 } else { 2048 },
			level: if phone { 3 } else { 4 },
		}
	}

	pub fn count(self) -> usize
	{
		1 + (self.side / 256).pow(2) as usize
	}

	pub fn offset(self, job: usize) -> (u32, u32)
	{
		let n = self.side / 256;
		(((job - 1) as u32 % n) * 256, ((job - 1) as u32 / n) * 256)
	}

	fn url(self, job: usize) -> String
	{
		if job == 0
		{
			let ext = if cfg!(target_arch = "wasm32")
			{
				"webp"
			}
			else
			{
				"png"
			};
			return format!("polar_normal_{}k.{ext}", self.side / 1024);
		}
		let (x, y) = self.offset(job);
		let first = self.side / 512;
		format!(
			"{WAC}/{}/{}/{}.png",
			self.level,
			first + y / 256,
			first + x / 256
		)
	}
}

#[derive(Default)]
pub(super) struct State
{
	pub spec: Option<Spec>,
	pub ready: bool,
	pub error: Option<String>,
	pub pending: VecDeque<(usize, assets::TextureImage)>,
	pub uploaded: usize,
	next: usize,
	in_flight: usize,
}

pub(super) type Shared = Arc<Mutex<State>>;

pub(super) fn wanted(view: math::View) -> bool
{
	view.zoom >= 2.0
		&& view
			.project([0.0, -1.0, 0.0], true)
			.is_some_and(|p| p[0].abs() < 1.2 && p[1].abs() < 1.2)
}

pub(super) fn advance(shared: &Shared, phone: bool, ctx: &egui::Context)
{
	let jobs = {
		let mut state = shared.lock();
		if state.error.is_some() || state.ready
		{
			return;
		}
		let spec = *state.spec.get_or_insert_with(|| Spec::new(phone));
		let mut jobs = Vec::new();
		while state.in_flight < 3
			&& state.pending.len() + state.in_flight < 4
			&& state.next < spec.count()
		{
			jobs.push((state.next, spec.url(state.next)));
			state.next += 1;
			state.in_flight += 1;
		}
		jobs
	};
	for (index, url) in jobs
	{
		let shared = Arc::clone(shared);
		let ctx = ctx.clone();
		fetch::image(url, move |result| {
			let mut state = shared.lock();
			state.in_flight -= 1;
			if state.error.is_none()
			{
				match result
				{
					Ok(image) => state.pending.push_back((index, image)),
					Err(error) =>
					{
						state.error = Some(error);
						state.pending.clear();
					}
				}
			}
			ctx.request_repaint();
		});
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn extents_tiers_offsets_and_urls()
	{
		for phone in [false, true]
		{
			let spec = Spec::new(phone);
			assert_eq!(spec.count(), if phone { 17 } else { 65 });
			assert_eq!(spec.offset(1), (0, 0));
			assert_eq!(
				spec.offset(spec.count() - 1),
				(spec.side - 256, spec.side - 256)
			);
			assert!(spec.url(1).starts_with(WAC));
			let pixel_size = 8561.953125_f64 / 2_f64.powi(spec.level as i32);
			assert!((pixel_size * spec.side as f64 / 2.0 - EXTENT as f64).abs() < 1e-6);
		}
		let state = State::default();
		assert!(!state.ready && state.error.is_none() && state.pending.is_empty());
	}

	#[test]
	fn gate_rejects_overview_hidden_and_offscreen_poles()
	{
		let mut view = math::View {
			yaw: 0.0,
			tilt: 0.0,
			zoom: 1.0,
			aspect: 1.0,
			pixels: [500.0; 2],
			scale: 1.0,
		};
		assert!(!wanted(view));
		view.zoom = 8.0;
		assert!(!wanted(view));
		view.tilt = -std::f32::consts::FRAC_PI_2;
		assert!(wanted(view));
		view.tilt = -1.0;
		assert!(!wanted(view));
		view.tilt = std::f32::consts::FRAC_PI_2;
		assert!(!wanted(view));
	}

	#[test]
	fn failed_and_complete_states_do_not_start_requests()
	{
		let state = Arc::new(Mutex::new(State::default()));
		state.lock().error = Some("unavailable".into());
		advance(&state, false, &egui::Context::default());
		assert!(state.lock().spec.is_none());
		state.lock().error = None;
		state.lock().ready = true;
		advance(&state, false, &egui::Context::default());
		assert!(state.lock().spec.is_none());
	}
}
```

### 6.2. globe/src/polar/fetch.rs

```rust
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
```

### 6.3. globe/src/renderer/polar.rs

```rust
use crate::polar::{Shared, Spec};
use crate::{assets, glow, renderer::Renderer};
use glow::HasContext as _;

pub(crate) struct Atlas
{
	color: glow::Texture,
	normal: Option<glow::Texture>,
	spec: Spec,
}

impl Atlas
{
	fn new(gl: &glow::Context, spec: Spec) -> Result<Self, String>
	{
		// SAFETY: the renderer's deferred callback owns the current context.
		unsafe {
			let color = gl.create_texture()?;
			gl.active_texture(glow::TEXTURE2);
			gl.bind_texture(glow::TEXTURE_2D, Some(color));
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::SRGB8_ALPHA8 as i32,
				spec.side as i32,
				spec.side as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(None),
			);
			Self::filter(gl);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				gl.delete_texture(color);
				return Err(format!("Polar allocation GL {error}"));
			}
			Ok(Self {
				color,
				normal: None,
				spec,
			})
		}
	}

	fn filter(gl: &glow::Context)
	{
		// SAFETY: the caller has bound the atlas or its complete normal texture.
		unsafe {
			for parameter in [glow::TEXTURE_WRAP_S, glow::TEXTURE_WRAP_T]
			{
				gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::CLAMP_TO_EDGE as i32);
			}
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR as i32,
			);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as i32,
			);
		}
	}

	fn tile(
		&self,
		gl: &glow::Context,
		job: usize,
		image: &assets::TextureImage,
	) -> Result<(), String>
	{
		if job >= self.spec.count() || image.width() != 256 || image.height() != 256
		{
			return Err("Unexpected polar tile geometry".into());
		}
		let (x, y) = self.spec.offset(job);
		// SAFETY: dimensions and atlas bounds are validated before this current-context upload.
		unsafe {
			gl.active_texture(glow::TEXTURE2);
			gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
			#[cfg(not(target_arch = "wasm32"))]
			gl.tex_sub_image_2d(
				glow::TEXTURE_2D,
				0,
				x as i32,
				y as i32,
				256,
				256,
				glow::RGB,
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(image.as_raw())),
			);
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
				gl.tex_sub_image_2d_with_html_image(
					glow::TEXTURE_2D,
					0,
					x as i32,
					y as i32,
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
				return Err(format!("Polar tile GL {error}"));
			}
		}
		Ok(())
	}

	pub(crate) fn destroy(self, gl: &glow::Context)
	{
		// SAFETY: the owning renderer is destroyed with its context current.
		unsafe {
			gl.delete_texture(self.color);
			if let Some(n) = self.normal
			{
				gl.delete_texture(n);
			}
		}
	}

	fn finish(&self, gl: &glow::Context) -> Result<(), String>
	{
		// SAFETY: all level-zero pixels are uploaded before mipmaps become visible.
		unsafe {
			gl.active_texture(glow::TEXTURE2);
			gl.bind_texture(glow::TEXTURE_2D, Some(self.color));
			gl.generate_mipmap(glow::TEXTURE_2D);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
			);
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Polar mipmap GL {error}"));
			}
		}
		Ok(())
	}
}

impl Renderer
{
	pub(crate) fn update_polar(&mut self, gl: &glow::Context, shared: &Shared)
	{
		let (spec, work) = {
			let mut state = shared.lock();
			if state.error.is_some() || state.ready
			{
				return;
			}
			let Some(spec) = state.spec
			else
			{
				return;
			};
			(spec, state.pending.pop_front())
		};
		let result = (|| {
			if self.polar.is_none()
			{
				self.polar = Some(Atlas::new(gl, spec)?);
			}
			let Some((job, image)) = work
			else
			{
				return Ok(false);
			};
			if job == 0
			{
				self.upload_polar_normal(gl, image, spec)?;
			}
			else
			{
				self.polar.as_ref().unwrap().tile(gl, job, &image)?;
			}
			Ok(true)
		})();
		self.publish_polar(gl, shared, spec, result);
	}

	fn publish_polar(
		&self,
		gl: &glow::Context,
		shared: &Shared,
		spec: Spec,
		result: Result<bool, String>,
	)
	{
		let mut state = shared.lock();
		match result
		{
			Ok(true) =>
			{
				state.uploaded += 1;
				if state.uploaded == spec.count()
				{
					if let Err(error) = self.polar.as_ref().unwrap().finish(gl)
					{
						state.error = Some(error);
						return;
					}
					state.ready = true;
					#[cfg(target_arch = "wasm32")]
					web_sys::console::log_1(
						&format!("Polar detail visible: {}px", spec.side).into(),
					);
				}
			}
			Ok(false) =>
			{}
			Err(error) =>
			{
				state.error = Some(error);
				state.pending.clear();
			}
		}
	}

	fn upload_polar_normal(
		&mut self,
		gl: &glow::Context,
		image: assets::TextureImage,
		spec: Spec,
	) -> Result<(), String>
	{
		if image.width() != spec.side || image.height() != spec.side
		{
			return Err("Unexpected polar normal geometry".into());
		}
		let previous = self.textures.len();
		#[cfg(not(target_arch = "wasm32"))]
		let result = self.upload_texture(gl, image.width(), image.height(), image.as_raw(), false);
		#[cfg(target_arch = "wasm32")]
		let result = self.upload_browser_texture(gl, &image, false);
		let texture = if self.textures.len() > previous
		{
			self.textures.pop()
		}
		else
		{
			None
		};
		if result.is_err()
		{
			// SAFETY: discard only this failed upload's registered texture, not the visible pair.
			if let Some(t) = texture
			{
				unsafe {
					gl.delete_texture(t);
				}
			}
			return result;
		}
		self.polar.as_mut().unwrap().normal = texture;
		Ok(())
	}

	pub(crate) fn bind_polar(&self, gl: &glow::Context, program: glow::Program, ready: bool)
	{
		let color = self.polar.as_ref().map_or(self.textures[0], |p| p.color);
		let normal = self
			.polar
			.as_ref()
			.and_then(|p| p.normal)
			.unwrap_or(self.textures[1]);
		// SAFETY: all samplers have complete textures even before the optional atlas is ready.
		unsafe {
			for (unit, name, texture) in
				[(2, "u_polar_color", color), (3, "u_polar_normal", normal)]
			{
				gl.active_texture(glow::TEXTURE0 + unit);
				gl.bind_texture(glow::TEXTURE_2D, Some(texture));
				gl.uniform_1_i32(gl.get_uniform_location(program, name).as_ref(), unit as i32);
			}
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_polar_ready").as_ref(),
				if ready { 1.0 } else { 0.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_polar_extent").as_ref(),
				crate::polar::EXTENT,
			);
		}
	}
}
```

### 6.4. globe/assets/polar_normals.py

```python
"""Bake object-space normals from the unrendered LOLA south-polar height service."""
from pathlib import Path
from io import BytesIO
import json
import hashlib
import numpy as np
from PIL import Image
import requests

RADIUS = 1737400.0
EXTENT = 547965.0
SERVICE = 'https://trek.nasa.gov/moon/trekarcgis/rest/services/LRO_LOLA_DEM_SPole75_30mp_v04/ImageServer'


def normals(heights, extent=EXTENT):
    if heights.ndim != 2 or min(heights.shape) < 3 or not np.isfinite(extent) or extent <= 0:
        raise ValueError('Expected a height grid at least three pixels across and positive extent')
    height, width = heights.shape
    x, y = np.meshgrid((np.arange(width)+0.5)*2*extent/width-extent,
                       extent-(np.arange(height)+0.5)*2*extent/height)
    rho = np.hypot(x, y)
    latitude = -np.pi/2 + 2*np.arctan(rho/(2*RADIUS))
    longitude = np.arctan2(x, y)
    up = np.stack((np.cos(latitude)*np.sin(longitude), np.sin(latitude),
                   np.cos(latitude)*np.cos(longitude)), axis=-1)
    valid = np.isfinite(heights) & (np.abs(heights) < 15000)
    position = up*(RADIUS+np.where(valid, heights, 0))[..., None]
    dy, dx = np.gradient(position, axis=(0, 1))
    normal = np.cross(dy, dx)
    normal /= np.linalg.norm(normal, axis=-1)[..., None]
    normal *= np.where(np.sum(normal*up, axis=-1) < 0, -1, 1)[..., None]
    normal[~valid] = up[~valid]
    return np.round((normal+1)*127.5).clip(0, 255).astype(np.uint8)


def prepare(cache, output):
    if not cache.exists():
        params = dict(f='image', bbox=f'{-EXTENT},{-EXTENT},{EXTENT},{EXTENT}',
                      size='2048,2048', format='tiff', pixelType='F32', compression='none',
                      renderingRule='{"rasterFunction":"None"}', interpolation='RSP_BilinearInterpolation')
        response = requests.get(SERVICE+'/exportImage', params=params, timeout=180)
        response.raise_for_status()
        image = Image.open(BytesIO(response.content))
        if image.mode != 'F' or image.size != (2048, 2048):
            raise ValueError('Service did not return the requested float height grid')
        cache.write_bytes(response.content)
        cache.with_suffix('.json').write_text(json.dumps(dict(url=response.url, bytes=len(response.content)), indent=2))
    heights = np.asarray(Image.open(cache), dtype=np.float64)
    for tier, grid in [(2, heights), (1, heights.reshape(1024, 2, 1024, 2).mean(axis=(1, 3)))]:
        image = Image.fromarray(normals(grid))
        image.save(output/f'polar_normal_{tier}k.png', optimize=True)
        image.save(output/f'polar_normal_{tier}k.webp', lossless=True, method=6)
        print(tier, 'normal bytes', (output/f'polar_normal_{tier}k.webp').stat().st_size)
    metadata = dict(source=SERVICE, source_sha256=hashlib.sha256(cache.read_bytes()).hexdigest(),
                    radius_m=RADIUS, extent_m=EXTENT, projection='IAU2000:30120',
                    source_spacing_m=30, exported_spacing_m=2*EXTENT/2048,
                    pixel_centers=True, height_units='meters',
                    missing_height_policy='radial normal',
                    normal_encoding='RGB8 object-space unit vector; round((n+1)*127.5)')
    (output/'polar_normals.meta.json').write_text(json.dumps(metadata, indent=2)+'\n')


if __name__ == '__main__':
    import sys
    prepare(Path(sys.argv[1]), Path(__file__).parent)
```

### 6.5. globe/assets/test_polar_normals.py

```python
import unittest
import numpy as np
from polar_normals import normals


class PolarNormals(unittest.TestCase):
    def test_flat_sphere_pole_and_orientation(self):
        n = normals(np.zeros((33, 33))).astype(float)/127.5-1
        self.assertLess(n[16, 16, 1], -0.99)
        self.assertGreater(n[16, -1, 0], 0)
        self.assertGreater(n[0, 16, 2], 0)
        self.assertTrue(np.all(np.abs(np.linalg.norm(n, axis=-1)-1) < 0.007))

    def test_missing_cells_use_radial_not_invalid_normals(self):
        n = normals(np.full((5, 5), np.nan))
        self.assertEqual(n[2, 2].tolist(), [128, 0, 128])

    def test_bad_dimensions_and_extent(self):
        for grid in [np.empty((0, 0)), np.zeros(4), np.zeros((2, 2))]:
            with self.assertRaises(ValueError): normals(grid)
        with self.assertRaises(ValueError): normals(np.zeros((5, 5)), -1)


if __name__ == '__main__':
    unittest.main()
```
