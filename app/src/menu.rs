//! Grouped settings and plain-language orientation, separated from the scientific canvas.

// Use the framework's UI types without a second egui dependency; C++ namespace alias.
use eframe::egui;
// Reuse one touch-friendly help implementation in both crates; C++ imported helper.
use globe::help::question;
// Access the owning application without duplicating scientific state; C++ parent class declaration.
use crate::MoonApp;

// Summarize the verified linked reference, retaining secondary-source attribution; C++ metadata formatter.
fn source_link(source: &str) -> Option<(String, &str)>
{
	let url = source
		.split_whitespace()
		.find(|word| word.starts_with("https://"))?;
	let reference = source.split(';').next()?;
	let verified = reference.rsplit("peer-reviewed ").next()?;
	let details = || {
		let author = verified.split_whitespace().next()?;
		let year = verified
			.split_whitespace()
			.map(|word| word.trim_matches(|c: char| !c.is_ascii_digit()))
			.find(|word| word.len() == 4)?;
		let table = verified
			.split_once("Table ")?
			.1
			.split_whitespace()
			.next()?
			.trim_end_matches([',', ';']);
		let relation = if reference.contains("peer-reviewed ")
		{
			" via"
		}
		else
		{
			":"
		};
		Some(format!(
			"Coordinates{relation} {author} {year}, Table {table}"
		))
	};
	Some((
		details().unwrap_or_else(|| "Coordinates: source publication".into()),
		url,
	))
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn short_verified_citations_and_missing_metadata()
	{
		let entries: Vec<crate::Entry> =
			serde_json::from_str(include_str!("../../sites.json")).unwrap();
		for (index, entry) in entries.iter().enumerate()
		{
			let (caption, url) = source_link(&entry.source).unwrap();
			assert_eq!(
				caption,
				if index < 4
				{
					"Coordinates: Glaser 2017, Table 2"
				}
				else
				{
					"Coordinates via Glaser 2017, Table 1"
				}
			);
			assert!(url.starts_with("https://ntrs.nasa.gov/"));
			assert!(!caption.contains("https://"));
		}
		assert!(source_link("").is_none());
		assert!(source_link("Missing link").is_none());
		assert_eq!(
			source_link("Unknown https://example.com").unwrap().0,
			"Coordinates: source publication"
		);
	}
}

// Explain and collapse a settings group with a stable identity; C++ templated callback helper.
fn section(
	ui: &mut egui::Ui,
	title: &str,
	open: bool,
	explanation: &str,
	body: impl FnOnce(&mut egui::Ui),
)
{
	// Keep each group's expansion state independent of frame order; C++ persistent identifier.
	let id = ui.make_persistent_id(title);
	// Put a genuine help button beside the collapsible heading; C++ header and body callbacks.
	egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open)
		.show_header(ui, |ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.strong(title);
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, explanation);
		})
		.body(body);
}

// Paint an unobtrusive vector crescent and the requested name; C++ custom-widget helper.
pub(crate) fn logo(ui: &mut egui::Ui)
{
	// Reserve the mark independently of screen pixel density; C++ logical-size layout.
	let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
	// Draw a soft blue-gray lunar disk; C++ circle drawing call.
	ui.painter()
		.circle_filled(rect.center(), 9.0, egui::Color32::from_rgb(132, 169, 193));
	// Cut out the unlit side using the panel's own background; C++ second filled circle.
	ui.painter().circle_filled(
		rect.center() + egui::vec2(4.0, -2.0),
		7.5,
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.visuals().panel_fill,
	);
	// Keep the title small enough for a phone header; C++ text widget.
	ui.strong("Karaya Explorer");
}

