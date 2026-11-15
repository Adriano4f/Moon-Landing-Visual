# The panorama from zero

## 1. What this view means

The panorama answers a local question: at one landing site and one UTC time, where are the Sun and Earth relative to the horizon? Azimuth is the horizontal direction, measured clockwise from geographic north: north is 0 degrees, east 90, south 180, and west 270. Elevation is the vertical angle above the local horizontal plane. Positive elevation is up; negative elevation is down. This graph is an angular diagram, not a perspective photograph. Its horizontal endpoints represent the same north direction, but are separate positions on the rectangular graph.

The default is now a standing-at-site scene with grey ground covering finite disks, plain-language statuses, whole-period cards and help. Read [the novice UI guide](07-ui.md) for its painter, occlusion and scale math. The original graph described here remains behind Technical view: yellow track for Sun, blue for Earth, larger current markers, grey zero-degree reference and red terrain line. A null profile still triggers an explicit flat-horizon warning, not a prediction of real terrain. DEM ingestion lives in prep, not this UI; see [the terrain guide](06-terrain.md). Interactive site/region selection is not included.

The rotating globe gives geographic context. Its fixed presentation light, one-minute rotation, and southward viewing tilt do not follow the selected UTC time. Do not infer power or communication availability from globe brightness. Only the Horizons tracks and core visibility functions drive the scientific readouts. The globe remains a separate crate with no dependency on lunar_core; the app passes only latitude and longitude to its marker setter.

Read [rendering from zero](03-moon.md) for vectors, matrices, projection, normals, and lighting, and [Rust from zero](04-globe-code.md) for ownership, borrowing, traits, closures, and platform compilation. The additions below build on those definitions rather than assuming familiarity with egui.

## 2. Preparing and validating the example

Run this command from the workspace root:

```sh
python3 prep/download_dem.py /home/ubuntu/data/lola
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
```

The prep CLI accepts site file, start date, stop date, step, optional output directory, optional external DEM directory, and optional observer height in meters (default 2). Horizons requests use observer center `coord@301`, geodetic longitude/latitude with DEM surface height plus observer height converted to kilometers, quantities 4 for azimuth/elevation, Sun body 10, and Earth body 399. The Horizons API is <https://ssd.jpl.nasa.gov/api/horizons.api>. Request construction is in [prep/src/main.rs](../prep/src/main.rs). Times are UTC. Both endpoints are included: $1{,}096\cdot24+1=26{,}305$ samples per body, including leap year 2028, below 90,000 rows per request.

The resulting `app/assets/data/connecting-ridge-c1.json` contains the published Connecting Ridge C1 site at 89.4398 degrees south, 222.83 degrees east, two aligned 26,305-sample tracks, 1,440 terrain bins, and terrain provenance. UTC Unix seconds count seconds since 1970-01-01 00:00 UTC; the first and last epochs are 1,798,761,600 and 1,893,456,000. The file is loaded at runtime, not embedded in the release executable or WASM. The terrain guide records its generated size and numerical checks.

The initial selected sample is 2027-01-01 00:00 UTC. Earth's azimuth is near the geometric mean direction of approximately 137 degrees, but lunar libration makes it vary; it must not be forced to 137. Solar elevation near this site is not restricted to a hard limit of plus/minus 1.6 degrees. Plotting never clips scientific values to an expected range. The three-year tracks now use the actual DEM-based observer altitude rather than the earlier zero-altitude demonstration.

`SiteData::parse` in [app/src/data.rs](../app/src/data.rs) uses serde's derived Deserialize implementation and serde_json::from_slice. Derive generates a decoder from named struct fields; C++ usually requires explicit adapters. The borrowed byte slice `&[u8]` is comparable to `std::span<const uint8_t>`. `#[serde(skip)]` leaves the Earth visibility cache out of the file schema. This cache starts empty and is filled only after validation.

Before indexing, the parser rejects empty arrays, unequal coordinate lengths, nonincreasing timestamps, timestamps outside chrono's supported UTC range, different Sun/Earth grids, nonfinite or out-of-range angles, empty site names, and invalid latitude. Longitude is east-positive and may wrap, so the renderer reduces finite longitude modulo 360. A non-null horizon must have at least one finite elevation bin in [-90,90]. An explicitly supplied empty profile is rejected rather than silently assigning core's special empty-profile value of -90 degrees. None and an empty profile are distinct concepts.

The validation's let-chain `if let Some(h) = &data.horizon && predicate(h)` first extracts the optional reference, then evaluates the predicate only when present. In C++, write `if (horizon && predicate(*horizon))`. No profile is unwrapped on the absent branch. This differs from an unconditional unwrap followed by a check.

