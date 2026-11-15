//! Immediate-mode panorama controls over validated, hourly Horizons samples.

mod view;
mod timeline;

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
	// Nearest preceding sample for the path window; C++ size_t.
	index: usize,
	// Centered window length in hours; zero means the entire dataset; C++ int64_t.
	hours: i64,
	// User-selected angular clearance for a direct-to-Earth link; C++ float.
	earth_margin_deg: f32,
	// Remember the optional numeric view; C++ bool.
	technical: bool,
	// Cache selected-period values rather than recomputing every animated frame; C++ value member.
	summary: lunar_core::Stats,
	timeline: crate::timeline::Shared,
	cursor:i64,
	summary_key:Option<(crate::timeline::Options,f32)>,
	sky_error:Option<String>,
}

// Define view-model member functions; C++ out-of-class member definitions.
impl Panorama
{
	// Start loading once during application construction; C++ a constructor with an asynchronous operation.
	pub fn new(ctx: egui::Context, file: String, timeline:crate::timeline::Shared) -> Self
	{
		Self::with_pending(data::load(ctx, file), timeline)
	}

	// Wrap a result slot filled by another producer, such as a custom-point job.
	pub fn with_pending(pending: Pending, timeline:crate::timeline::Shared) -> Self
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
			pending,
			data: None,
			error: None,
			index,
			hours: 720,
			// Default to the user-selected half-degree communication margin; C++ scalar initializer.
			earth_margin_deg: EARTH_MARGIN_DEG,
			// Start in the nontechnical view without an obstructing dialog; C++ value initialization.
			technical: false,
			summary: lunar_core::Stats::default(),
			timeline,
			cursor:crate::timeline::START,
			summary_key:None,
			sky_error:None,
		}
	}

	// Consume a completion at most once, before drawing; C++ moves out of an optional under a mutex.
	pub fn update(&mut self, ctx:&egui::Context)
	{
		// Drop the short-lived mutex guard before changing the view model; C++ scoped lock_guard.
		let result = self.pending.lock().take();
		// Exhaustively separate pending, success and failure; C++ switch over optional<expected> alternatives.
		match result
		{
			// Move valid data into UI ownership; C++ move assignment.
			Some(Ok(mut data)) =>
			{
				// Pending nonselected loads may already have a nondefault radio setting; C++ cache restoration.
				data.update_earth_margin(self.earth_margin_deg);
				// A requested initial index must fit the data that actually arrived; C++ clamped index.
				self.index = self.index.min(data.sun.t.len() - 1);
				// Compute only after validation; C++ cached statistics.
				self.cursor=data.sun.t[self.index];
				self.summary_key=None;
				// Move ownership into the view; C++ move assignment.
				self.data = Some(data);
			}
			// Keep errors visible instead of retrying every animation frame; C++ move assignment.
			Some(Err(error)) => self.error = Some(error),
			// Pending or already consumed requires no work; C++ empty branch.
			None =>
			{}
		}
		self.extend_sky(ctx);
		self.refresh_summary();
	}

	// Return coordinates without exposing scientific data to globe; C++ optional<pair<double,double>>.
	pub fn site(&self) -> Option<(f64, f64)>
	{
		// Borrow the model and copy two scalars; C++ optional transform of a const reference.
		self.data.as_ref().map(|d| (d.site.lat_deg, d.site.lon_deg))
	}

	// Export one UTC epoch for every site's own track lookup; C++ optional<int64_t>.
	pub fn time(&self) -> Option<i64>
	{
		self.data.as_ref().map(|_|self.cursor)
	}

	// Restore shared time when switching cached site views; C++ checked binary search.
	pub fn set_time(&mut self, time: i64)
	{
		let period=self.timeline.lock().options.period;
		self.cursor=time.clamp(period.start,period.end);
		// Loaded tracks have strictly ordered epochs; C++ optional guard.
		if let Some(data) = &self.data
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.index = data
				.sun
				.t
				.partition_point(|&t| t <= self.cursor)
				.saturating_sub(1)
				.min(data.sun.t.len() - 1);
		}
	}

	// Evaluate another site's own current sample, never reuse the selected site's angles; C++ optional array.
	pub fn directions_at(&mut self, time: i64, margin: f32) -> Option<[globe::BodyDirection; 2]>
	{
		let data=self.data.as_ref()?;
		if time<data.sun.t[0] || time>*data.sun.t.last()? {return None;}
		// All-sites radio styling must use the currently selected safety clearance; C++ cache update on change.
		if self.earth_margin_deg != margin
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.earth_margin_deg = margin;
			// Check this state or input before acting; C++ conditional branch.
			if let Some(data) = &mut self.data
			{
				data.update_earth_margin(margin);
				// Update this persistent member explicitly; C++ member assignment.
				self.summary_key=None;
			}
		}
		// Do not silently extrapolate beyond a site's coverage; C++ range guard.
		// Resolve a real sample at or immediately before the requested epoch; C++ member call.
		self.set_time(time);
		self.directions()
	}

	// Share the radio setting without exposing arrays to the renderer; C++ const scalar getter.
	pub fn margin(&self) -> f32
	{
		self.earth_margin_deg
	}

	// Surface background load failures even when another site's panorama is selected; C++ optional const string view.
	pub fn load_error(&self) -> Option<&str>
	{
		self.error.as_deref()
	}

	// Copy only the current scientific directions/availability into the independent renderer; C++ optional<array<BodyDirection,2>>.
	pub fn directions(&self) -> Option<[globe::BodyDirection; 2]>
	{
		let data=self.data.as_ref()?;
		let [a,e]=crate::timeline::sample(&data.sun,self.cursor)?;
		let sun=globe::BodyDirection { az_deg:a,el_deg:e,visible:sun_fraction(e,data.horizon_at(a))>0.0 };
		let [a,e]=crate::timeline::sample(&data.earth,self.cursor)?;
		Some([sun,globe::BodyDirection { az_deg:a,el_deg:e,visible:e>data.horizon_at(a)+self.earth_margin_deg }])
	}

	// Show time settings in the grouped menu, not over the panorama; C++ member function.
	pub fn time_controls(&mut self, ui: &mut egui::Ui)
	{
		if self.data.is_none() { ui.label("Load a site to choose time."); return; }
		let mut cursor=self.cursor;
		self.timeline.lock().controls(ui,&mut cursor);
		self.set_time(cursor);
		self.paths_controls(ui);
	}

	fn paths_controls(&mut self, ui:&mut egui::Ui)
	{
		ui.horizontal(|ui| {
			ui.small("Paths");
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
			globe::help::question(ui, "Choose the faint path length around the current date. Paths stay inside your selected period; this setting does not change statistics.");
		});
	}
	// Recalculate scientific clearance and summaries from one menu control; C++ member function.
	pub fn radio_controls(&mut self, ui: &mut egui::Ui)
	{
		// Preserve an explicit pending/error state without indexing absent samples; C++ optional guard.
		let Some(data) = &mut self.data
		else
		{
			ui.label("Load a site to adjust clearance.");
			return;
		};
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
			self.summary_key=None;
		}

		globe::help::question(
			ui,
			"Ask for extra room above hills before counting a clear Earth path. Seeing part of Earth does not guarantee radio service.",
		);
	}
	// Preserve detailed plots as an opt-in setting; C++ member function.
	pub fn technical_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			ui.checkbox(&mut self.technical, "Technical view");
			globe::help::question(ui, "Show the same sky as a numbered graph. Direction runs clockwise from north; height is measured above horizontal.");
		});
	}
}

