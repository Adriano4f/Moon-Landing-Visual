use super::*;

impl Panorama
{
	// Present a plain-language scene first, with optional numerical detail; C++ immediate-mode member function.
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		// Scrollable child UIs must wrap prose instead of widening the entire panorama; C++ local text-layout policy.
		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
		globe::help::annotated(
			ui,
			egui::RichText::new("Standing on the Moon").heading(),
			"Imagine standing at the selected point. The strip shows every direction around you; gray is terrain, yellow is Sun, blue is Earth. Hills hide the parts below their edge. Height is stretched and labeled. Faint paths cover the menu time window; this is not a landing-safety or thermal model.",
		);
		// Never draw invented tracks while waiting or after an error; C++ optional extraction.
		let Some(data) = &mut self.data
		else
		{
			// Show the real loader state; C++ optional value_or.
			globe::help::annotated(
				ui,
				self.error
					.as_deref()
					.unwrap_or("Loading site and sky data..."),
				"Results are unavailable until the selected track has loaded and passed validation. No substitute directions or statistics are shown for missing or invalid data.",
			);
			// Stop before indexing arrays; C++ early return.
			return;
		};
		// Identify the published observation point; C++ borrowed text.
		globe::help::annotated(
			ui,
			&data.site.name,
			"The selected published observation point. Use Site in the menu to change it; nearby coordinates can have very different horizons.",
		);
		solar_status(ui, data, self.index);
		earth_status(ui, data, self.index);
		// Keep UTC explicit, independent of the computer's time zone; C++ time formatting.
		globe::help::annotated(
			ui,
			utc_label(data.sun.t[self.index]),
			"The current shared UTC date and time. Open Time in the menu to change it. Hourly sampling can miss short events.",
		);
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
		summary_cards(ui, data, &self.summary);
	}
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
			technical_items(plot, data, index, range.clone());
		});
}

fn solar_status(ui: &mut egui::Ui, data: &SiteData, index: usize)
{
	// Delegate the solar area to the scientific core; C++ pure function call.
	let fraction = sun_fraction(
		data.sun.el_deg[index],
		data.horizon_at(data.sun.az_deg[index]),
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
	globe::help::annotated(
		ui,
		egui::RichText::new(sun_text)
			.size(18.0)
			.strong()
			.color(sun_color),
		"Sunlight can reach the site when some of the yellow disk clears the terrain. The percentage is the visible area of the disk, not guaranteed electrical output. Panel direction, hardware, dust and eclipses are not modeled.",
	);
}

fn earth_status(ui: &mut egui::Ui, data: &SiteData, index: usize)
{
	// Include the selected radio clearance, not just visible blue pixels; C++ cached bool.
	let earth_visible = data.earth_above[index];
	// Keep the full plain-language status visible without interpreting a graph; C++ conditional string.
	globe::help::annotated(
		ui,
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
		"A direct radio signal could reach Earth without terrain blocking it, including the chosen safety margin. This is a geometric opportunity, not guaranteed service; relays, antennas and Earth stations are not modeled.",
	);
	// Qualify a blocked radio link that may still have a partly visible disk; C++ explanatory label.
	ui.small("Radio status includes the safety margin, not just the visible blue disk.");
}

fn summary_cards(ui: &mut egui::Ui, data: &SiteData, summary: &lunar_core::Stats)
{
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
	card_grid(ui, summary, card_width);
	// Avoid implying that hourly held-sample statistics are continuous event solutions; C++ disclaimer.
	ui.small("Hourly estimates. Sun time means at least part of the disk is visible. Power and radio service are not guaranteed.");
}

fn card_grid(ui: &mut egui::Ui, summary: &lunar_core::Stats, card_width: f32)
{
	// Two columns wrap well on both phone and desktop panels; C++ grid of framed cards.
	egui::Grid::new("site_summary")
		.num_columns(2)
		.spacing([8.0, 6.0])
		.show(ui, |ui| {
			// Include units and the definition of sunlight in the visible labels; C++ range-for over values.
			for (i, (title, value)) in [
				("Time with any Sun", format!("{:.1}%", summary.pct_sun)),
				("Time with Earth link", format!("{:.1}%", summary.pct_earth)),
				("Time with both", format!("{:.1}%", summary.pct_both)),
				(
					"Longest night",
					format!("{:.2} days", summary.longest_night_h / 24.0),
				),
			]
			.iter()
			.enumerate()
			{
				summary_card(ui, title, value, i, card_width);
				// Close each pair explicitly; C++ grid row break.
				if i % 2 == 1
				{
					ui.end_row();
				}
			}
		});
}

fn summary_card(ui: &mut egui::Ui, title: &str, value: &str, i: usize, card_width: f32)
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
								ui.horizontal_wrapped(|ui| { ui.small(title); globe::help::question(ui, match i { 0 => "Fraction of the entire data period with at least some Sun visible. Partial disks count; this is not energy yield.", 1 => "Fraction of the entire period with direct Earth clearance at your chosen radio margin, not guaranteed radio service.", 2 => "Fraction of the entire period when sunlight and direct Earth clearance are both available.", _ => "Longest continuous sampled interval without any Sun, measured in days. A day here is 24 hours; short events between hourly samples can be missed." }); });
							});
						});
}

fn technical_items(
	plot: &mut egui_plot::PlotUi<'_>,
	data: &SiteData,
	index: usize,
	range: Range<usize>,
)
{
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
}
