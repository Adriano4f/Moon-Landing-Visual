//! Validated Horizons data and asynchronous disk/HTTP loading, with no DEM policy.

use chrono::{DateTime, Utc};
use eframe::egui::{self, mutex::Mutex};
use lunar_core::{EARTH_MARGIN_DEG, Horizon, Site, Track, visible};
use serde::Deserialize;
use std::sync::Arc;

pub(crate) const FILE: &str = "connecting-ridge-c1.json";

#[derive(Deserialize)]
pub(crate) struct SiteData
{
	pub site: Site,
	pub sun: Track,
	pub earth: Track,
	/// Preserve null as absent terrain, not as an invented DEM.
	pub horizon: Option<Horizon>,
	/// This cache is computed after validation, not read from JSON.
	#[serde(skip)]
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

impl SiteData
{
	pub fn parse(bytes: &[u8]) -> Result<Self, String>
	{
		let mut data: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
		if data.site.name.trim().is_empty()
			|| !data.site.lat_deg.is_finite()
			|| !data.site.lon_deg.is_finite()
			|| !(-90.0..=90.0).contains(&data.site.lat_deg)
		{
			// Return a diagnostic instead of creating a misleading marker.
			return Err("Invalid site name or coordinates".into());
		}
		for track in [&data.sun, &data.earth]
		{
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
				// Avoid indexing invalid data or bridging missing arrays.
				return Err("Invalid or empty track arrays, timestamps, or angles".into());
			}
		}
		if data.sun.t != data.earth.t
		{
			return Err("Sun and Earth timestamps differ".into());
		}
		// Check supplied terrain only; no DEM is read or selected here.
		if let Some(h) = &data.horizon
			&& (h.h_deg.is_empty()
				|| h.h_deg
					.iter()
					.any(|e| !e.is_finite() || !(-90.0..=90.0).contains(e)))
		{
			// A supplied profile must actually contain valid bins.
			return Err("A supplied horizon needs nonempty finite elevation bins".into());
		}
		// Cache the default communication clearance, not mere center-above-horizon visibility.
		data.update_earth_margin(EARTH_MARGIN_DEG);
		Ok(data)
	}

	pub fn horizon_at(&self, az: f32) -> f32
	{
		// Do not use core's empty-profile -90 convention for missing DEM.
		self.horizon.as_ref().map_or(0.0, |h| h.at(az))
	}

	/// Recompute only when the user changes clearance.
	pub fn update_earth_margin(&mut self, margin_deg: f32)
	{
		// Retain the labeled flat fallback for older datasets.
		let flat = Horizon { h_deg: vec![0.0] };
		// Delegate the strict elevation > horizon + margin criterion to core.
		self.earth_above = visible(
			&self.earth,
			self.horizon.as_ref().unwrap_or(&flat),
			margin_deg,
		);
	}
}

pub(crate) type Pending = Arc<Mutex<Option<Result<SiteData, String>>>>;

pub(crate) fn load(ctx: egui::Context, file: String) -> Pending
{
	let pending = Arc::new(Mutex::new(None));
	let completed = Arc::clone(&pending);
	read_asset(file, move |bytes| {
		*completed.lock() = Some(bytes.and_then(|b| SiteData::parse(&b)));
		ctx.request_repaint();
	});
	pending
}

pub(crate) fn read_asset(file: String, done: impl FnOnce(Result<Vec<u8>, String>) + Send + 'static)
{
	#[cfg(not(target_arch = "wasm32"))]
	std::thread::spawn(move || {
		let directory = std::env::var_os("MOON_DATA_DIR")
			.map(std::path::PathBuf::from)
			.unwrap_or_else(|| {
				std::env::current_exe()
					.ok()
					.and_then(|p| p.parent().map(|p| p.join("assets/data")))
					.filter(|p| p.is_dir())
					.unwrap_or_else(|| {
						std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/data")
					})
			});
		let path = directory.join(&file);
		done(std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display())));
	});
	// Fetch the same relative resource copied by Trunk, without embedding it in WASM.
	#[cfg(target_arch = "wasm32")]
	ehttp::fetch(
		ehttp::Request::get(format!("assets/data/{file}")),
		move |response| {
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

#[cfg(test)]
mod tests
{
	use super::*;

	fn fixture() -> serde_json::Value
	{
		// Two aligned samples cross north, with Earth above flat zero.
		serde_json::json!({"site":{"name":"Test","lat_deg":-89.45,"lon_deg":222.69},
			"sun":{"t":[0,3600],"az_deg":[359,1],"el_deg":[0,1]},
			"earth":{"t":[0,3600],"az_deg":[137,138],"el_deg":[1,-1]},"horizon":null})
	}

	#[test]
	fn bundled_three_years_are_aligned_hourly_and_have_terrain()
	{
		// Embed only in the test executable, never in the app/WASM.
		let d = SiteData::parse(include_bytes!("../assets/data/connecting-ridge-c1.json")).unwrap();
		assert_eq!(d.sun.t.len(), 26305);
		assert!(d.sun.t.len() < 90_000);
		// Check UTC cadence, independently of sample count.
		assert!(d.sun.t.windows(2).all(|t| t[1] - t[0] == 3600));
		// Verify exact endpoint dates.
		assert_eq!(d.sun.t[0], 1_798_761_600);
		// Include the leap year 2028 through the 2030 endpoint.
		assert_eq!(*d.sun.t.last().unwrap(), 1_893_456_000);
		// Real terrain uses 1,440 geographic azimuths.
		assert_eq!(d.horizon.as_ref().unwrap().h_deg.len(), 1440);
		// Keep the selected position tied to the published 2 m C1 site, not a rounded label.
		assert_eq!((d.site.lat_deg, d.site.lon_deg), (-89.4398, 222.83));
	}

	#[test]
	fn flat_and_interpolated_terrain_visibility()
	{
		let mut v = fixture();
		let mut flat = SiteData::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
		// Earth centers use strict elevation > horizon.
		assert_eq!(flat.earth_above, [true, false]);
		// Raising the clearance to one degree rejects an Earth center exactly one degree high.
		flat.update_earth_margin(1.0);
		assert_eq!(flat.earth_above, [false, false]);
		flat.update_earth_margin(EARTH_MARGIN_DEG);
		// At the horizon half the solar disk is visible.
		assert!((lunar_core::sun_fraction(0.0, flat.horizon_at(359.0)) - 0.5).abs() < 1e-6);
		// Supply a synthetic cyclic profile, not a chosen DEM.
		v["horizon"] = serde_json::json!({"h_deg":[1,3]});
		let terrain = SiteData::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
		// Uniform bins interpolate at 90 degrees.
		assert!((terrain.horizon_at(90.0) - 2.0).abs() < 1e-6);
		assert!(terrain.earth_above.iter().all(|v| !v));
		// Values straddling half a degree prove parsing uses the default clearance, not zero.
		let mut close = fixture();
		close["earth"]["el_deg"] = serde_json::json!([0.25, 0.75]);
		let clearance = SiteData::parse(&serde_json::to_vec(&close).unwrap()).unwrap();
		assert_eq!(clearance.earth_above, [false, true]);
	}

	#[test]
	fn rejects_invalid_inputs()
	{
		// Empty input is not an empty successful dataset.
		assert!(SiteData::parse(b"").is_err());
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
			// Reset the fixture so errors do not mask one another.
			let mut v = fixture();
			*v.pointer_mut(pointer).unwrap() = replacement;
			assert!(
				SiteData::parse(&serde_json::to_vec(&v).unwrap()).is_err(),
				"{pointer}"
			);
		}
	}
}
