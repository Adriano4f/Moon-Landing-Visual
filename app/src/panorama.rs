//! Immediate-mode panorama controls over validated, hourly Horizons samples.

mod view;
mod timeline;

use crate::data::{self, Pending, SiteData};
use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32};
use egui_plot::{GridMark, HLine, Legend, Line, Plot, Points};
use lunar_core::{EARTH_MARGIN_DEG, Track, sun_fraction};
use std::ops::Range;

/// Own only persistent choices and data, not a retained widget tree.
pub(crate) struct Panorama
{
	pending: Pending,
	data: Option<SiteData>,
	error: Option<String>,
	index: usize,
	/// Centered window length in hours; zero means the entire dataset.
	hours: i64,
	earth_margin_deg: f32,
	technical: bool,
	/// Cache selected-period values rather than recomputing every animated frame.
	summary: lunar_core::Stats,
	timeline: crate::timeline::Shared,
	cursor:i64,
	summary_key:Option<(crate::timeline::Options,f32)>,
	sky_error:Option<String>,
}

impl Panorama
{
	pub fn new(ctx: egui::Context, file: String, timeline:crate::timeline::Shared) -> Self
	{
		Self::with_pending(data::load(ctx, file), timeline)
	}

	/// Wrap a result slot filled by another producer, such as a custom-point job.
	pub fn with_pending(pending: Pending, timeline:crate::timeline::Shared) -> Self
	{
		#[cfg(not(target_arch = "wasm32"))]
		let index = std::env::var("MOON_SAMPLE_INDEX")
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(0);
		#[cfg(target_arch = "wasm32")]
		let index = 0;
		Self {
			pending,
			data: None,
			error: None,
			index,
			hours: 720,
			earth_margin_deg: EARTH_MARGIN_DEG,
			technical: false,
			summary: lunar_core::Stats::default(),
			timeline,
			cursor:crate::timeline::START,
			summary_key:None,
			sky_error:None,
		}
	}

	pub fn update(&mut self, ctx:&egui::Context)
	{
		// Release the completion-slot mutex before updating the view model.
		let result = self.pending.lock().take();
		match result
		{
			Some(Ok(mut data)) =>
			{
				data.update_earth_margin(self.earth_margin_deg);
				// A requested initial index must fit the data that actually arrived.
				self.index = self.index.min(data.sun.t.len() - 1);
				self.cursor=data.sun.t[self.index];
				self.summary_key=None;
				self.data = Some(data);
			}
			// Keep errors visible instead of retrying every animation frame.
			Some(Err(error)) => self.error = Some(error),
			None =>
			{}
		}
		self.extend_sky(ctx);
		self.refresh_summary();
	}

	/// Return coordinates without exposing scientific data to globe.
	pub fn site(&self) -> Option<(f64, f64)>
	{
		self.data.as_ref().map(|d| (d.site.lat_deg, d.site.lon_deg))
	}

	/// Export one UTC epoch for every site's own track lookup.
	pub fn time(&self) -> Option<i64>
	{
		self.data.as_ref().map(|_|self.cursor)
	}

	pub fn set_time(&mut self, time: i64)
	{
		let period=self.timeline.lock().options.period;
		self.cursor=time.clamp(period.start,period.end);
		// Loaded tracks have strictly ordered epochs.
		if let Some(data) = &self.data
		{
			self.index = data
				.sun
				.t
				.partition_point(|&t| t <= self.cursor)
				.saturating_sub(1)
				.min(data.sun.t.len() - 1);
		}
	}

	/// Evaluate another site's own current sample, never reuse the selected site's angles.
	pub fn directions_at(&mut self, time: i64, margin: f32) -> Option<[globe::BodyDirection; 2]>
	{
		let data=self.data.as_ref()?;
		if time<data.sun.t[0] || time>*data.sun.t.last()? {return None;}
		// All-sites radio styling must use the currently selected safety clearance.
		if self.earth_margin_deg != margin
		{
			self.earth_margin_deg = margin;
			if let Some(data) = &mut self.data
			{
				data.update_earth_margin(margin);
				self.summary_key=None;
			}
		}
		// Do not silently extrapolate beyond a site's coverage.
		self.set_time(time);
		self.directions()
	}

	/// Share the radio setting without exposing arrays to the renderer.
	pub fn margin(&self) -> f32
	{
		self.earth_margin_deg
	}

	pub fn load_error(&self) -> Option<&str>
	{
		self.error.as_deref()
	}

	/// Copy only the current scientific directions/availability into the independent renderer.
	pub fn directions(&self) -> Option<[globe::BodyDirection; 2]>
	{
		let data=self.data.as_ref()?;
		let [a,e]=crate::timeline::sample(&data.sun,self.cursor)?;
		let sun=globe::BodyDirection { az_deg:a,el_deg:e,visible:sun_fraction(e,data.horizon_at(a))>0.0 };
		let [a,e]=crate::timeline::sample(&data.earth,self.cursor)?;
		Some([sun,globe::BodyDirection { az_deg:a,el_deg:e,visible:e>data.horizon_at(a)+self.earth_margin_deg }])
	}

