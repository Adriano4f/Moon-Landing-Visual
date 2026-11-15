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
use lunar_core::{Track, sun_fraction};
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
}

// Define view-model member functions; C++ out-of-class member definitions.
impl Panorama
{
	// Start loading once during application construction; C++ a constructor with an asynchronous operation.
	pub fn new(ctx: egui::Context) -> Self
	{
		// Choose the first UTC sample and a centered 30-day window; C++ aggregate initialization.
		Self {
			pending: data::load(ctx),
			data: None,
			error: None,
			index: 0,
			hours: 720,
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
			Some(Ok(data)) => self.data = Some(data),
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

	// Rebuild widgets every frame from stable state; C++ an immediate-mode draw method.
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		// Announce the scientific view independently of the presentation globe; C++ submit a heading.
		ui.heading("Sun and Earth panorama");
		// Require validated data before building sliders or indexing arrays; C++ checked optional extraction.
		let Some(data) = &self.data
		else
		{
			// Preserve the asynchronous failure reason or pending state; C++ optional::value_or.
			ui.label(self.error.as_deref().unwrap_or("Loading Horizons data..."));
			// No fictional ephemeris is displayed while unavailable; C++ early return.
			return;
		};
		// Identify the actual site; C++ submit a borrowed string label.
		ui.label(&data.site.name);
		// Make the missing-DEM assumption explicit; C++ conditional expression.
		ui.label(
			if data.horizon.is_some()
			{
				"terrain horizon"
			}
			else
			{
				"flat horizon (no DEM yet)"
			},
		);
		// Display the selected sample in UTC, not local browser time; C++ chrono formatting.
		ui.strong(utc_label(data.sun.t[self.index]));
		// Give the UTC slider the remaining row width; C++ set a local widget style dimension.
		ui.spacing_mut().slider_width = (ui.available_width() - 100.0).max(100.0);
		// Quantize the slider to real samples, so no uncomputed ephemerides are invented; C++ size_t slider binding.
		ui.add(
			egui::Slider::new(&mut self.index, 0..=data.sun.t.len() - 1)
				.show_value(false)
				.text("UTC sample"),
		);
		// Place window choices and single-sample steps together; C++ immediate-mode row callback.
		ui.horizontal(|ui| {
			// Saturate at the first sample rather than underflowing usize; C++ max(index-1,0).
			if ui.button("< 1h").clicked()
			{
				self.index = self.index.saturating_sub(1);
			}
			// Clamp the next step at the final inclusive endpoint; C++ min(index+1,last).
			if ui.button("1h >").clicked()
			{
				self.index = (self.index + 1).min(data.sun.t.len() - 1);
			}
			// Rebuild the menu with a stable widget identity; C++ a combo box bound to the integer choice.
			egui::ComboBox::from_id_salt("time_window")
				.selected_text(window_label(self.hours))
				.show_ui(ui, |ui| {
					// Fixed scientific time spans, not variable sample counts; C++ range-for over pairs.
					for (hours, label) in [
						(24, "1 day"),
						(168, "7 days"),
						(720, "30 days"),
						(0, "All data"),
					]
					{
						// Clicking updates persistent choice, while widgets themselves are rebuilt; C++ radio/menu binding.
						ui.selectable_value(&mut self.hours, hours, label);
					}
				});
		});
		// Borrow the selected Sun direction and interpolate its local horizon; C++ two float locals.
		let (sun_az, sun_el) = (data.sun.az_deg[self.index], data.sun.el_deg[self.index]);
		// Use the core's finite-disk approximation, not a duplicate visibility formula; C++ function call.
		let fraction = sun_fraction(sun_el, data.horizon_at(sun_az));
		// Distinguish any/partial/full solar visibility; C++ nested conditional expression.
		let sun_status = if fraction <= 0.0
		{
			"below"
		}
		else if fraction >= 1.0
		{
			"above"
		}
		else
		{
			"partly above"
		};
		// Keep Sun numbers and plotted color consistent; C++ formatted colored label.
		ui.colored_label(
			Color32::YELLOW,
			format!("Sun az {sun_az:.2}, el {sun_el:+.2} deg"),
		);
		// State that the percentage describes the solar disk, not time availability; C++ formatted label.
		ui.label(format!(
			"Sun {sun_status}; {:.1}% disk visible",
			fraction * 100.0
		));
		// Borrow the aligned Earth direction; C++ two float locals.
		let (earth_az, earth_el) = (data.earth.az_deg[self.index], data.earth.el_deg[self.index]);
		// Use the precomputed core center-visibility result; C++ bool lookup and conditional string.
		let earth_status = if data.earth_above[self.index]
		{
			"above"
		}
		else
		{
			"below"
		};
		// Report azimuth, elevation, and visibility without substituting an expected azimuth; C++ formatted colored label.
		ui.colored_label(
			Color32::LIGHT_BLUE,
			format!("Earth az {earth_az:.2}, el {earth_el:+.2} deg"),
		);
		// Clarify Earth is a center-above-horizon test, unlike the solar disk fraction; C++ string formatting.
		ui.label(format!("Earth center {earth_status} horizon"));
		// Select by UTC duration, allowing irregular future timestamps; C++ binary-search interval.
		let range = window(&data.sun.t, self.index, self.hours);
		// Explain endpoint clipping without crowding a phone with duplicate timestamps; C++ formatted count label.
		ui.small(format!(
			"Window: {} samples (centered, clipped)",
			range.len()
		));
		// Reserve a usable graph even when a short viewport needs outer scrolling; C++ max of remaining height.
		let height = ui.available_height().max(220.0);
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
								f64::from(track.az_deg[self.index].rem_euclid(360.0)),
								f64::from(track.el_deg[self.index]),
							]],
						)
						.color(color)
						.radius(4.0),
					);
				}
			});
	}
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
fn segments(track: &Track, range: Range<usize>) -> Vec<Vec<[f64; 2]>>
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
}
