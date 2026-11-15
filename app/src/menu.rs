//! Grouped settings and plain-language orientation, separated from the scientific canvas.

// Use the framework's UI types without a second egui dependency; C++ namespace alias.
use eframe::egui;
// Reuse one touch-friendly help implementation in both crates; C++ imported helper.
use globe::help::question;
// Access the owning application without duplicating scientific state; C++ parent class declaration.
use crate::MoonApp;

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
			question(ui, "Compare published lunar south-pole landing points and dates for sunlight and a direct radio path to Earth. Open Menu for grouped settings and Explanations. The globe is illustrative; the panorama uses the computed terrain horizon. This is not an engineering or landing-safety assessment.");
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
				.default_pos([8.0, 38.0])
				.default_width(300.0)
				.max_width(320.0)
				.max_height((ctx.content_rect().height() - 60.0).max(100.0))
				.vscroll(true)
				.show(&ctx, |ui| self.menu_contents(ui));
			// Persist the presentation change between frames; C++ member assignment.
			self.menu_open = open;
		}
		else
		{
			// A bounded sidebar gives the main views the remaining desktop width; C++ side-container callback.
			egui::Panel::left("settings")
				.default_size(270.0)
				.size_range(250.0..=320.0)
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
		section(
			ui,
			"Explanations",
			false,
			"A short guide to what this app shows, how to read it and what it does not predict.",
			explanations,
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
			question(ui, "Show only the chosen site's Sun and Earth directions. Bright means available under the local terrain and radio test; dim means blocked. The disks and ray lengths are not to scale.");
		});
		// Submit this immediate-mode widget or explanation; C++ UI helper call.
		ui.horizontal_wrapped(|ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.radio_value(&mut self.all_rays, true, "All sites");
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, "Load and show every site's own directions at the shared date. This downloads additional data. Back-facing sites remain hidden by the globe; readiness and errors appear below.");
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
			"The horizon is the edge of the ground against the sky. The strip looks all the way around you: north, east, south, west, then north again. Height is stretched and labeled. Gray hills hide parts of the Sun and Earth; faint lines show their paths over your chosen window.",
		),
		(
			"Sun and power",
			"Yellow is the Sun. Its visible percentage is the area of its round face above the hills. Even a small visible part counts as some sunlight. This is not guaranteed electrical power: panel direction, dust, equipment and eclipses are not modeled.",
		),
		(
			"Earth, direct communication and margin",
			"Blue is Earth. A direct signal travels straight to Earth without a relay. The safety margin asks for extra room above the hills; a partly visible blue disk can still fail that test. The app does not predict antennas, Earth-station schedules or interference.",
		),
		(
			"Time and summaries",
			"UTC is the shared world clock. Data is sampled once per hour, so short events can be missed. Summary percentages cover the full stated period, not just the path window. A day in the longest-night card is 24 hours.",
		),
		(
			"Rays and camera",
			"Yellow and blue rays are direction diagrams at the selected date, not distances or physical beams. Dim means blocked at that site. Drag to orbit the Moon or use View from Earth. Go to site centers your point. The globe has limited texture detail; use Site to separate close neighbors.",
		),
		(
			"Lighting, earthshine and fill",
			"Real Sun and Earth directions change with time. Earthshine is faint reflected sunlight from Earth; boosted exposure makes only that contribution 1000 times brighter. Far side fill is invented gray-blue light for inspection, not real illumination. Neither aid changes the scientific results.",
		),
		(
			"Data sources and limits",
			"Coordinates come from published polar studies and are cited in Site. Sky directions come from NASA/JPL Horizons. Terrain uses NASA LOLA elevation products, with a 2 m observer. The globe uses NASA color and relief maps, but is not a detailed terrain simulation. Relays, heat, landing safety, equipment and short between-hour events are not predicted.",
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
				egui::ComboBox::from_id_salt("published-site").selected_text(&self.catalog[self.selected].name).show_ui(ui, |ui| {
					// Visit each explanatory item in order; C++ range-for loop.
					for (id, entry) in self.catalog.iter().enumerate() { ui.selectable_value(&mut self.selected, id, &entry.name).on_hover_text(&entry.source); }
				});
				// Submit this immediate-mode widget or explanation; C++ UI helper call.
				question(ui, "Select a published landing point. Its local sky, statuses and summaries replace the previous site's data while preserving the shared date. Data loads only when needed; coordinates are not moved to improve results.");
			});
				// Submit this immediate-mode widget or explanation; C++ UI helper call.
				ui.small(&self.catalog[self.selected].source);
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