	/// Show time settings in the grouped menu, not over the panorama.
	pub fn time_controls(&mut self, ui: &mut egui::Ui)
	{
		if self.data.is_none() { ui.label("Load a site to choose time."); return; }
		let mut cursor=self.cursor;
		ui.scope(|ui| {
			ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(65));
			if ui.ctx().content_rect().width() >= 700.0
			{
				ui.spacing_mut().interact_size = egui::vec2(24.0, 24.0);
				ui.spacing_mut().button_padding = egui::vec2(6.0, 3.0);
				ui.spacing_mut().item_spacing = egui::vec2(6.0, 4.0);
			}
			self.timeline.lock().controls(ui, &mut cursor);
		});
		self.set_time(cursor);
	}

	pub(crate) fn paths_controls(&mut self, ui:&mut egui::Ui)
	{
		ui.horizontal(|ui| {
			ui.small("Paths");
			// Explain that the window controls faint paths, not summary percentages.
			egui::ComboBox::from_id_salt("time_window")
				.selected_text(window_label(self.hours))
				.show_ui(ui, |ui| {
					// Use durations rather than arbitrary point counts.
					for (h, label) in [
						(24, "1 day"),
						(168, "7 days"),
						(720, "30 days"),
						(0, "All data"),
					]
					{
						ui.selectable_value(&mut self.hours, h, label);
					}
				});
		});
	}
	pub fn radio_controls(&mut self, ui: &mut egui::Ui)
	{
		// Preserve an explicit pending/error state without indexing absent samples.
		let Some(data) = &mut self.data
		else
		{
			ui.label("Load a site to adjust clearance.");
			return;
		};
		ui.label("Earth safety margin");
		// Reserve room for the editable value and units as well as the slider.
		ui.spacing_mut().slider_width = (ui.available_width() - 125.0).max(60.0);
		// Recompute full-period percentages only when the clearance changes.
		if ui
			.add(
				egui::Slider::new(&mut self.earth_margin_deg, 0.5..=1.0)
					.suffix(" degrees")
					.fixed_decimals(2),
			)
			.changed()
		{
			data.update_earth_margin(self.earth_margin_deg);
			// Reuse core's duration-weighted statistics.
			self.summary_key=None;
		}

		globe::help::question(
			ui,
			"Earth must be at least this many degrees above the hills to count as linked.",
		);
	}
	pub fn technical_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			ui.checkbox(&mut self.technical, "Technical view");
			globe::help::question(ui, "Show the same sky as a graph of direction from north against height.");
		});
	}
}

#[cfg(test)]
fn availability(data: &SiteData) -> lunar_core::Stats
{
	let sun: Vec<_> = data
		.sun
		.az_deg
		.iter()
		.zip(&data.sun.el_deg)
		.map(|(&a, &e)| sun_fraction(e, data.horizon_at(a)) > 0.0)
		.collect();
	lunar_core::stats(&data.sun.t, &sun, &data.earth_above)
}

/// Format a UTC epoch without consulting a platform clock.
pub(crate) fn utc_label(t: i64) -> String
{
	// Defensive fallback is unreachable after validation, but formatting never panics.
	DateTime::<Utc>::from_timestamp(t, 0).map_or_else(
		|| "Invalid UTC".into(),
		|d| d.format("%Y-%m-%d %H:%M UTC").to_string(),
	)
}

fn window_label(hours: i64) -> &'static str
{
	match hours
	{
		24 => "1 day",
		168 => "7 days",
		720 => "30 days",
		_ => "All data",
	}
}

/// Find a centered, clipped interval; precondition: validated nonempty sorted times and a valid index.
fn window(times: &[i64], index: usize, hours: i64) -> Range<usize>
{
	// Zero is an explicit entire-dataset choice, not a zero-duration plot.
	if hours == 0
	{
		return 0..times.len();
	}
	let half = hours * 1800;
	times.partition_point(|&t| t < times[index] - half)
		..times.partition_point(|&t| t <= times[index] + half)
}