// Keep panel layout code close to its own controls; C++ out-of-class member definitions.
impl MoonApp
{
	// Render a fixed desktop sidebar or a phone drawer without squeezing the canvas; C++ responsive container.
	pub(crate) fn settings(&mut self, root: &mut egui::Ui, phone: bool)
	{
		// Retain a context handle for floating windows while panels borrow the root; C++ shared UI handle.
		let ctx = root.ctx().clone();
		// The compact brand stays visible when the phone menu is closed; C++ top-panel callback.
		egui::Panel::top("brand").show(root, |ui| { ui.horizontal(|ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			logo(ui);
			// Apply the UI event only when requested; C++ conditional branch.
			if phone && ui.button(if self.menu_open { "Close menu" } else { "Menu" }).clicked() { self.menu_open = !self.menu_open; }
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, "Compare places and dates for sunlight and a clear view of Earth. This is not a landing-safety assessment.");
		}); });
		// A floating phone drawer leaves the stacked main view at its natural width; C++ modal-free settings window.
		if phone
		{
			// Bind this frame's presentation value; C++ local initialization.
			let mut open = self.menu_open;
			egui::Window::new("Menu")
				.id(egui::Id::unique("phone-menu"))
				.open(&mut open)
				.collapsible(false)
				.default_pos([8.0, 66.0])
				.default_width(300.0)
				.max_width(320.0)
				.max_height((ctx.content_rect().height() - 90.0).max(100.0))
				.vscroll(true)
				.show(&ctx, |ui| self.menu_contents(ui));
			// Persist the presentation change between frames; C++ member assignment.
			self.menu_open = open;
		}
		else
		{
			// A bounded sidebar gives the main views the remaining desktop width; C++ side-container callback.
			egui::Panel::left("settings")
				.default_size(290.0)
				.size_range(280.0..=340.0)
				.show(root, |ui| {
					egui::ScrollArea::vertical().show(ui, |ui| self.menu_contents(ui));
				});
		}
	}

	// Group controls by the question a visitor is trying to answer; C++ member function.
	fn menu_contents(&mut self, ui: &mut egui::Ui)
	{
		// Wrap every explanation to the sidebar width; C++ local text policy.
		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
		self.site_time_sections(ui);
		section(
			ui,
			"More options",
			false,
			"Camera, lighting, rays and extra radio clearance. These settings are optional.",
			|ui| self.advanced_sections(ui),
		);
		section(
			ui,
			"Explanations",
			false,
			"A short guide to the picture, results and their limits.",
			explanations,
		);
	}

	// Hide specialist settings until requested, retaining established groups; C++ nested menu helper.
	fn advanced_sections(&mut self, ui: &mut egui::Ui)
	{
		section(
			ui,
			"View / camera",
			false,
			"Change only how you inspect the Moon, not the observation point or the scientific date.",
			|ui| self.globe.camera_controls(ui),
		);
		section(
			ui,
			"Lighting",
			false,
			"Sun and Earth directions follow the selected date. Exposure and fill are optional illustration aids, not changes to power availability.",
			|ui| self.globe.lighting_controls(ui),
		);
		section(
			ui,
			"Rays",
			false,
			"Yellow and blue direction diagrams start at published sites. They are not the actual distances to Sun or Earth.",
			|ui| self.ray_controls(ui),
		);
		section(
			ui,
			"Radio margin",
			false,
			"Ask for extra angular clearance above the hills before declaring a direct Earth radio opportunity.",
			|ui| {
				// Apply the UI event only when requested; C++ conditional branch.
				if let Some(p) = &mut self.panoramas[self.selected]
				{
					p.radio_controls(ui);
				}
			},
		);
		section(
			ui,
			"Technical",
			false,
			"Optional numerical graph of the same sky data, for users who want coordinates and detailed tracks.",
			|ui| {
				// Apply the UI event only when requested; C++ conditional branch.
				if let Some(p) = &mut self.panoramas[self.selected]
				{
					p.technical_controls(ui);
				}
			},
		);
	}

	// Keep loading progress and failures beside the mode that requested background tracks; C++ member function.
	fn ray_controls(&mut self, ui: &mut egui::Ui)
	{
		self.globe.surface_ray_controls(ui);
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.horizontal_wrapped(|ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.radio_value(&mut self.all_rays, false, "Selected site only");
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, "Show this site's Sun and Earth directions. Bright means clear; dim means blocked, and lengths are not real distances.");
		});
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.horizontal_wrapped(|ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.radio_value(&mut self.all_rays, true, "All sites");
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, "Show each site's directions at the same time. This downloads more data; the Moon still hides points on its back.");
		});
		// Apply the UI event only when requested; C++ conditional branch.
		if self.all_rays
		{
			// Bind this frame's presentation value; C++ local initialization.
			let ready = self
				.panoramas
				.iter()
				.flatten()
				.filter(|p| p.site().is_some())
				.count();
			globe::help::annotated(
				ui,
				format!("Site tracks ready: {ready}/{}", self.catalog.len()),
				"Only successfully loaded tracks can draw rays. Missing or invalid files are reported, not replaced by invented directions.",
			);
			// Visit each explanatory item in order; C++ range-for loop.
			for (id, p) in self.panoramas.iter().enumerate()
			{
				// Apply the UI event only when requested; C++ conditional branch.
				if let Some(error) = p.as_ref().and_then(|p| p.load_error())
				{
					globe::help::annotated(
						ui,
						egui::RichText::new(format!("{}: {error}", self.catalog[id].name))
							.color(egui::Color32::LIGHT_RED),
						"This site's data could not be loaded or validated. Its rays and results are unavailable; other loaded sites remain usable.",
					);
				}
			}
		}
	}
}

