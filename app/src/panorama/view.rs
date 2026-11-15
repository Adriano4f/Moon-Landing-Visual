use super::*;

impl Panorama
{
	pub fn show(&mut self, ui: &mut egui::Ui)
	{
		// Scrollable child UIs must wrap prose instead of widening the entire panorama.
		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
		let gap = if ui.ctx().content_rect().width() >= 760.0 { 4.0 } else { 6.0 };
		ui.spacing_mut().item_spacing = egui::vec2(8.0, gap);
		globe::help::annotated(
			ui,
			egui::RichText::new("Standing on the Moon").heading(),
			"The full circle around your site; gray hills hide the Sun (yellow) or Earth (blue) when they are behind them.",
		);
		// Never draw invented tracks while waiting or after an error.
		let Some(data) = &self.data
		else
		{
			globe::help::annotated(
				ui,
				self.error
					.as_deref()
					.unwrap_or("Loading site and sky data..."),
				"Results appear once the selected site's data has loaded.",
			);
			return;
		};
		ui.small(&data.site.name);
		let Some(directions)=self.directions() else { ui.label(self.sky_status()); return; };
		solar_status(ui, data, directions[0]);
		earth_status(ui, directions[1].visible);
		// Keep UTC explicit, independent of the computer's time zone.
		ui.small(utc_label(self.cursor));
		if self.technical
		{
			technical(ui, data, self.cursor, self.path_range());
		}
		else
		{
			crate::sky::show(
				ui,
				data,
				self.cursor,
				self.path_range(),
			);
		}
		egui::Frame::NONE
			.fill(Color32::from_rgb(28, 32, 38))
			.corner_radius(10.0)
			.inner_margin(8.0)
			.show(ui, |ui| self.time_controls(ui));
		let options=self.options();
		if self.summary_key==Some((options,self.earth_margin_deg)) {
			summary_cards(ui, &self.summary);
		} else { ui.label(self.sky_status()); }
	}
}

fn technical(ui: &mut egui::Ui, data: &SiteData, time:i64, range:Range<usize>)
{
	// Select by UTC duration, allowing irregular future timestamps.
	// Explain endpoint clipping without crowding a phone with duplicate timestamps.
	ui.small(format!(
		"Window: {} samples (centered, clipped)",
		range.len()
	));
	let height = if ui.ctx().content_rect().width() < 700.0
	{
		210.0
	}
	else
	{
		(ui.available_height() - 300.0).clamp(170.0, 300.0)
	};
	// Fix azimuth bounds while allowing elevation to fit terrain/track extrema.
	Plot::new("panorama")
		// Leave room for the right endpoint's centered tick label inside the panel clip.
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
		// Force cardinal ticks rather than accepting automatic 100-degree spacing.
		.x_grid_spacer(|_| {
			(0..=4)
				.map(|i| GridMark {
					value: f64::from(i) * 90.0,
					step_size: 90.0,
				})
				.collect()
		})
		// Name both north endpoints of the unwrapped plot.
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
			technical_items(plot, data, time, range.clone());
		});
}

fn solar_status(ui: &mut egui::Ui, data: &SiteData, direction:globe::BodyDirection)
{
	let fraction = sun_fraction(
		direction.el_deg,
		data.horizon_at(direction.az_deg),
	);
	// Green indicates any available sunlight, not full disk exposure.
	let (sun_text, sun_color) = if fraction > 0.0
	{
		(
			format!("Sunlight: {:.1}% of Sun visible", fraction * 100.0),
			Color32::LIGHT_GREEN,
		)
	}
	else
	{
		("Sunlight: blocked by hills".into(), Color32::LIGHT_RED)
	};
	ui.label(egui::RichText::new(sun_text).size(16.0).color(sun_color))
		.on_hover_text("Percentage of the Sun showing above the hills right now.");
}

fn earth_status(ui: &mut egui::Ui, earth_visible:bool)
{
	// Include the selected radio clearance, not just visible blue pixels.
	// Keep the full plain-language status visible without interpreting a graph.
	ui.label(egui::RichText::new(
			if earth_visible
			{
				"Earth link: clear above hills"
			}
			else
			{
				"Earth link: blocked or too close to hills"
			},
		)
		.size(16.0)
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
		))
		.on_hover_text("Earth is linked when you can see it directly, at least the safety margin above the hills.");
}

fn summary_cards(ui: &mut egui::Ui, summary: &lunar_core::Stats)
{
	let columns = if ui.available_width() >= 560.0 { 4 } else { 2 };
	let card_width = ((ui.available_width() - 16.0 * columns as f32 - 8.0 * (columns - 1) as f32) / columns as f32).max(70.0);
	card_grid(ui, summary, card_width, columns);
}

fn card_grid(ui: &mut egui::Ui, summary: &lunar_core::Stats, card_width: f32, columns: usize)
{
	egui::Grid::new("site_summary")
		.num_columns(columns)
		.spacing([8.0, 6.0])
		.show(ui, |ui| {
			// Include units and the definition of sunlight in the visible labels.
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
				summary_card(ui, title, value, card_width);
				if i % columns == columns - 1
				{
					ui.end_row();
				}
			}
		});
}

fn summary_card(ui: &mut egui::Ui, title: &str, value: &str, card_width: f32)
{
	egui::Frame::NONE
						.inner_margin(8.0)
						.corner_radius(10.0)
						.fill(Color32::from_rgb(18, 25, 33))
						.show(ui, |ui| {
							// Stack each number above its caption instead of inheriting the grid's row layout.
							ui.vertical(|ui| {
								ui.set_width(card_width);
								ui.label(egui::RichText::new(value).size(21.0).strong());
								ui.small(title);
							});
						});
}

fn technical_items(
	plot: &mut egui_plot::PlotUi<'_>,
	data: &SiteData,
	time: i64,
	range: Range<usize>,
)
{
	// Always show the geometric flat reference, even if terrain is supplied.
	plot.hline(HLine::new("Flat 0 deg", 0.0).color(Color32::GRAY));
	// Add terrain only when present; this is profile interpolation, not DEM ingestion.
	if let Some(horizon) = &data.horizon
	{
		let bins = horizon.h_deg.len().max(360);
		// Close the cyclic line at 360 using Horizon::at, including all bin boundaries.
		let points: Vec<_> = (0..=bins)
			.map(|i| {
				let az = i as f32 * 360.0 / bins as f32;
				[f64::from(az), f64::from(horizon.at(az))]
			})
			.collect();
		// Keep the terrain visually distinct from the flat reference and both bodies.
		plot.line(Line::new("Terrain", points).color(Color32::LIGHT_RED));
	}
	// Apply identical window/wrap rules to both tracks.
	for (name, track, color) in [
		("Sun", &data.sun, Color32::YELLOW),
		("Earth", &data.earth, Color32::LIGHT_BLUE),
	]
	{
		let Some([az,el])=crate::timeline::sample(track,time) else {continue;};
		// Never connect segments separated by a north wrap.
		for segment in segments(track, range.clone())
		{
			plot.line(Line::new(name, segment).color(color));
		}
		plot.points(
			Points::new(
				format!("{name} now"),
				vec![[
					f64::from(az.rem_euclid(360.0)),
					f64::from(el),
				]],
			)
			.color(color)
			.radius(4.0),
		);
	}
}