/// Split at a >180-degree jump, preserving time order; precondition: validated arrays/range.
pub(crate) fn segments(track: &Track, range: Range<usize>) -> Vec<Vec<[f64; 2]>>
{
	// Own separate polylines rather than drawing a spurious chord across north.
	let mut parts: Vec<Vec<[f64; 2]>> = Vec::new();
	for i in range
	{
		// Normalize an exact 360 endpoint to north before checking wraps.
		let point = [
			f64::from(track.az_deg[i].rem_euclid(360.0)),
			f64::from(track.el_deg[i]),
		];
		// Start a new line for the first point or a seam crossing.
		if parts
			.last()
			.and_then(|p| p.last())
			.is_none_or(|last| (last[0] - point[0]).abs() > 180.0)
		{
			parts.push(vec![point]);
		}
		// The previous branch guarantees a final part exists.
		else if let Some(part) = parts.last_mut()
		{
			// Append only adjacent non-wrapping samples.
			part.push(point);
		}
	}
	parts
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn wraps_do_not_draw_chords()
	{
		// Cross north in both directions and include exact 360.
		let track = Track {
			t: vec![0, 1, 2, 3, 4, 5],
			az_deg: vec![358.0, 359.0, 0.0, 1.0, 359.0, 360.0],
			el_deg: vec![0.0; 6],
		};
		assert_eq!(
			segments(&track, 0..6)
				.iter()
				.map(Vec::len)
				.collect::<Vec<_>>(),
			[2, 2, 1, 1]
		);
		assert!(segments(&track, 0..0).is_empty());
		assert_eq!(segments(&track, 2..3)[0].len(), 1);
	}
	#[test]
	fn windows_are_utc_intervals_not_sample_counts()
	{
		// Deliberately irregular times discriminate duration from index arithmetic.
		let t = [0, 3600, 7200, 100_000];
		// Centering 24h includes the first three but not a distant fourth.
		assert_eq!(window(&t, 1, 24), 0..3);
		assert_eq!(window(&t, 3, 24), 3..4);
		// All data must mean both endpoints, regardless of current index.
		assert_eq!(window(&t, 0, 0), 0..4);
		assert_eq!(window(&[0], 0, 24), 0..1);
		assert_eq!(utc_label(0), "1970-01-01 00:00 UTC");
	}
	#[test]
	fn summary_uses_full_duration_and_current_clearance()
	{
		// Unequal intervals distinguish duration weighting from point counting.
		let bytes=serde_json::to_vec(&serde_json::json!({"site":{"name":"test","lat_deg":-89.0,"lon_deg":0.0},"sun":{"t":[0,3600,86400],"az_deg":[0.0,0.0,0.0],"el_deg":[-1.0,1.0,1.0]},"earth":{"t":[0,3600,86400],"az_deg":[0.0,0.0,0.0],"el_deg":[0.75,0.75,0.75]},"horizon":null})).unwrap();
		let mut data = SiteData::parse(&bytes).unwrap();
		// The last endpoint contributes no invented duration.
		let summary = availability(&data);
		assert!((summary.pct_sun - 100.0 * 23.0 / 24.0).abs() < 1e-9);
		assert_eq!(summary.longest_night_h, 1.0);
		assert_eq!(summary.pct_earth, 100.0);
		data.update_earth_margin(1.0);
		let blocked = availability(&data);
		assert_eq!(blocked.pct_earth, 0.0);
		assert_eq!(blocked.pct_both, 0.0);
		// Build a view model without starting asynchronous I/O or an OpenGL context.
		let mut panorama = Panorama {
			pending: Pending::default(),
			data: Some(data),
			error: None,
			index: 0,
			hours: 24,
			earth_margin_deg: 1.0,
			technical: false,
			summary: blocked,
			timeline:std::sync::Arc::new(egui::mutex::Mutex::new(crate::timeline::Timeline::default())),
			cursor:0,
			summary_key:None,
			sky_error:None,
		};
		panorama.timeline.lock().options.period=crate::timeline::Period {start:0,end:86400};
		// At sample zero the Sun is below flat terrain and Earth fails the edited radio margin.
		let first = panorama.directions().unwrap();
		assert_eq!(first[0].el_deg, -1.0);
		// The ray flags must use the same predicates as the UI, not globe illumination.
		assert!(!first[0].visible && !first[1].visible);
		// Moving time must replace the exported direction rather than retaining the first sample.
		panorama.set_time(3600);
		let next = panorama.directions().unwrap();
		// The Sun has risen while Earth remains below the requested safety clearance.
		assert_eq!(next[0].el_deg, 1.0);
		assert!(next[0].visible && !next[1].visible);
		// Each background site's own track is sampled at shared UTC, with no extrapolation.
		assert!(panorama.directions_at(-1, 0.5).is_none());
		assert!(panorama.directions_at(0, 0.5).unwrap()[1].visible);
		assert_eq!(panorama.time(), Some(0));
		assert!(panorama.directions_at(i64::MAX, 0.5).is_none());
		assert!(panorama.directions_at(3600, 1.0).unwrap()[0].visible);
		// Failed or missing loading state must not export stale scientific geometry.
		panorama.data = None;
		// Preserve an explicit loading error without inventing a sample.
		panorama.error = Some("fixture unavailable".into());
		assert!(panorama.directions().is_none());
	}
}
