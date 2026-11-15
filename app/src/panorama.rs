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
	pub fn new(ctx: egui::Context, file: String) -> Self
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
			pending: data::load(ctx, file),
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
			Some(Ok(mut data)) =>
			{
				// Pending nonselected loads may already have a nondefault radio setting; C++ cache restoration.
				data.update_earth_margin(self.earth_margin_deg);
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

	// Export one UTC epoch for every site's own track lookup; C++ optional<int64_t>.
	pub fn time(&self) -> Option<i64>
	{
		self.data.as_ref().map(|d| d.sun.t[self.index])
	}

	// Restore shared time when switching cached site views; C++ checked binary search.
	pub fn set_time(&mut self, time: i64)
	{
		// Loaded tracks have strictly ordered epochs; C++ optional guard.
		if let Some(data) = &self.data
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.index = data
				.sun
				.t
				.partition_point(|&t| t <= time)
				.saturating_sub(1)
				.min(data.sun.t.len() - 1);
		}
	}

	// Evaluate another site's own current sample, never reuse the selected site's angles; C++ optional array.
	pub fn directions_at(&mut self, time: i64, margin: f32) -> Option<[globe::BodyDirection; 2]>
	{
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
				self.summary = availability(data);
			}
		}
		// Do not silently extrapolate beyond a site's coverage; C++ range guard.
		let d = self.data.as_ref()?;
		// Check this state or input before acting; C++ conditional branch.
		if time < d.sun.t[0] || time > *d.sun.t.last()?
		{
			return None;
		}
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
				// Update this persistent member explicitly; C++ member assignment.
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
				// Update this persistent member explicitly; C++ member assignment.
				self.index = self.index.saturating_sub(1);
			}
			// Clamp at the inclusive final epoch; C++ clamped increment.
			if ui.button("1 hour >").clicked()
			{
				// Update this persistent member explicitly; C++ member assignment.
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
