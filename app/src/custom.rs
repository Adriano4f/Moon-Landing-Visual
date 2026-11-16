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
/// Give up after this many seconds without another terrain response.
const TIMEOUT_S: f64 = 90.0;

enum Stage
{
	Fetching
	{
		terrain: Vec<Slot>,
		received: usize,
		last_response: f64,
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
	elapsed: f64,
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
	// The job owns the inactivity deadline across all terrain requests.
	request.timeout = None;
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
	format!("Custom point ({})", crate::coordinates(lat, lon))
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
			elapsed: 0.0,
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
			elapsed: 0.0,
			stage: Stage::Fetching { terrain, received: 0, last_response: 0.0 },
		}
	}

	pub fn status(&self) -> Option<String>
	{
		match &self.stage
		{
			Stage::Fetching { terrain, .. } =>
			{
				let ready = terrain.iter().filter(|s| s.lock().is_some()).count();
				if ready == terrain.len()
				{
					return Some("Waiting for shared sky data...".into());
				}
				Some(format!(
					"Downloading NASA terrain around this point ({ready}/{}), {:.0} s...",
					terrain.len(), self.elapsed
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
		self.elapsed = elapsed;
		match &mut self.stage
		{
			Stage::Fetching { terrain, received, last_response } =>
			{
				let ready = terrain.iter().filter(|s| s.lock().is_some()).count();
				if ready > *received
				{
					*received = ready;
					*last_response = elapsed;
				}
				for response in terrain.iter()
				{
					if let Some(Err(error)) = &*response.lock()
					{
						return Err(error.clone());
					}
				}
				if ready < terrain.len()
				{
					if elapsed - *last_response >= TIMEOUT_S
					{
						return Err("No NASA terrain response for 90 s. Click the point again to retry.".into());
					}
					return Ok(());
				}
				let Some(ephemeris) = self.timeline.lock().sky.clone()
				else
				{
					return Ok(());
				};
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

	fn fetching(terrain: Vec<Slot>) -> Custom
	{
		let timeline = Arc::new(Mutex::new(timeline::Timeline::default()));
		let mut custom = Custom::restored([0.0, 0.0], timeline);
		custom.stage = Stage::Fetching { terrain, received: 0, last_response: 0.0 };
		custom
	}

	#[test]
	fn terrain_timeout_resets_on_each_response()
	{
		let first = slot();
		let mut custom = fetching(vec![Arc::clone(&first), slot()]);
		assert!(custom.advance(80.0).is_ok());
		*first.lock() = Some(Ok(vec![]));
		assert!(custom.advance(80.0).is_ok());
		assert_eq!(custom.status().unwrap(), "Downloading NASA terrain around this point (1/2), 80 s...");
		assert!(custom.advance(169.0).is_ok());
		assert_eq!(custom.advance(170.0).unwrap_err(),
			"No NASA terrain response for 90 s. Click the point again to retry.");
	}

	#[test]
	fn terrain_deadline_is_independent_of_shared_sky()
	{
		let mut pending = fetching(vec![slot()]);
		assert!(pending.advance(89.0).is_ok());
		assert!(pending.advance(90.0).is_err());
		for terrain in [vec![], vec![Arc::new(Mutex::new(Some(Ok(vec![]))))]]
		{
			let mut custom = fetching(terrain);
			assert!(custom.advance(1.0).is_ok());
			assert!(custom.advance(500.0).is_ok());
			assert_eq!(custom.status().as_deref(), Some("Waiting for shared sky data..."));
			custom.timeline.lock().sky = Some(Err("sky unavailable".into()));
			assert_eq!(custom.advance(501.0).unwrap_err(), "sky unavailable");
		}
		let mut failed = fetching(vec![Arc::new(Mutex::new(Some(Err("terrain unavailable".into()))))]);
		assert_eq!(failed.advance(1.0).unwrap_err(), "terrain unavailable");
	}

	#[test]
	fn exact_points_and_failed_retries()
	{
		let point = [15.123456789, 356.896551724];
		let timeline = Arc::new(Mutex::new(timeline::Timeline::default()));
		let mut custom = Custom::restored(point, timeline);
		assert_eq!(custom.point, point);
		assert_eq!(custom.name, "Custom point (15.12°, -3.10°)");
		assert!(custom.can_reuse(point));
		assert!(!custom.can_reuse([point[0] + 0.000001, point[1]]));
		assert_eq!(name([-0.12345, 10.98765]), "Custom point (-0.12°, 10.99°)");
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
			elapsed: 0.0,
			stage: Stage::Fetching { terrain: vec![], received: 0, last_response: 0.0 },
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
