//! Any-point job: fetch Trek terrain, integrate the horizon over several frames, then build site data.

use crate::data::{Altitude, Pending, SiteData};
use crate::timeline;
use eframe::egui::{self, mutex::Mutex};
use lunar_core::anypoint::{self, HorizonJob, Tile};
use lunar_core::ephemeris::{Ephemeris, topocentric};
use lunar_core::{Site, Track};
use std::sync::Arc;

// One asynchronous byte download; None while in flight.
type Slot = Arc<Mutex<Option<Result<Vec<u8>, String>>>>;

// Same three-year UTC span as the published sites, so summaries are comparable.
const START_UTC: i64 = 1_798_761_600;
const END_UTC: i64 = 1_893_456_000;
// Instrument height above terrain, as for the published sites.
const MAST_M: f64 = 2.0;
// Bins integrated per frame; small enough to keep the UI responsive.
const BINS_PER_FRAME: usize = 12;

enum Stage
{
	Fetching
	{
		terrain: Vec<Slot>,
	},
	Computing
	{
		job: Box<HorizonJob>,
		ephemeris: Arc<Ephemeris>,
	},
	Finished,
}

pub(crate) struct Custom
{
	pub tile: Tile,
	pub name: String,
	pub pending: Pending,
	timeline: timeline::Shared,
	stage: Stage,
}

fn slot() -> Slot
{
	Arc::new(Mutex::new(None))
}

// Fetch one URL and wake the UI on completion; works natively and in the browser.
fn fetch(ctx: &egui::Context, url: String) -> Slot
{
	let result = slot();
	let done = Arc::clone(&result);
	let ctx = ctx.clone();
	ehttp::fetch(ehttp::Request::get(&url), move |response| {
		*done.lock() = Some(response.and_then(|r| {
			if r.ok
			{
				Ok(r.bytes)
			}
			else
			{
				Err(format!("Terrain server returned HTTP {}", r.status))
			}
		}));
		ctx.request_repaint();
	});
	result
}

impl Custom
{
	pub fn start(ctx: &egui::Context, lat: f64, lon: f64, timeline: timeline::Shared) -> Self
	{
		let tile = anypoint::tile_of(lat, lon);
		let terrain = anypoint::terrain_requests(tile.lat_deg, tile.lon_deg)
			.into_iter()
			.map(|url| fetch(ctx, url))
			.collect();
		timeline::request(&timeline, ctx);
		let lon_signed = if tile.lon_deg > 180.0
		{
			tile.lon_deg - 360.0
		}
		else
		{
			tile.lon_deg
		};
		Self {
			name: format!("Custom point ({:.2}, {:.2})", tile.lat_deg, lon_signed),
			tile,
			pending: Arc::new(Mutex::new(None)),
			timeline,
			stage: Stage::Fetching { terrain },
		}
	}

	// Plain-language progress while the job runs; None once finished.
	pub fn status(&self) -> Option<String>
	{
		match &self.stage
		{
			Stage::Fetching { terrain, .. } =>
			{
				let ready = terrain.iter().filter(|s| s.lock().is_some()).count();
				Some(format!(
					"Downloading NASA terrain around this tile ({ready}/{})...",
					terrain.len()
				))
			}
			Stage::Computing { job, .. } => Some(format!(
				"Computing the horizon: {:.0}%",
				job.progress() * 100.0
			)),
			Stage::Finished => None,
		}
	}

	// Advance at most one frame's worth of work.
	pub fn step(&mut self, ctx: &egui::Context)
	{
		if let Err(error) = self.advance()
		{
			*self.pending.lock() = Some(Err(error));
			self.stage = Stage::Finished;
		}
		if !matches!(self.stage, Stage::Finished)
		{
			ctx.request_repaint();
		}
	}

	fn advance(&mut self) -> Result<(), String>
	{
		match &mut self.stage
		{
			Stage::Fetching { terrain } =>
			{
				let ready = |s: &Slot| s.lock().is_some();
				let Some(ephemeris) = self.timeline.lock().sky.clone()
				else
				{
					return Ok(());
				};
				if !terrain.iter().all(ready)
				{
					return Ok(());
				}
				let take = |s: &Slot| s.lock().take().unwrap();
				let tiffs = terrain.iter().map(take).collect::<Result<Vec<_>, _>>()?;
				let ephemeris = ephemeris?;
				let job = HorizonJob::new(
					self.tile.lat_deg,
					self.tile.lon_deg,
					anypoint::N_AZ,
					MAST_M,
					&tiffs,
				)?;
				self.stage = Stage::Computing {
					job: Box::new(job),
					ephemeris,
				};
			}
			Stage::Computing { job, ephemeris } =>
			{
				if let Some(horizon) = job.step(BINS_PER_FRAME)?
				{
					let height = job.surface_m() + MAST_M;
					let (lat, lon) = (self.tile.lat_deg, self.tile.lon_deg);
					let mut sun = Track::default();
					let mut earth = Track::default();
					for (i, pair) in ephemeris.samples.iter().enumerate()
					{
						let t = ephemeris.epoch(i).ok_or("Ephemeris epoch overflow")?;
						if !(START_UTC..=END_UTC).contains(&t)
						{
							continue;
						}
						for (track, body) in [(&mut sun, pair[0]), (&mut earth, pair[1])]
						{
							let (az, el) = topocentric(body, lat, lon, height)?;
							track.t.push(t);
							track.az_deg.push(az as f32);
							track.el_deg.push(el as f32);
						}
					}
					if sun.t.is_empty()
					{
						return Err("Ephemeris does not cover 2027 to 2030".into());
					}
					let mut data = SiteData {
						site: Site {
							name: self.name.clone(),
							lat_deg: lat,
							lon_deg: lon,
						},
						sun,
						earth,
						horizon: Some(horizon),
						earth_above: vec![],
						terrain: Some(Altitude {
							surface_height_m: job.surface_m(),
							observer_height_m: MAST_M,
						}),
					};
					data.update_earth_margin(lunar_core::EARTH_MARGIN_DEG);
					*self.pending.lock() = Some(Ok(data));
					self.stage = Stage::Finished;
				}
			}
			Stage::Finished =>
			{}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn shared_sky_loading_and_errors_reach_the_custom_point()
	{
		let timeline = Arc::new(Mutex::new(timeline::Timeline::default()));
		let mut custom = Custom {
			tile: anypoint::tile_of(0.0, 0.0),
			name: "Custom point".into(),
			pending: Arc::new(Mutex::new(None)),
			timeline: Arc::clone(&timeline),
			stage: Stage::Fetching { terrain: vec![] },
		};
		let ctx = egui::Context::default();
		custom.step(&ctx);
		assert!(custom.pending.lock().is_none());
		assert!(custom.status().is_some());
		timeline.lock().sky = Some(Err("injected unavailable sky".into()));
		custom.step(&ctx);
		assert!(
			matches!(custom.pending.lock().take(),Some(Err(error)) if error=="injected unavailable sky")
		);
		assert!(custom.status().is_none());
	}
}