// Define ordinary-language concepts before offering technical detail; C++ static text section.
fn explanations(ui: &mut egui::Ui)
{
	// Each topic can be read independently on a small screen; C++ range-for over explanatory records.
	for (title, text) in [
		(
			"Horizon and panorama",
			"Read the strip from north through east, south and west to north again. Gray ground hides anything below its edge. Heights are stretched.",
		),
		(
			"Sun and power",
			"Yellow is the Sun. Its percentage is the visible area above hills, not electricity produced. Equipment, dust and eclipses are not included.",
		),
		(
			"Earth, direct communication and margin",
			"Blue is Earth. A direct signal needs a clear path plus extra room above hills. Antennas and ground-station availability are not predicted.",
		),
		(
			"Time and summaries",
			"UTC is the world clock. Hourly samples can miss short events. Summaries cover the stated period, not just the paths; one day is 24 hours.",
		),
		(
			"Rays and camera",
			"Rays show directions, not distances; dim means blocked. Drag to orbit, pinch or scroll to zoom, and use Go to site to center a point.",
		),
		(
			"Lighting, earthshine and fill",
			"Earthshine is faint sunlight reflected by Earth. Boost makes it 1,000 times brighter. Far-side fill is invented light for inspection; neither changes results.",
		),
		(
			"Data sources and limits",
			"Published coordinates are cited under Site. NASA Horizons supplies sky directions; LOLA supplies terrain with a 2 m observer. The globe is not a landing-safety model.",
		),
	]
	{
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.strong(title);
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.label(text);
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.add_space(6.0);
	}
}

impl MoonApp
{
	fn site_time_sections(&mut self, ui: &mut egui::Ui)
	{
		section(
			ui,
			"Site",
			true,
			"Choose an exact published observation point. Nearby points may overlap on the globe; the list always distinguishes them.",
			|ui| {
				// Submit this immediate-mode widget or explanation; C++ UI helper call.
				ui.horizontal_wrapped(|ui| {
					let width = (ui.available_width() - 54.0).max(100.0);
					egui::ComboBox::from_id_salt("published-site")
						.width(width)
						.wrap_mode(egui::TextWrapMode::Truncate)
						.selected_text(&self.catalog[self.selected].name)
						.show_ui(ui, |ui| {
							// Visit each explanatory item in order; C++ range-for loop.
							for (id, entry) in self.catalog.iter().enumerate()
							{
								ui.selectable_value(&mut self.selected, id, &entry.name);
							}
						});
					// Submit this immediate-mode widget or explanation; C++ UI helper call.
					let text =
						"Choose a published point. Nearby places can have very different hills.";
					if let Some((caption, url)) = source_link(&self.catalog[self.selected].source)
					{
						globe::help::question_with_link(ui, text, &caption, url);
					}
					else
					{
						question(ui, text);
					}
				});
			},
		);
		section(
			ui,
			"Time",
			true,
			"Choose the shared UTC date and the length of the faint paths. Summaries always cover the entire dataset.",
			|ui| {
				// Apply the UI event only when requested; C++ conditional branch.
				if let Some(p) = &mut self.panoramas[self.selected]
				{
					p.time_controls(ui);
					// Apply the UI event only when requested; C++ conditional branch.
					if let Some(t) = p.time()
					{
						// Persist the presentation change between frames; C++ member assignment.
						self.time = Some(t);
					}
				}
			},
		);
	}
}
