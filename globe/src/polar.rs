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
