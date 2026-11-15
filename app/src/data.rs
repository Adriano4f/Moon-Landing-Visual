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
pub(crate) const FILE: &str = "connecting-ridge-c1.json";

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
	#[serde(default)]
	pub terrain: Option<Altitude>,
}

#[derive(Deserialize)]
pub(crate) struct Altitude
{
	pub surface_height_m:f64,
	pub observer_height_m:f64,
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
pub(crate) fn load(ctx: egui::Context, file: String) -> Pending
{
	// None means pending, distinct from success and failure; C++ optional starts disengaged.
	let pending = Arc::new(Mutex::new(None));
	let completed = Arc::clone(&pending);
	read_asset(file, move |bytes| {
		// Decode only on successful I/O, then publish the result; C++ expected::and_then under lock_guard.
		*completed.lock() = Some(bytes.and_then(|b| SiteData::parse(&b)));
		ctx.request_repaint();
	});
	pending
}

// Raw bytes of one shipped asset: disk on native, the Trunk copy over HTTP on the web.
pub(crate) fn read_asset(file: String, done: impl FnOnce(Result<Vec<u8>, String>) + Send + 'static)
{
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
		let path = directory.join(&file);
		done(std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display())));
	});
	// Fetch the same relative resource copied by Trunk, without embedding it in WASM; C++ asynchronous HTTP request.
	#[cfg(target_arch = "wasm32")]
	ehttp::fetch(
		ehttp::Request::get(format!("assets/data/{file}")),
		move |response| {
			// Reject HTTP errors even if their bodies happen to resemble JSON; C++ expected::and_then.
			done(response.and_then(|r| {
				if r.ok
				{
					Ok(r.bytes)
				}
				else
				{
					Err(format!("HTTP {} loading {file}", r.status))
				}
			}));
		},
	);
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
		// Enforce the independent regression expectation; C++ test assertion.
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
		// Bind this intermediate value for the current operation; C++ local initialization.
		let clearance = SiteData::parse(&serde_json::to_vec(&close).unwrap()).unwrap();
		// Enforce the independent regression expectation; C++ test assertion.
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