Implementation: [app/src/data.rs:40](../app/src/data.rs#L40).

## 3. Immediate-mode egui from zero

A widget is an interactive element such as a slider or button. A retained-mode UI keeps a tree of widget objects between frames. An immediate-mode UI instead runs code describing this frame's widgets each frame. egui still retains internal interaction memory, such as keyboard focus and whether a popup is open. The application retains data and choices, not a tree of Slider objects.

MoonApp owns Globe and Panorama. Panorama owns optional validated data, an optional error, the current sample index, a window length, and a shared pending result. In every eframe App::ui callback, update first takes any completed result. Then ordinary Rust statements submit labels, controls, and plot items. Rust closures such as `|ui| { ... }` receive a temporary mutable UI reference; their C++ counterpart is a lambda taking a UI reference. A `move` completion closure, in contrast, owns its captured handles because it outlives the constructing stack frame.

A stable widget identity lets egui associate this frame's description with its prior interaction memory. Plot::new("panorama") and ComboBox::from_id_salt("time_window") provide stable identifiers. Do not replace them with the changing timestamp: that would create a logically new widget every frame. The slider borrows `&mut self.index`, comparable to passing a mutable size_t reference to a C++ immediate-mode control. A button's Response::clicked is true in the frame of activation; the code then changes the persistent index. Repaint shows the new state without finding and mutating a retained label object.

CentralPanel with Frame::NONE.fill(BLACK) keeps the globe background black. Context::set_theme selects ThemePreference::Dark, not merely one theme's visual colors: this prevents a light system theme from making black text disappear on the black panel. At widths of at least 700 logical points, Ui::columns creates two equal columns. Below 700, Ui::allocate_ui reserves a globe rectangle above the panorama. Globe height is 24 percent of viewport height, clamped to 140 through 220 logical points. A logical point is a device-independent UI unit, not necessarily one physical display pixel. The panorama gets at least 460 points of height. A vertical ScrollArea permits smaller screens to reach the remainder instead of shrinking the graph below 220 points. The finite viewport size is captured before entering ScrollArea: its scroll axis is unbounded and must not become an OpenGL viewport height. The same layout works in native windows and browser CSS viewports.

Implementation: [app/src/lib.rs:44](../app/src/lib.rs#L44), [app/src/panorama.rs:129](../app/src/panorama.rs#L129).

## 4. Asynchronous loading and ownership

The UI must not synchronously wait for disk or HTTP. The loader creates an `Arc<Mutex<Option<Result<SiteData,String>>>>`. Read the type from inside outward: Result contains success or a diagnostic; Option is None while pending; Mutex guards concurrent access; Arc lets producer and consumer share ownership. In C++, the corresponding concepts are expected, optional, mutex/lock_guard, and shared_ptr. A type alias named Pending shortens this nested type without changing its representation.

On native targets, std::thread::spawn moves a completion callback into a worker. It checks MOON_DATA_DIR first. Without an override, it tries assets/data beside the executable if that directory exists, then falls back to the compile-time app crate's assets/data directory. The filename is fixed to the committed example. An explicitly selected directory is authoritative: a missing file produces an error containing its path, not a silent fallback to different data. For a packaged Gentoo executable, copy the JSON into assets/data beside it or set MOON_DATA_DIR. The globe's MOON_ASSET_DIR remains independent.

On web targets, ehttp::Request::get requests assets/data/connecting-ridge-c1.json. app/index.html already tells Trunk to copy the assets directory. ehttp::fetch completes asynchronously; the callback checks Response::ok because HTTP errors can be successful transports. Response::bytes is then decoded with the same parser. Browser same-origin and CORS rules still apply. The data request does not change the globe's per-tier texture fetch policy.

The completion stores Some(Ok(data)) or Some(Err(message)) under a short mutex guard and calls Context::request_repaint. The UI calls Option::take under the same mutex: this moves the completion out and leaves None. It does not clone 26,305 samples each frame. Success moves into Panorama's data field; failure remains a visible diagnostic. While pending, the pane says Loading Horizons data. A failed load never substitutes synthetic tracks. Native and web branches are selected by cfg, comparable to C++ conditional compilation, rather than by a runtime branch that includes both implementations.

Implementation: [app/src/data.rs:125](../app/src/data.rs#L125), [app/src/panorama.rs:71](../app/src/panorama.rs#L71).

## 5. UTC selection and window mathematics

The slider represents an integer sample index $i$, with $0\leq i<n$. The corresponding time is exactly $t_i$, not an interpolated ephemeris. Here adjacent samples are one hour apart. A normalized slider fraction $u$ conceptually selects $i=\operatorname{round}(u(n-1))$, clamped to valid endpoints by the integer Slider. The displayed date uses chrono DateTime::from_timestamp and format with UTC explicitly printed. No local-time conversion or browser clock is involved. The single-sample buttons use saturating_sub and min to stay within the array.

The window choices are 1 day, 7 days, 30 days, or All data. A finite window of $H$ hours is centered on the selected time and clipped to the stored interval:

$$
t_{\min}=t_i-1{,}800H,\qquad t_{\max}=t_i+1{,}800H.
$$

The first displayed index is the first timestamp at least $t_{\min}$; the exclusive ending index is the first timestamp greater than $t_{\max}$. Slice::partition_point implements these lower/upper searches on the validated sorted times. In C++, use lower_bound and upper_bound. This is duration arithmetic, not a fixed number of array neighbors, so it remains correct for an irregular future time grid. Boundaries are inclusive in time but the Rust range is half-open in indices. The current sample is always included. At the first sample the default 30-day centered window contains only the following 15 days plus the current endpoint, 361 hourly samples. All data is a separate zero sentinel meaning 0..n, not a zero-duration window.

For example, on hourly timestamps [0,3600,7200], a 1-hour centered window around 3600 covers [1800,5400] seconds and contains only index 1. A 2-hour centered window covers [0,7200] and includes all three. Tests also use a deliberately irregular time grid to distinguish true duration selection from a sample-count approximation.

Implementation: [app/src/panorama.rs:504](../app/src/panorama.rs#L504), [app/src/panorama.rs:481](../app/src/panorama.rs#L481).

## 6. Plot construction and north-wrap handling

Plot::new creates a plot builder rather than drawing immediately. Builder methods consume and return the configuration; C++ fluent builders often return a reference instead. show runs a closure with a mutable PlotUi, into which we submit lines and points. height uses the remaining pane height with a 220-point minimum; width reserves 20 points at the right so the centered 360 N tick text is not clipped by the panel edge. Ui::columns normally justifies children across the full column, so the right pane uses with_layout and Layout::top_down(Align::Min) to honor this narrower plot width. C++ immediate-mode libraries similarly use child-container layout policies. Plot coordinates use f64; f64::from converts f32 angles without losing their original precision. This conversion cannot restore precision absent from the Horizons file.

default_x_bounds fixes 0 through 360. Zoom, drag, scrolling, and double-click reset are disabled so casual touch movement cannot replace the azimuth domain. include_y(-8) and include_y(8) provide a readable minimum vertical span while automatic y bounds can expand for actual supplied terrain or track extrema. set_margin_fraction keeps x padding zero and adds five percent vertically. x_axis_label and y_axis_label name the physical units. x_grid_spacer produces five GridMark values separated by 90 degrees. x_axis_formatter names N/E/S/W, including north again at 360. This explicitly prevents an automatic 100-degree grid from omitting the cardinal directions. Legend groups named items and matches yellow Sun and blue Earth labels to their series.

Line::new takes a name and points in chronological order, not sorted azimuth order. Sorting by azimuth would destroy the trajectory. A direct segment from 359 to 1 degrees would span almost the entire graph even though the physical motion is only two degrees. The segments helper normalizes exact 360 to 0 and starts a new polyline whenever $|A_j-A_{j-1}|>180$ degrees. Thus [358,359,0,1] becomes [358,359] and [0,1]. Reverse wraps are split too. No extra physical ephemeris is invented at the boundary; there is a small gap between the samples. This rule chooses the short angular transition and is appropriate for the hourly data. Very sparse data moving over 180 degrees between observations would require additional samples to remove ambiguity.

An empty selection yields no lines. A one-point piece remains safe; the separate current Points marker always renders. Each body's pieces share its name so they form one legend entry. Line::color and Points::color apply the series color. Points::radius is in screen points, not degrees. A constant HLine::new draws the zero-degree reference across the whole azimuth domain.

For a supplied Horizon with $m$ uniform bins, the renderer samples at $\max(m,360)$ intervals including both endpoints. Each point calls Horizon::at; all original bin boundaries are included, and the 360-degree endpoint interpolates back to north. It does not read a DEM or recompute terrain rays. The terrain line remains distinct from the zero reference.

Implementation: [app/src/panorama.rs:396](../app/src/panorama.rs#L396), [app/src/panorama.rs:519](../app/src/panorama.rs#L519).

## 7. Visibility and the rotating site marker

For Sun azimuth $A_S$, use the local horizon $h_S=\operatorname{Horizon.at}(A_S)$, or exactly zero if absent. lunar_core::sun_fraction now integrates the visible circular-segment area, not the visible diameter. It is not a time-availability percentage. Zero means below, one means fully above, and intermediate values are labeled partly above. At center elevation equal to horizon, one half of the disk area is visible. The cheap model holds the horizon elevation constant across the solar disk; it does not integrate a jagged skyline across every solar pixel. The [terrain guide](06-terrain.md#9-visible-solar-area-and-earth-clearance) derives the formula.

For Earth use lunar_core::visible with default margin $m=0.5$ degrees and the strict center-elevation test $E_E>h_E+m$, not a finite Earth disk calculation. Equality is below. One added slider restricts $m$ to 0.5 through 1.0 degrees; this is a communication-clearance choice, not antenna gain or a DSN station model. The loader caches the default bool vector after validation. Slider::new borrows Panorama's float mutably, suffix adds the units, and Response::changed triggers SiteData::update_earth_margin only after an edit. In C++, these correspond to a float reference, widget caption, and change-event boolean. The UI otherwise only indexes the cache. The Sun and Earth may legitimately have different visibility at the same moment.

The renderer's geographic convention maps latitude $\phi$ and east longitude $\lambda$ into a unit vector:

$$
\mathbf p=(\cos\phi\sin\lambda,\ \sin\phi,\ \cos\phi\cos\lambda).
$$

Globe::set_site accepts Option<(f64,f64)> and converts degrees to radians. None clears the marker; invalid coordinates also clear it. The globe does not know SiteData or Horizon. Rotation around north is followed by a southward presentation tilt:

$$
\begin{gathered}
M=R_x(-\pi/6)R_y(\theta),\\
R_x(\tau)=\begin{pmatrix}1&0&0\\0&\cos\tau&-\sin\tau\\0&\sin\tau&\cos\tau\end{pmatrix},\\
R_y(\theta)=\begin{pmatrix}\cos\theta&0&\sin\theta\\0&1&0\\-\sin\theta&0&\cos\theta\end{pmatrix}.
\end{gathered}
$$

math::rotation returns this column-major nine-float matrix. Its sin_cos call returns sine then cosine; C++ calls std::sin and std::cos separately. The GL uniform_matrix_3_f32_slice upload corresponds to glUniformMatrix3fv with transpose=false. Both shaders use u_model for geometry and mapped normals. The CPU marker uses the same matrix, rather than a separate animation or sign convention. std::array::from_fn computes each row's dot product with the input direction; C++ can fill an std::array in a counted loop.

Why 30 degrees? At camera $C=(0,0,3.3)$, a unit sphere point is front-facing only if $\mathbf p_w\cdot(C-\mathbf p_w)>0$, or $3.3p_{w,z}>1$. An equatorial view hides the 89.45-degree-south example behind the perspective limb even though it lies close to the drawn silhouette. The tilt moves the south pole to $(0,-\sqrt3/2,1/2)$, giving $3.3/2-1=0.65>0$: it is visible throughout rotation. At zero animation angle and a square viewport, that pole projects to approximately $(0,-0.7467)$ NDC, inside the lower part of the Moon. This is a deliberate geographic presentation view, not an orbital camera derived from Horizons. North still maps to the geographic +y object axis; the tilt does not alter latitude or longitude.

For world point $\mathbf p_w=M\mathbf p$, the shader's clip divisor is $w=3.3-p_{w,z}$. Reusing math::perspective gives $x_{ndc}=P_{00}p_{w,x}/w$ and $y_{ndc}=P_{11}p_{w,y}/w$. NDC is mapped into the UI rectangle centered at $(c_x,c_y)$ with dimensions $(W,H)$:

$$
x_{ui}=c_x+\tfrac12Wx_{ndc},\qquad y_{ui}=c_y-\tfrac12Hy_{ndc}.
$$

The minus sign reverses OpenGL's upward y into UI's downward y. Painter::circle_filled and circle_stroke add a small red, white-outlined overlay after the GL callback. Its radius is constant in logical points, not meters. Analytic sphere occlusion hides far-side points; it is not terrain occlusion. The marker is not drawn while textures are unavailable. CPU and GL aspect ratios agree apart from possible subpixel viewport rounding. The near-pole site's on-screen motion is naturally small, but it uses the exact same rotation as the texture. Unit tests project it through all 360 integer rotation degrees, check lower-hemisphere placement, reject invalid inputs, and check a hidden antipode.

Implementation: [globe/src/math.rs:125](../globe/src/math.rs#L125), [globe/src/math.rs:136](../globe/src/math.rs#L136), [globe/src/lib.rs:103](../globe/src/lib.rs#L103).

## 8. Panorama API index and C++ equivalents

The complete annotated [data source](#91-appsrcdatars) and [view source](#92-appsrcpanoramars) follow. App integration and renderer changes are quoted in [the complete globe code](04-globe-code.md). Each identifier below links to its implementing source; the surrounding sections explain the contracts, not just their names.

| API or construct | Purpose here | C++ equivalent |
|:--|:--|:--|
| [Deserialize, serde(skip), from_slice](../app/src/data.rs) | Generate/execute JSON decoding, omit derived cache | Explicit JSON adapters |
| [is_empty, len, windows, iter, any, contains](../app/src/data.rs) | Validate aligned finite ordered arrays | empty, size, adjacent ranges, any_of, interval checks |
| [is_finite, trim](../app/src/data.rs) | Reject unusable numeric/site metadata | isfinite, trimmed string |
| [DateTime::from_timestamp, Utc, format, to_string](../app/src/panorama.rs) | Explicit UTC text | sys_seconds and chrono formatting |
| [visible, sun_fraction, Horizon::at](../app/src/data.rs) | Core scientific functions | Project function/member calls |
| [Arc::new, Arc::clone, Mutex::new, lock](../app/src/data.rs) | Share a synchronized completion | shared_ptr, mutex, lock_guard |
| [type Pending, pub(crate)](../app/src/data.rs) | Alias and module visibility | using alias, internal interface |
| [map, map_err, and_then, map_or, map_or_else](../app/src/data.rs) | Preserve optional/results | Checked optional/expected transforms |
| [take, as_ref, as_deref, unwrap_or, unwrap_or_else](../app/src/data.rs) | Borrow/consume wrappers, defaults | optional references, exchange, value_or |
| [var_os, PathBuf::from, Path::new, join](../app/src/data.rs) | Runtime deployment paths | getenv and filesystem::path |
| [current_exe, parent, filter, is_dir, display](../app/src/data.rs) | Installed data directory, diagnostics | Executable lookup and directory checks |
| [fs::read, thread::spawn, env!](../app/src/data.rs) | Worker disk read, workspace fallback | File read, thread, build macro |
| [Request::get, fetch, Response::ok/status/bytes](../app/src/data.rs) | HTTP with status handling | Async HTTP client |
| [Context::request_repaint](../app/src/data.rs) | Wake completed UI | Post redraw event |
| [CentralPanel::show, Frame::fill, Context::set_theme, ThemePreference::Dark](../app/src/lib.rs) | Dark shell | Container/theme calls |
| [ScrollArea::vertical, allocate_ui, columns, with_layout, Layout::top_down, Align::Min](../app/src/lib.rs) | Responsive bounded panes | Child containers and columns |
| [available_size, available_width, available_height](../app/src/lib.rs) | Logical layout size | Layout size queries |
| [heading, label, strong, small, colored_label](../app/src/panorama.rs) | Current-frame text | Immediate-mode text functions |
| [spacing_mut, slider_width](../app/src/panorama.rs) | Slider row width | Local style dimension |
| [Slider::new, show_value, text, add](../app/src/panorama.rs) | Current index binding | Integer slider with mutable reference |
| [horizontal, button, clicked](../app/src/panorama.rs) | Sample-step controls | Row lambda, button response |
| [saturating_sub, min, max, clamp](../app/src/panorama.rs) | Valid indices and sizes | Bounded arithmetic |
| [ComboBox::from_id_salt, selected_text, show_ui, selectable_value](../app/src/panorama.rs) | Stable window menu | Combo with stable ID |
| [Range, partition_point, clone](../app/src/panorama.rs) | Half-open UTC interval | lower_bound, upper_bound |
| [Plot::new, height, width, default_x_bounds](../app/src/panorama.rs) | Plot identity, size, azimuth | Plot configuration |
| [include_y, set_margin_fraction](../app/src/panorama.rs) | Elevation and padding | Auto-range constraints |
| [allow_zoom, allow_drag, allow_scroll, allow_double_click_reset](../app/src/panorama.rs) | Preserve angular domain | Input policy flags |
| [x_axis_label, y_axis_label, x_grid_spacer, GridMark](../app/src/panorama.rs) | Units, cardinal grid | Axis/tick configuration |
| [x_axis_formatter, Legend::default, legend, show](../app/src/panorama.rs) | Compass and legend | Formatter lambda, callback |
| [HLine::new, PlotUi::hline](../app/src/panorama.rs) | Flat reference | Horizontal-line series |
| [Line::new, color, PlotUi::line](../app/src/panorama.rs) | Separate track/horizon pieces | Named polylines |
| [Points::new, radius, PlotUi::points](../app/src/panorama.rs) | Current markers | Screen-sized scatter points |
| [rem_euclid, f64::from, collect](../app/src/panorama.rs) | Normalize, convert, materialize | Positive modulo, casts, vectors |
| [last, last_mut, is_none_or, push](../app/src/panorama.rs) | Build seam pieces | Checked back access, push_back |
| [serde_json::json!, to_vec, pointer_mut](../app/src/data.rs) | Editable test schemas | JSON builder and pointer lookup |
| [include_bytes!, assert!, assert_eq!, unwrap](../app/src/data.rs) | Test-only fixture/assertions | Test fixture and framework |
| [Globe::set_site, direction, rotation, marker](../globe/src/lib.rs) | Independent site projection | Optional setter, math helpers |
| [sin_cos, to_radians, array::from_fn](../globe/src/math.rs) | Shared rigid transform | sin/cos, degree conversion, array fill |
| [uniform_matrix_3_f32_slice, u_model](../globe/src/lib.rs) | Geometry/normal matrix | glUniformMatrix3fv |
| [aspect_ratio, center, width, height, left_top](../globe/src/lib.rs) | NDC to UI | Rectangle geometry |
| [circle_filled, circle_stroke, Painter::text](../globe/src/lib.rs) | Marker and caption | Screen-space primitives |
| [Stroke::new, FontId::proportional, Align2::LEFT_TOP](../globe/src/lib.rs) | Overlay style | Pen, font, alignment |

## 9. Exact panorama source

These complete quotations include tests. Comments explain each statement, its purpose, and its C++ counterpart. Continued argument/expression lines belong to their enclosing statement; closing braces and delimiters close that scope. The scientific core and prep source are unchanged. JSON values are not repeated as 52,000 lines of typeset data.

### 9.1. app/src/data.rs

```rust
//! Validated Horizons data and asynchronous disk/HTTP loading, with no DEM policy.

// Import UTC conversion; C++ chrono provides calendar/time-point types.
use chrono::{DateTime, Utc};
// Import shared completion storage; C++ shared_ptr and a mutex-protected value.
use eframe::egui::{self, mutex::Mutex};
// Import the scientific contracts rather than duplicating visibility math; C++ includes their header.
use lunar_core::{EARTH_MARGIN_DEG, Horizon, Site, Track, visible};
// Import the derive trait used by the JSON decoder; C++ needs explicit serialization adapters.
use serde::Deserialize;
// Import reference-counted ownership; C++ std::shared_ptr.
use std::sync::Arc;

// Keep one filename for native disk and web HTTP paths; C++ constexpr string_view.
const FILE: &str = "connecting-ridge-c1.json";

// Generate JSON deserialization for the prep schema; C++ supplies field adapters manually.
#[derive(Deserialize)]
// Own a validated, aligned pair of tracks; C++ aggregate with vectors and optional horizon.
pub(crate) struct SiteData
{
	// Retain geographic latitude and east longitude; C++ a Site value member.
	pub site: Site,
	// Own the Sun's sample arrays; C++ a Track value member.
	pub sun: Track,
	// Own the Earth's sample arrays on the same UTC grid; C++ a Track value member.
	pub earth: Track,
	// Preserve null as absent terrain, not as an invented DEM; C++ optional<Horizon>.
	pub horizon: Option<Horizon>,
	// This cache is computed after validation, not read from JSON; C++ excludes a derived member from serialization.
	#[serde(skip)]
	// Cache the core's Earth center visibility once per load; C++ vector<bool>.
	pub earth_above: Vec<bool>,
}

// Attach helpers to this owned model; C++ member definitions outside the class.
impl SiteData
{
	// Decode bytes and enforce preconditions before any indexing; C++ expected<SiteData,string>.
	pub fn parse(bytes: &[u8]) -> Result<Self, String>
	{
		// Borrow bytes without copying and propagate JSON errors; C++ span plus expected error propagation.
		let mut data: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
		// Reject unusable site metadata; C++ checks finite doubles and numeric intervals.
		if data.site.name.trim().is_empty()
			|| !data.site.lat_deg.is_finite()
			|| !data.site.lon_deg.is_finite()
			|| !(-90.0..=90.0).contains(&data.site.lat_deg)
		{
			// Return a diagnostic instead of creating a misleading marker; C++ unexpected<string>.
			return Err("Invalid site name or coordinates".into());
		}
		// Validate both borrowed tracks identically; C++ range-for over two const pointers.
		for track in [&data.sun, &data.earth]
		{
			// Refuse empty/misaligned arrays, unordered times and invalid directions; C++ short-circuit predicates.
			if track.t.is_empty()
				|| track.t.len() != track.az_deg.len()
				|| track.t.len() != track.el_deg.len()
				|| track.t.windows(2).any(|t| t[0] >= t[1])
				|| track
					.t
					.iter()
					.any(|&t| DateTime::<Utc>::from_timestamp(t, 0).is_none())
				|| track
					.az_deg
					.iter()
					.any(|a| !a.is_finite() || !(0.0..=360.0).contains(a))
				|| track
					.el_deg
					.iter()
					.any(|e| !e.is_finite() || !(-90.0..=90.0).contains(e))
			{
				// Avoid indexing invalid data or bridging missing arrays; C++ unexpected<string>.
				return Err("Invalid or empty track arrays, timestamps, or angles".into());
			}
		}
		// Require exact timestamp correspondence, not merely equal lengths; C++ vector equality.
		if data.sun.t != data.earth.t
		{
			// Report incompatible tracks; C++ returns an error alternative.
			return Err("Sun and Earth timestamps differ".into());
		}
		// Check supplied terrain only; no DEM is read or selected here; C++ if-init with optional.
		if let Some(h) = &data.horizon
			&& (h.h_deg.is_empty()
				|| h.h_deg
					.iter()
					.any(|e| !e.is_finite() || !(-90.0..=90.0).contains(e)))
		{
			// A supplied profile must actually contain valid bins; C++ unexpected<string>.
			return Err("A supplied horizon needs nonempty finite elevation bins".into());
		}
		// Cache the default communication clearance, not mere center-above-horizon visibility; C++ member call.
		data.update_earth_margin(EARTH_MARGIN_DEG);
		// Move the validated model to the caller; C++ expected containing a moved value.
		Ok(data)
	}

	// Interpolate a supplied profile or explicitly choose flat zero; C++ const member returning float.
	pub fn horizon_at(&self, az: f32) -> f32
	{
		// Do not use core's empty-profile -90 convention for missing DEM; C++ optional transform with fallback.
		self.horizon.as_ref().map_or(0.0, |h| h.at(az))
	}

	// Recompute only when the user changes clearance; C++ mutating cache member function.
	pub fn update_earth_margin(&mut self, margin_deg: f32)
	{
		// Retain the labeled flat fallback for older datasets; C++ temporary constant profile.
		let flat = Horizon { h_deg: vec![0.0] };
		// Delegate the strict elevation > horizon + margin criterion to core; C++ vector assignment from a scientific function.
		self.earth_above = visible(
			&self.earth,
			self.horizon.as_ref().unwrap_or(&flat),
			margin_deg,
		);
	}
}

// Name pending/success/failure shared storage; C++ shared_ptr<mutex-protected optional<expected<SiteData,string>>>.
pub(crate) type Pending = Arc<Mutex<Option<Result<SiteData, String>>>>;

// Start one asynchronous load and wake egui when it finishes; C++ factory returning shared completion state.
pub(crate) fn load(ctx: egui::Context) -> Pending
{
	// None means pending, distinct from success and failure; C++ optional starts disengaged.
	let pending = Arc::new(Mutex::new(None));
	// Give the completion its own shared owner; C++ copies a shared_ptr.
	let completed = Arc::clone(&pending);
	// Move captured owners into a one-shot callback; C++ a mutable value-capturing lambda.
	let done = move |bytes: Result<Vec<u8>, String>| {
		// Decode only on successful I/O, then atomically publish the result; C++ expected::and_then under lock_guard.
		*completed.lock() = Some(bytes.and_then(|b| SiteData::parse(&b)));
		// Wake the event loop from either a worker or browser completion; C++ posts a redraw event.
		ctx.request_repaint();
	};
	// Compile the disk loader only for native targets; C++ #if around a function body.
	#[cfg(not(target_arch = "wasm32"))]
	// Keep disk I/O off the UI thread; C++ std::thread with a value-captured callback.
	std::thread::spawn(move || {
		// Accept a deployment override; C++ getenv converted to optional<filesystem::path>.
		let directory = std::env::var_os("MOON_DATA_DIR")
			.map(std::path::PathBuf::from)
			.unwrap_or_else(|| {
				// Prefer assets/data beside the executable; C++ filesystem::path with optional error handling.
				std::env::current_exe()
					.ok()
					.and_then(|p| p.parent().map(|p| p.join("assets/data")))
					.filter(|p| p.is_dir())
					// Fall back to the workspace app assets; C++ build-defined source path.
					.unwrap_or_else(|| {
						std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/data")
					})
			});
		// Include the failing path in the error instead of silently switching files; C++ filesystem read with expected.
		let path = directory.join(FILE);
		// Publish exactly one disk result; C++ invokes the captured completion.
		done(std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display())));
	});
	// Compile HTTP loading only for the browser; C++ #if for platform code.
	#[cfg(target_arch = "wasm32")]
	// Fetch the same relative resource copied by Trunk, without embedding it in WASM; C++ asynchronous HTTP request.
	ehttp::fetch(
		ehttp::Request::get(format!("assets/data/{FILE}")),
		move |response| {
			// Reject HTTP errors even if their bodies happen to resemble JSON; C++ expected::and_then.
			done(response.and_then(|r| {
				if r.ok
				{
					Ok(r.bytes)
				}
				else
				{
					Err(format!("HTTP {} loading {FILE}", r.status))
				}
			}));
		},
	);
	// Return another owner while completion stays alive independently; C++ returns shared_ptr.
	pending
}

// Compile test-only fixtures out of release binaries; C++ conditional test compilation.
#[cfg(test)]
// Isolate test names; C++ anonymous/test namespace.
mod tests
{
	// Access private implementation helpers; C++ includes the implementation's test interface.
	use super::*;

	// Build a tiny editable valid schema; C++ returns a JSON value object.
	fn fixture() -> serde_json::Value
	{
		// Two aligned samples cross north, with Earth above flat zero; C++ constructs an object/array tree.
		serde_json::json!({"site":{"name":"Test","lat_deg":-89.45,"lon_deg":222.69},
			"sun":{"t":[0,3600],"az_deg":[359,1],"el_deg":[0,1]},
			"earth":{"t":[0,3600],"az_deg":[137,138],"el_deg":[1,-1]},"horizon":null})
	}

	// Register a test; C++ testing framework macro.
	#[test]
	// Verify the committed Horizons payload, not a substitute dataset; C++ a data regression test.
	fn bundled_three_years_are_aligned_hourly_and_have_terrain()
	{
		// Embed only in the test executable, never in the app/WASM; C++ test-only binary fixture.
		let d = SiteData::parse(include_bytes!("../assets/data/connecting-ridge-c1.json")).unwrap();
		// Count inclusive interval endpoints; C++ assert equality.
		assert_eq!(d.sun.t.len(), 26305);
		// Keep the request below the published row limit; C++ assert comparison.
		assert!(d.sun.t.len() < 90_000);
		// Check UTC cadence, independently of sample count; C++ ranges::all_of adjacent pairs.
		assert!(d.sun.t.windows(2).all(|t| t[1] - t[0] == 3600));
		// Verify exact endpoint dates; C++ chrono time-point equality.
		assert_eq!(d.sun.t[0], 1_798_761_600);
		// Include the leap year 2028 through the 2030 endpoint; C++ assert equality.
		assert_eq!(*d.sun.t.last().unwrap(), 1_893_456_000);
		// Real terrain uses 1,440 geographic azimuths; C++ optional extraction and vector size assertion.
		assert_eq!(d.horizon.as_ref().unwrap().h_deg.len(), 1440);
		// Keep the selected position tied to the published 2 m C1 site, not a rounded label; C++ coordinate regression.
		assert_eq!((d.site.lat_deg, d.site.lon_deg), (-89.4398, 222.83));
	}

	// Register a test; C++ testing framework macro.
	#[test]
	// Check normal flat and supplied-profile visibility; C++ unit test.
	fn flat_and_interpolated_terrain_visibility()
	{
		// Own an editable fixture; C++ JSON value.
		let mut v = fixture();
		// Parse the null fallback; C++ serialize then deserialize for schema coverage.
		let mut flat = SiteData::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
		// Earth centers use strict elevation > horizon; C++ vector comparison.
		assert_eq!(flat.earth_above, [true, false]);
		// Raising the clearance to one degree rejects an Earth center exactly one degree high; C++ strict-threshold check.
		flat.update_earth_margin(1.0);
		assert_eq!(flat.earth_above, [false, false]);
		// Restore the default before exercising the existing horizon checks; C++ cache update.
		flat.update_earth_margin(EARTH_MARGIN_DEG);
		// At the horizon half the solar disk is visible; C++ call the scientific function.
		assert!((lunar_core::sun_fraction(0.0, flat.horizon_at(359.0)) - 0.5).abs() < 1e-6);
		// Supply a synthetic cyclic profile, not a chosen DEM; C++ replace a JSON member.
		v["horizon"] = serde_json::json!({"h_deg":[1,3]});
		// Parse and evaluate terrain through the same code path; C++ checked conversion.
		let terrain = SiteData::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
		// Uniform bins interpolate at 90 degrees; C++ floating-point tolerance assertion.
		assert!((terrain.horizon_at(90.0) - 2.0).abs() < 1e-6);
		// Both Earth samples are now below the supplied profile; C++ ranges::none_of.
		assert!(terrain.earth_above.iter().all(|v| !v));
		// Values straddling half a degree prove parsing uses the default clearance, not zero; C++ threshold fixture.
		let mut close = fixture();
		close["earth"]["el_deg"] = serde_json::json!([0.25, 0.75]);
		let clearance = SiteData::parse(&serde_json::to_vec(&close).unwrap()).unwrap();
		assert_eq!(clearance.earth_above, [false, true]);
	}

	// Register a test; C++ testing framework macro.
	#[test]
	// Cover malformed, empty, mismatched and invalid numeric inputs; C++ table-driven rejection test.
	fn rejects_invalid_inputs()
	{
		// Empty input is not an empty successful dataset; C++ assert error alternative.
		assert!(SiteData::parse(b"").is_err());
		// Try independent corruptions of otherwise valid data; C++ range-for over replacement pairs.
		for (pointer, replacement) in [
			("/sun/t", serde_json::json!([])),
			("/sun/az_deg", serde_json::json!([1])),
			("/sun/t", serde_json::json!([0, 0])),
			("/earth/t", serde_json::json!([0, 3601])),
			("/site/lat_deg", serde_json::json!(-100)),
			("/sun/el_deg", serde_json::json!([91, 0])),
			("/sun/az_deg", serde_json::json!([361, 0])),
			("/horizon", serde_json::json!({"h_deg":[]})),
			("/horizon", serde_json::json!({"h_deg":[100]})),
		]
		{
			// Reset the fixture so errors do not mask one another; C++ a fresh local JSON object.
			let mut v = fixture();
			// Replace the selected nested member; C++ assign through a JSON pointer.
			*v.pointer_mut(pointer).unwrap() = replacement;
			// The production parser must reject every corruption; C++ assert with diagnostic context.
			assert!(
				SiteData::parse(&serde_json::to_vec(&v).unwrap()).is_err(),
				"{pointer}"
			);
		}
	}
}
```

### 9.2. app/src/panorama.rs

```rust
//! Immediate-mode panorama controls over validated, hourly Horizons samples.

// Import the owned model and pending-loader state; C++ includes project declarations.
use crate::data::{self, Pending, SiteData};
// Import UTC formatting without a platform clock; C++ chrono formats sys_seconds.
use chrono::{DateTime, Utc};
// Import UI widgets and plot primitives; C++ using declarations shorten qualified names.
use eframe::egui::{self, Color32};
// Import named plot items and explicit compass grid marks; C++ plotting library types.
use egui_plot::{GridMark, HLine, Legend, Line, Plot, Points};
// Import the track contract and solar disk visibility function; C++ project header declarations.
use lunar_core::{EARTH_MARGIN_DEG, Track, sun_fraction};
// Name half-open index intervals; C++ a pair of iterator offsets.
use std::ops::Range;

// Own only persistent choices and data, not a retained widget tree; C++ an immediate-mode view model.
pub(crate) struct Panorama
{
	// Shared result slot while loading; C++ shared_ptr to synchronized optional<expected>.
	pending: Pending,
	// Validated successful data, absent while pending or failed; C++ optional<SiteData>.
	data: Option<SiteData>,
	// Terminal diagnostic kept visible; C++ optional<string>.
	error: Option<String>,
	// Current sample, not interpolated ephemerides; C++ size_t.
	index: usize,
	// Centered window length in hours; zero means the entire dataset; C++ int64_t.
	hours: i64,
	// User-selected angular clearance for a direct-to-Earth link; C++ float.
	earth_margin_deg: f32,
	// Remember the optional numeric view; C++ bool.
	technical: bool,
	// Keep the help dialog open across frames; C++ bool.
	help: bool,
	// Cache full-period values rather than recomputing every animated frame; C++ value member.
	summary: lunar_core::Stats,
}

// Define view-model member functions; C++ out-of-class member definitions.
impl Panorama
{
	// Start loading once during application construction; C++ a constructor with an asynchronous operation.
	pub fn new(ctx: egui::Context) -> Self
	{
		// Native launchers can select a reproducible starting sample; C++ environment parsing with fallback.
		#[cfg(not(target_arch = "wasm32"))]
		let index = std::env::var("MOON_SAMPLE_INDEX")
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(0);
		// Browsers begin at the first actual sample; C++ web-specific default.
		#[cfg(target_arch = "wasm32")]
		let index = 0;
		// Keep the chosen starting sample and a centered 30-day window; C++ aggregate initialization.
		Self {
			pending: data::load(ctx),
			data: None,
			error: None,
			index,
			hours: 720,
			// Default to the user-selected half-degree communication margin; C++ scalar initializer.
			earth_margin_deg: EARTH_MARGIN_DEG,
			// Start in the nontechnical view without an obstructing dialog; C++ value initialization.
			technical: false,
			help: false,
			summary: lunar_core::Stats::default(),
		}
	}

	// Consume a completion at most once, before drawing; C++ moves out of an optional under a mutex.
	pub fn update(&mut self)
	{
		// Drop the short-lived mutex guard before changing the view model; C++ scoped lock_guard.
		let result = self.pending.lock().take();
		// Exhaustively separate pending, success and failure; C++ switch over optional<expected> alternatives.
		match result
		{
			// Move valid data into UI ownership; C++ move assignment.
			Some(Ok(data)) =>
			{
				// A requested initial index must fit the data that actually arrived; C++ clamped index.
				self.index = self.index.min(data.sun.t.len() - 1);
				// Compute only after validation; C++ cached statistics.
				self.summary = availability(&data);
				// Move ownership into the view; C++ move assignment.
				self.data = Some(data);
			}
			// Keep errors visible instead of retrying every animation frame; C++ move assignment.
			Some(Err(error)) => self.error = Some(error),
			// Pending or already consumed requires no work; C++ empty branch.
			None =>
			{}
		}
	}

	// Return coordinates without exposing scientific data to globe; C++ optional<pair<double,double>>.
	pub fn site(&self) -> Option<(f64, f64)>
	{
		// Borrow the model and copy two scalars; C++ optional transform of a const reference.
		self.data.as_ref().map(|d| (d.site.lat_deg, d.site.lon_deg))
	}

	// Copy only the current scientific directions/availability into the independent renderer; C++ optional<array<BodyDirection,2>>.
	pub fn directions(&self) -> Option<[globe::BodyDirection; 2]>
	{
		// Loading and errors expose no invented ephemerides; C++ optional transform.
		self.data.as_ref().map(|data| {
			[
				// Any visible solar area makes a solid Sun ray, matching the main status; C++ aggregate value.
				globe::BodyDirection {
					az_deg: data.sun.az_deg[self.index],
					el_deg: data.sun.el_deg[self.index],
					visible: sun_fraction(
						data.sun.el_deg[self.index],
						data.horizon_at(data.sun.az_deg[self.index]),
					) > 0.0,
				},
				// The selected radio clearance controls the Earth ray's brightness; C++ cached boolean.
				globe::BodyDirection {
					az_deg: data.earth.az_deg[self.index],
					el_deg: data.earth.el_deg[self.index],
					visible: data.earth_above[self.index],
				},
			]
		})
	}

	// Present a plain-language scene first, with optional numerical detail; C++ immediate-mode member function.
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		// Scrollable child UIs must wrap prose instead of widening the entire panorama; C++ local text-layout policy.
		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
		// Keep the explanation available even while data is loading; C++ row callback.
		ui.horizontal(|ui| {
			// Name the viewpoint, not the plotting library; C++ heading widget.
			ui.heading("Standing on the Moon");
			// A question mark toggles the short explanation; C++ button event.
			if ui
				.button("?")
				.on_hover_text("What am I looking at?")
				.clicked()
			{
				self.help = !self.help;
			}
		});
		// A modeless window can be closed without changing scientific state; C++ dialog with bool reference.
		egui::Window::new("How to read this view").open(&mut self.help).vscroll(true).default_width(310.0).show(ui.ctx(), |ui| {
			// Limit line length for reading on small screens; C++ layout width constraint.
			ui.set_max_width(320.0);
			// Define the horizon before using it; C++ wrapped text widget.
			ui.label("The horizon is the edge of the ground against the sky. Nearby hills can hide the Sun or Earth.");
			// Define the round-body wording and angular/time units; C++ explanatory label.
			ui.label("A disk is the round face of the Sun or Earth. Its visible percentage says how much of that face clears the hills. A degree measures an angle: a full turn is 360 degrees. UTC is the shared world clock, not your local time.");
			// Explain the panoramic mapping without assumed graphics knowledge; C++ text widget.
			ui.label("The strip shows every direction around you, from north through east, south and west, back to north. Up means higher in the sky. Height is stretched to make low hills easier to see; the amount is written below.");
			// Explain both the paths and finite disks; C++ text widget.
			ui.label("Yellow is the Sun and blue is Earth. Grey ground hides the parts below the horizon. Faint lines show their paths during your chosen time window. The globe beside it is an illustration, not this sky view.");
			// State the distinction between a potential geometric link and a guaranteed service; C++ text widget.
			ui.label("Direct communication means a radio signal could travel straight to Earth without the Moon blocking it. The safety margin asks for extra room above the hills. The blue disk can be visible while the radio test still fails.");
			// Avoid promising engineering capabilities from a geometric model; C++ text widget.
			ui.label("Sun visible means sunlight could reach solar panels, not guaranteed electrical power. This does not model radio relays, ground-station availability, radio hardware, panel direction, eclipses, temperatures or landing safety. The table uses hourly samples, so short events can be missed.");
		});
		// Never draw invented tracks while waiting or after an error; C++ optional extraction.
		let Some(data) = &mut self.data
		else
		{
			// Show the real loader state; C++ optional value_or.
			ui.label(
				self.error
					.as_deref()
					.unwrap_or("Loading site and sky data..."),
			);
			// Stop before indexing arrays; C++ early return.
			return;
		};
		// Identify the published observation point; C++ borrowed text.
		ui.small(&data.site.name);
		// Delegate the solar area to the scientific core; C++ pure function call.
		let fraction = sun_fraction(
			data.sun.el_deg[self.index],
			data.horizon_at(data.sun.az_deg[self.index]),
		);
		// Green indicates any available sunlight, not full disk exposure; C++ conditional text/color pair.
		let (sun_text, sun_color) = if fraction > 0.0
		{
			(
				format!(
					"Sun visible: solar power available ({:.1}% of the disk)",
					fraction * 100.0
				),
				Color32::LIGHT_GREEN,
			)
		}
		else
		{
			(
				"Sun hidden behind terrain: no solar power".into(),
				Color32::LIGHT_RED,
			)
		};
		// Use large wrapping text so the whole status fits a phone; C++ rich-text widget.
		ui.label(
			egui::RichText::new(sun_text)
				.size(18.0)
				.strong()
				.color(sun_color),
		);
		// Include the selected radio clearance, not just visible blue pixels; C++ cached bool.
		let earth_visible = data.earth_above[self.index];
		// Keep the full plain-language status visible without interpreting a graph; C++ conditional string.
		ui.label(
			egui::RichText::new(
				if earth_visible
				{
					"Earth visible: direct communication possible"
				}
				else
				{
					"Earth hidden: no direct communication"
				},
			)
			.size(18.0)
			.strong()
			.color(
				if earth_visible
				{
					Color32::LIGHT_GREEN
				}
				else
				{
					Color32::LIGHT_RED
				},
			),
		);
		// Qualify a blocked radio link that may still have a partly visible disk; C++ explanatory label.
		ui.small("Radio status includes the safety margin, not just the visible blue disk.");
		// Keep UTC explicit, independent of the computer's time zone; C++ time formatting.
		ui.strong(utc_label(data.sun.t[self.index]));
		// Let the time slider fill the available width; C++ local style assignment.
		ui.spacing_mut().slider_width = (ui.available_width() - 65.0).max(100.0);
		// Select only real hourly samples; C++ bounded integer slider.
		ui.add(
			egui::Slider::new(&mut self.index, 0..=data.sun.t.len() - 1)
				.show_value(false)
				.text("Time"),
		);
		// Wrapping rows remain usable when narrow; C++ flow layout callback.
		ui.horizontal_wrapped(|ui| {
			// Saturating subtraction keeps the first sample safe; C++ clamped decrement.
			if ui.button("< 1 hour").clicked()
			{
				self.index = self.index.saturating_sub(1);
			}
			// Clamp at the inclusive final epoch; C++ clamped increment.
			if ui.button("1 hour >").clicked()
			{
				self.index = (self.index + 1).min(data.sun.t.len() - 1);
			}
			// Explain that the window controls faint paths, not summary percentages; C++ combo box.
			egui::ComboBox::from_id_salt("time_window")
				.selected_text(window_label(self.hours))
				.show_ui(ui, |ui| {
					// Use durations rather than arbitrary point counts; C++ range-for.
					for (h, label) in [
						(24, "1 day"),
						(168, "7 days"),
						(720, "30 days"),
						(0, "All data"),
					]
					{
						// Bind the chosen duration; C++ radio item.
						ui.selectable_value(&mut self.hours, h, label);
					}
				});
			// Preserve the original plot as an explicit alternative; C++ bool checkbox.
			ui.checkbox(&mut self.technical, "Technical view");
		});
		// Define the clearance control before presenting its numeric value; C++ wrapped text.
		ui.label("Safety margin above terrain for the radio signal");
		// Reserve room for the editable value and units as well as the slider; C++ bounded control width.
		ui.spacing_mut().slider_width = (ui.available_width() - 125.0).max(60.0);
		// Recompute full-period percentages only when the clearance changes; C++ change event.
		if ui
			.add(
				egui::Slider::new(&mut self.earth_margin_deg, 0.5..=1.0)
					.suffix(" degrees")
					.fixed_decimals(2),
			)
			.changed()
		{
			// Update the cache used by the current status and summary consistently; C++ member call.
			data.update_earth_margin(self.earth_margin_deg);
			// Reuse core's duration-weighted statistics; C++ cached value assignment.
			self.summary = availability(data);
		}
		// Show the same scientific data in either presentation; C++ view-mode branch.
		if self.technical
		{
			technical(ui, data, self.index, self.hours);
		}
		else
		{
			// All disks are angular-size correct before the labeled vertical stretch; C++ renderer call.
			crate::sky::show(
				ui,
				data,
				self.index,
				window(&data.sun.t, self.index, self.hours),
			);
		}
		// Anchor summary cards to the full period, never the selected path window; C++ subheading.
		ui.strong("Across the whole data period");
		// Show exact dates rather than an unexplained average; C++ formatted label.
		ui.small(format!(
			"{} to {}",
			utc_label(data.sun.t[0]),
			utc_label(*data.sun.t.last().unwrap())
		));
		// Reserve frame padding and the gutter before dividing the available width; C++ bounded card size.
		let card_width = ((ui.available_width() - 40.0) / 2.0).clamp(90.0, 180.0);
		// Two columns wrap well on both phone and desktop panels; C++ grid of framed cards.
		egui::Grid::new("site_summary")
			.num_columns(2)
			.spacing([8.0, 6.0])
			.show(ui, |ui| {
				// Include units and the definition of sunlight in the visible labels; C++ range-for over values.
				for (i, (title, value)) in [
					("Time with any Sun", format!("{:.1}%", self.summary.pct_sun)),
					(
						"Time with Earth link",
						format!("{:.1}%", self.summary.pct_earth),
					),
					("Time with both", format!("{:.1}%", self.summary.pct_both)),
					(
						"Longest night",
						format!("{:.2} days", self.summary.longest_night_h / 24.0),
					),
				]
				.iter()
				.enumerate()
				{
					// A filled frame separates each summary from controls; C++ styled card container.
					egui::Frame::group(ui.style())
						.fill(Color32::from_gray(28))
						.show(ui, |ui| {
							// Stack each number above its caption instead of inheriting the grid's row layout; C++ column container.
							ui.vertical(|ui| {
								// Give both cards equal widths inside the parent column; C++ width constraint.
								ui.set_width(card_width);
								// Render the number prominently; C++ large-text label.
								ui.label(egui::RichText::new(value).size(21.0).strong());
								// Explain the number in ordinary language; C++ small label.
								ui.small(*title);
							});
						});
					// Close each pair explicitly; C++ grid row break.
					if i % 2 == 1
					{
						ui.end_row();
					}
				}
			});
		// Avoid implying that hourly held-sample statistics are continuous event solutions; C++ disclaimer.
		ui.small("Hourly estimates. Sun time means at least part of the disk is visible. Power and radio service are not guaranteed.");
	}
}

// Compute full-period availability using the same predicates as the current status; C++ pure aggregate helper.
fn availability(data: &SiteData) -> lunar_core::Stats
{
	// Keep partial sunlight distinct from a full-disk requirement; C++ transform to vector<bool>.
	let sun: Vec<_> = data
		.sun
		.az_deg
		.iter()
		.zip(&data.sun.el_deg)
		.map(|(&a, &e)| sun_fraction(e, data.horizon_at(a)) > 0.0)
		.collect();
	// Hold each boolean over its actual timestamp interval; C++ scientific function call.
	lunar_core::stats(&data.sun.t, &sun, &data.earth_above)
}

// Retain the original numerical plot independently of the novice scene; C++ free drawing function.
fn technical(ui: &mut egui::Ui, data: &SiteData, index: usize, hours: i64)
{
	// Select by UTC duration, allowing irregular future timestamps; C++ binary-search interval.
	let range = window(&data.sun.t, index, hours);
	// Explain endpoint clipping without crowding a phone with duplicate timestamps; C++ formatted count label.
	ui.small(format!(
		"Window: {} samples (centered, clipped)",
		range.len()
	));
	// Keep a readable graph while leaving space for summary cards below it; C++ fixed logical height.
	let height = 260.0;
	// Fix azimuth bounds while allowing elevation to fit terrain/track extrema; C++ plotting builder.
	Plot::new("panorama")
		// Leave room for the right endpoint's centered tick label inside the panel clip; C++ explicit plot width.
		.width((ui.available_width() - 20.0).max(100.0))
		.height(height)
		.default_x_bounds(0.0, 360.0)
		.include_y(-8.0)
		.include_y(8.0)
		.set_margin_fraction(egui::vec2(0.0, 0.05))
		.allow_zoom(false)
		.allow_drag(false)
		.allow_scroll(false)
		.allow_double_click_reset(false)
		.x_axis_label("Azimuth (deg, north clockwise)")
		.y_axis_label("Elevation (deg)")
		// Force cardinal ticks rather than accepting automatic 100-degree spacing; C++ generate five grid marks.
		.x_grid_spacer(|_| {
			(0..=4)
				.map(|i| GridMark {
					value: f64::from(i) * 90.0,
					step_size: 90.0,
				})
				.collect()
		})
		// Name both north endpoints of the unwrapped plot; C++ formatter lambda.
		.x_axis_formatter(|mark, _| {
			match mark.value as i32
			{
				0 => "0 N",
				90 => "90 E",
				180 => "180 S",
				270 => "270 W",
				_ => "360 N",
			}
			.into()
		})
		.legend(Legend::default())
		.show(ui, |plot| {
			// Always show the geometric flat reference, even if terrain is supplied; C++ submit a horizontal line.
			plot.hline(HLine::new("Flat 0 deg", 0.0).color(Color32::GRAY));
			// Add terrain only when present; this is profile interpolation, not DEM ingestion; C++ if(optional).
			if let Some(horizon) = &data.horizon
			{
				// Include every original uniform bin, with extra points for coarse profiles; C++ max(size,360).
				let bins = horizon.h_deg.len().max(360);
				// Close the cyclic line at 360 using Horizon::at, including all bin boundaries; C++ transform a counted range.
				let points: Vec<_> = (0..=bins)
					.map(|i| {
						// Convert this uniform bin boundary to degrees; C++ float arithmetic.
						let az = i as f32 * 360.0 / bins as f32;
						// Ask the core for the interpolated elevation, then widen plot coordinates; C++ array<double,2>.
						[f64::from(az), f64::from(horizon.at(az))]
					})
					.collect();
				// Keep the terrain visually distinct from the flat reference and both bodies; C++ submit a colored polyline.
				plot.line(Line::new("Terrain", points).color(Color32::LIGHT_RED));
			}
			// Apply identical window/wrap rules to both tracks; C++ range-for over named borrowed tracks.
			for (name, track, color) in [
				("Sun", &data.sun, Color32::YELLOW),
				("Earth", &data.earth, Color32::LIGHT_BLUE),
			]
			{
				// Never connect segments separated by a north wrap; C++ range-for over polyline pieces.
				for segment in segments(track, range.clone())
				{
					// Same body name groups pieces under a single legend entry; C++ submit each independent line.
					plot.line(Line::new(name, segment).color(color));
				}
				// Mark the current direction even for a one-sample interval; C++ one-point scatter series.
				plot.points(
					Points::new(
						format!("{name} now"),
						vec![[
							f64::from(track.az_deg[index].rem_euclid(360.0)),
							f64::from(track.el_deg[index]),
						]],
					)
					.color(color)
					.radius(4.0),
				);
			}
		});
}

// Format a UTC epoch without consulting a platform clock; C++ chrono::format of sys_seconds.
fn utc_label(t: i64) -> String
{
	// Defensive fallback is unreachable after validation, but formatting never panics; C++ optional transform/value_or.
	DateTime::<Utc>::from_timestamp(t, 0).map_or_else(
		|| "Invalid UTC".into(),
		|d| d.format("%Y-%m-%d %H:%M UTC").to_string(),
	)
}

// Map the small persistent integer choice to menu text; C++ switch returning string_view.
fn window_label(hours: i64) -> &'static str
{
	// The lifetime is static because literals live for the process; C++ static string storage.
	match hours
	{
		24 => "1 day",
		168 => "7 days",
		720 => "30 days",
		_ => "All data",
	}
}

// Find a centered, clipped interval; precondition: validated nonempty sorted times and a valid index; C++ binary searches.
fn window(times: &[i64], index: usize, hours: i64) -> Range<usize>
{
	// Zero is an explicit entire-dataset choice, not a zero-duration plot; C++ early return.
	if hours == 0
	{
		return 0..times.len();
	}
	// Half of hours converted to seconds on each side of the current sample; C++ integer multiplication.
	let half = hours * 1800;
	// Include exact boundary samples; C++ lower_bound and upper_bound offsets.
	times.partition_point(|&t| t < times[index] - half)
		..times.partition_point(|&t| t <= times[index] + half)
}

// Split at a >180-degree jump, preserving time order; precondition: validated arrays/range; C++ vector<vector<point>>.
pub(crate) fn segments(track: &Track, range: Range<usize>) -> Vec<Vec<[f64; 2]>>
{
	// Own separate polylines rather than drawing a spurious chord across north; C++ nested vectors.
	let mut parts: Vec<Vec<[f64; 2]>> = Vec::new();
	// Visit only the chosen time samples; C++ counted loop over a half-open interval.
	for i in range
	{
		// Normalize an exact 360 endpoint to north before checking wraps; C++ positive modulo.
		let point = [
			f64::from(track.az_deg[i].rem_euclid(360.0)),
			f64::from(track.el_deg[i]),
		];
		// Start a new line for the first point or a seam crossing; C++ optional last-element tests.
		if parts
			.last()
			.and_then(|p| p.last())
			.is_none_or(|last| (last[0] - point[0]).abs() > 180.0)
		{
			// Seed this independent piece, including isolated samples; C++ emplace_back(vector{point}).
			parts.push(vec![point]);
		}
		// The previous branch guarantees a final part exists; C++ else with mutable vector back().
		else if let Some(part) = parts.last_mut()
		{
			// Append only adjacent non-wrapping samples; C++ push_back.
			part.push(point);
		}
	}
	// Move all pieces to the plot caller; C++ return by value.
	parts
}

// Keep tests out of production binaries; C++ test-only compilation.
#[cfg(test)]
// Group helper regressions; C++ test namespace.
mod tests
{
	// Import private helpers; C++ test translation unit includes declarations.
	use super::*;
	// Register the regression; C++ test macro.
	#[test]
	// Cover normal, empty, forward and reverse wrap cases; C++ unit test.
	fn wraps_do_not_draw_chords()
	{
		// Cross north in both directions and include exact 360; C++ aggregate vectors.
		let track = Track {
			t: vec![0, 1, 2, 3, 4, 5],
			az_deg: vec![358.0, 359.0, 0.0, 1.0, 359.0, 360.0],
			el_deg: vec![0.0; 6],
		};
		// Expect independent pieces without losing points; C++ equality of transformed sizes.
		assert_eq!(
			segments(&track, 0..6)
				.iter()
				.map(Vec::len)
				.collect::<Vec<_>>(),
			[2, 2, 1, 1]
		);
		// An empty selection produces no polyline; C++ assert empty vector.
		assert!(segments(&track, 0..0).is_empty());
		// A single sample is safe even though it has no line span; C++ size comparison.
		assert_eq!(segments(&track, 2..3)[0].len(), 1);
	}
	// Register the regression; C++ test macro.
	#[test]
	// Check real time boundaries, clipping, and the whole-dataset sentinel; C++ unit test.
	fn windows_are_utc_intervals_not_sample_counts()
	{
		// Deliberately irregular times discriminate duration from index arithmetic; C++ vector<int64_t>.
		let t = [0, 3600, 7200, 100_000];
		// Centering 24h includes the first three but not a distant fourth; C++ interval equality.
		assert_eq!(window(&t, 1, 24), 0..3);
		// At the endpoint the window clips safely; C++ interval equality.
		assert_eq!(window(&t, 3, 24), 3..4);
		// All data must mean both endpoints, regardless of current index; C++ interval equality.
		assert_eq!(window(&t, 0, 0), 0..4);
		// One sample remains selectable and readable; C++ interval equality.
		assert_eq!(window(&[0], 0, 24), 0..1);
		// Verify UTC rendering at the Unix epoch; C++ formatted timestamp assertion.
		assert_eq!(utc_label(0), "1970-01-01 00:00 UTC");
	}
	// Register a summary regression independent of the bundled dataset; C++ test macro.
	#[test]
	fn summary_uses_full_duration_and_current_clearance()
	{
		// Unequal intervals distinguish duration weighting from point counting; C++ JSON fixture.
		let bytes=serde_json::to_vec(&serde_json::json!({"site":{"name":"test","lat_deg":-89.0,"lon_deg":0.0},"sun":{"t":[0,3600,86400],"az_deg":[0.0,0.0,0.0],"el_deg":[-1.0,1.0,1.0]},"earth":{"t":[0,3600,86400],"az_deg":[0.0,0.0,0.0],"el_deg":[0.75,0.75,0.75]},"horizon":null})).unwrap();
		// Use real validation and default-margin caching; C++ expected extraction.
		let mut data = SiteData::parse(&bytes).unwrap();
		// The last endpoint contributes no invented duration; C++ weighted percentage checks.
		let summary = availability(&data);
		assert!((summary.pct_sun - 100.0 * 23.0 / 24.0).abs() < 1e-9);
		assert_eq!(summary.longest_night_h, 1.0);
		assert_eq!(summary.pct_earth, 100.0);
		// Recalculation honors the edited margin for the entire period; C++ cache mutation.
		data.update_earth_margin(1.0);
		let blocked = availability(&data);
		assert_eq!(blocked.pct_earth, 0.0);
		assert_eq!(blocked.pct_both, 0.0);
		// Build a view model without starting asynchronous I/O or an OpenGL context; C++ test aggregate.
		let mut panorama = Panorama {
			pending: Pending::default(),
			data: Some(data),
			error: None,
			index: 0,
			hours: 24,
			earth_margin_deg: 1.0,
			technical: false,
			help: false,
			summary: blocked,
		};
		// At sample zero the Sun is below flat terrain and Earth fails the edited radio margin; C++ optional extraction.
		let first = panorama.directions().unwrap();
		// Preserve Sun/Earth ordering and the actual numeric sample; C++ numeric assertion.
		assert_eq!(first[0].el_deg, -1.0);
		// The ray flags must use the same predicates as the UI, not globe illumination; C++ boolean assertions.
		assert!(!first[0].visible && !first[1].visible);
		// Moving time must replace the exported direction rather than retaining the first sample; C++ index edit.
		panorama.index = 1;
		// Re-read the chosen sample through the production bridge; C++ optional extraction.
		let next = panorama.directions().unwrap();
		// The Sun has risen while Earth remains below the requested safety clearance; C++ scalar assertions.
		assert_eq!(next[0].el_deg, 1.0);
		// Visibility changes with the same selected index; C++ boolean assertion.
		assert!(next[0].visible && !next[1].visible);
		// Failed or missing loading state must not export stale scientific geometry; C++ optional reset.
		panorama.data = None;
		// Preserve an explicit loading error without inventing a sample; C++ error assignment.
		panorama.error = Some("fixture unavailable".into());
		// Absence propagates to Globe::set_directions and clears the previous pair; C++ nullopt assertion.
		assert!(panorama.directions().is_none());
	}
}
```

### 9.3. app/Cargo.toml

```toml
[package]
name = "app"
version = "0.1.0"
edition = "2024"

[dependencies]
globe = { path = "../globe" }
lunar_core = { path = "../core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", default-features = false, features = ["std"] }
eframe = { version = "0.37", default-features = false, features = ["default_fonts", "glow", "wayland", "x11"] }
egui_plot = "0.38"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = "0.11"

[target.'cfg(target_arch = "wasm32")'.dependencies]
ehttp = "0.7"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlCanvasElement"] }
```
