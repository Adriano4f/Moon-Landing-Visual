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
			if explanation.is_empty() { ui.set_min_height(ui.spacing().interact_size.y); }
			else { question(ui, explanation); }
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
			question(ui, "Compare lunar south pole sites by how often they see the Sun and Earth.");
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
			"Optional camera, lighting, ray and Earth margin settings.",
			|ui| self.advanced_sections(ui),
		);
		section(
			ui,
			"Explanations",
			false,
			"A short guide to what each part of the screen shows.",
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
			"Change how you look at the Moon without changing the site or date.",
			|ui| self.globe.camera_controls(ui),
		);
		section(
			ui,
			"Lighting",
			false,
			"Make the Moon picture easier to see without changing any result.",
			|ui| self.globe.lighting_controls(ui),
		);
		section(
			ui,
			"Rays",
			false,
			"Yellow and blue lines point from a site toward the Sun and Earth.",
			|ui| self.ray_controls(ui),
		);
		section(
			ui,
			"Radio margin",
			false,
			"Extra angle above the hills that Earth must clear to count as a direct link.",
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
			"Show the same sky as a graph with exact angles.",
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
			question(ui, "Point lines from this site toward the Sun and Earth, dim when blocked.");
		});
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.horizontal_wrapped(|ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.radio_value(&mut self.all_rays, true, "All sites");
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, "Point lines from every site toward the Sun and Earth at once.");
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
				"Number of sites whose data has loaded and can draw rays.",
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
						"This site's data failed to load, so it has no rays or results.",
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
			"View around the site",
			"The strip shows the full circle around the site; gray hills hide what is behind them.",
		),
		(
			"Sun",
			"The Sun percentage is how much of the Sun shows above the hills.",
		),
		(
			"Earth link",
			"Earth is linked when it is in direct view, a little above the hills.",
		),
		(
			"Time",
			"Times are UTC, and the summary cards cover the whole chosen period.",
		),
		(
			"Rays",
			"Yellow rays point to the Sun and blue rays point to Earth.",
		),
		(
			"Lighting",
			"Extra light only brightens the picture; it never changes the results.",
		),
		(
			"Data",
			"Sun and Earth positions and the hills come from NASA data.",
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
			"Choose the published landing site you want to look from.",
			|ui| {
				// Submit this immediate-mode widget or explanation; C++ UI helper call.
				ui.horizontal(|ui| {
					let width = (ui.available_width() - 54.0).max(100.0);
					ui.scope(|ui| {
						ui.set_max_width(width);
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
					});
					// Submit this immediate-mode widget or explanation; C++ UI helper call.
					let text =
						"Results assume an observer standing 2 m above the ground at this published site.";
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
			"",
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