// Compute full-period availability using the same predicates as the current status; C++ pure aggregate helper.
#[cfg(test)]
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

// Format a UTC epoch without consulting a platform clock; C++ chrono::format of sys_seconds.
pub(crate) fn utc_label(t: i64) -> String
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
		// Enforce the independent regression expectation; C++ test assertion.
		assert!((summary.pct_sun - 100.0 * 23.0 / 24.0).abs() < 1e-9);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(summary.longest_night_h, 1.0);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(summary.pct_earth, 100.0);
		// Recalculation honors the edited margin for the entire period; C++ cache mutation.
		data.update_earth_margin(1.0);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let blocked = availability(&data);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(blocked.pct_earth, 0.0);
		// Enforce the independent regression expectation; C++ test assertion.
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
			summary: blocked,
			timeline:std::sync::Arc::new(egui::mutex::Mutex::new(crate::timeline::Timeline::default())),
			cursor:0,
			summary_key:None,
			sky_error:None,
		};
		panorama.timeline.lock().options.period=crate::timeline::Period {start:0,end:86400};
		// At sample zero the Sun is below flat terrain and Earth fails the edited radio margin; C++ optional extraction.
		let first = panorama.directions().unwrap();
		// Preserve Sun/Earth ordering and the actual numeric sample; C++ numeric assertion.
		assert_eq!(first[0].el_deg, -1.0);
		// The ray flags must use the same predicates as the UI, not globe illumination; C++ boolean assertions.
		assert!(!first[0].visible && !first[1].visible);
		// Moving time must replace the exported direction rather than retaining the first sample; C++ index edit.
		panorama.set_time(3600);
		// Re-read the chosen sample through the production bridge; C++ optional extraction.
		let next = panorama.directions().unwrap();
		// The Sun has risen while Earth remains below the requested safety clearance; C++ scalar assertions.
		assert_eq!(next[0].el_deg, 1.0);
		// Visibility changes with the same selected index; C++ boolean assertion.
		assert!(next[0].visible && !next[1].visible);
		// Each background site's own track is sampled at shared UTC, with no extrapolation; C++ temporal regression.
		assert!(panorama.directions_at(-1, 0.5).is_none());
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(panorama.directions_at(0, 0.5).unwrap()[1].visible);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(panorama.time(), Some(0));
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(panorama.directions_at(i64::MAX, 0.5).is_none());
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(panorama.directions_at(3600, 1.0).unwrap()[0].visible);
		// Failed or missing loading state must not export stale scientific geometry; C++ optional reset.
		panorama.data = None;
		// Preserve an explicit loading error without inventing a sample; C++ error assignment.
		panorama.error = Some("fixture unavailable".into());
		// Absence propagates to Globe::set_directions and clears the previous pair; C++ nullopt assertion.
		assert!(panorama.directions().is_none());
	}
}
