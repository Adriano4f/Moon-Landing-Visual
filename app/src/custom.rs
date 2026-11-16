//! Any-point job: fetch Trek terrain, integrate the horizon over several frames, then build site data.

use crate::data::{Altitude, Pending, SiteData};
use crate::timeline;
use eframe::egui::{self, mutex::Mutex};
use lunar_core::anypoint::{self, HorizonJob};
use lunar_core::ephemeris::{Ephemeris, topocentric};
use lunar_core::{Horizon, Site, Track};
use std::sync::Arc;

type Slot = Arc<Mutex<Option<Result<Vec<u8>, String>>>>;

/// Same three-year UTC span as the published sites, so summaries are comparable.
const START_UTC: i64 = 1_798_761_600;
const END_UTC: i64 = 1_893_456_000;
/// Instrument height above terrain, as for the published sites.
const MAST_M: f64 = 2.0;
const BINS_PER_FRAME: usize = 4;
/// Horizon work per frame; keeps the UI smooth while using most of the frame.
const FRAME_BUDGET: std::time::Duration = std::time::Duration::from_millis(12);
/// Give up on NASA downloads after this many seconds.
const TIMEOUT_S: f64 = 30.0;

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
	pub point: [f64; 2],
	pub name: String,
	pub pending: Pending,
	timeline: timeline::Shared,
	/// Set when the job ended in an error, so a new click retries.
	pub failed: bool,
	started: f64,
	stage: Stage,
}

fn slot() -> Slot
{
	Arc::new(Mutex::new(None))
}

fn fetch(ctx: &egui::Context, url: String) -> Slot
{
	let result = slot();
	let done = Arc::clone(&result);
	let ctx = ctx.clone();
	let mut request = ehttp::Request::get(&url);
	request.timeout = Some(std::time::Duration::from_secs_f64(TIMEOUT_S));
	ehttp::fetch(request, move |response| {
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

/// Display label with longitude in -180..180.
fn name([lat, lon]: [f64; 2]) -> String
{
	let lon = if lon > 180.0 { lon - 360.0 } else { lon };
	format!("Custom point ({lat:.2}, {lon:.2})")
}

impl Custom
{
	pub fn can_reuse(&self, point: [f64; 2]) -> bool
	{
		self.point == point && !self.failed
	}

	pub fn restored(point: [f64; 2], timeline: timeline::Shared) -> Self
	{
		Self {
			name: name(point),
			point,
			pending: Arc::new(Mutex::new(None)),
			timeline,
			failed: false,
			started: 0.0,
			stage: Stage::Finished,
		}
	}

	pub fn start(ctx: &egui::Context, lat: f64, lon: f64, timeline: timeline::Shared) -> Self
	{
		let terrain = anypoint::terrain_requests(lat, lon)
			.into_iter()
			.map(|url| fetch(ctx, url))
			.collect();
		timeline::request(&timeline, ctx);
		Self {
			name: name([lat, lon]),
			point: [lat, lon],
			pending: Arc::new(Mutex::new(None)),
			timeline,
			failed: false,
			started: ctx.input(|i| i.time),
			stage: Stage::Fetching { terrain },
		}
	}

	pub fn status(&self) -> Option<String>
	{
		match &self.stage
		{
			Stage::Fetching { terrain, .. } =>
			{
				let ready = terrain.iter().filter(|s| s.lock().is_some()).count();
				Some(format!(
					"Downloading NASA terrain around this point ({ready}/{})...",
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

	pub fn step(&mut self, ctx: &egui::Context)
	{
		let elapsed = ctx.input(|i| i.time) - self.started;
		if let Err(error) = self.advance(elapsed)
		{
			self.failed = true;
			*self.pending.lock() = Some(Err(error));
			self.stage = Stage::Finished;
		}
		if !matches!(self.stage, Stage::Finished)
		{
			ctx.request_repaint();
		}
	}

	fn advance(&mut self, elapsed: f64) -> Result<(), String>
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
					if elapsed > TIMEOUT_S
					{
						return Err("NASA terrain server did not answer in 30 s. Click the point again to retry.".into());
					}
					return Ok(());
				}
				let take = |s: &Slot| s.lock().take().unwrap();
				let tiffs = terrain.iter().map(take).collect::<Result<Vec<_>, _>>()?;
				let ephemeris = ephemeris?;
				let job =
					HorizonJob::new(self.point[0], self.point[1], anypoint::N_AZ, MAST_M, &tiffs)?;
				self.stage = Stage::Computing {
					job: Box::new(job),
					ephemeris,
				};
			}
			Stage::Computing { job, ephemeris } =>
			{
				if let Some(horizon) = step_for(job, FRAME_BUDGET)?
				{
					let height = job.surface_m() + MAST_M;
					let [lat, lon] = self.point;
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

/// Run horizon bins until the job ends or the frame budget is used.
fn step_for(job: &mut HorizonJob, budget: std::time::Duration) -> Result<Option<Horizon>, String>
{
	let start = web_time::Instant::now();
	loop
	{
		if let Some(horizon) = job.step(BINS_PER_FRAME)?
		{
			return Ok(Some(horizon));
		}
		if start.elapsed() >= budget
		{
			return Ok(None);
		}
	}
}

pub(crate) struct Cached
{
	pub point: [f64; 2],
	pub view: crate::panorama::Panorama,
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn exact_points_and_failed_retries()
	{
		let point = [15.123456789, 356.896551724];
		let timeline = Arc::new(Mutex::new(timeline::Timeline::default()));
		let mut custom = Custom::restored(point, timeline);
		assert_eq!(custom.point, point);
		assert_eq!(custom.name, "Custom point (15.12, -3.10)");
		assert!(custom.can_reuse(point));
		assert!(!custom.can_reuse([point[0] + 0.000001, point[1]]));
		assert_eq!(name([-0.12345, 10.98765]), "Custom point (-0.12, 10.99)");
		custom.failed = true;
		assert!(!custom.can_reuse(point));
	}

	#[test]
	fn shared_sky_loading_and_errors_reach_the_custom_point()
	{
		let timeline = Arc::new(Mutex::new(timeline::Timeline::default()));
		let mut custom = Custom {
			point: [0.0, 0.0],
			name: "Custom point".into(),
			pending: Arc::new(Mutex::new(None)),
			timeline: Arc::clone(&timeline),
			failed: false,
			started: 0.0,
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
