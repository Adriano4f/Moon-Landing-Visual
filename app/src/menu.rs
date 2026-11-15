//! Grouped settings and plain-language orientation, separated from the scientific canvas.

use eframe::egui;
use globe::help::question;
use crate::MoonApp;

const GUIDE_DISMISSED: &str = "karaya-guide-dismissed";

pub(super) fn guide_visible(storage: Option<&dyn eframe::Storage>) -> bool
{
	storage.and_then(|s| s.get_string(GUIDE_DISMISSED)).as_deref() != Some("true")
}

pub(super) fn dismiss_guide(storage: &mut dyn eframe::Storage)
{
	storage.set_string(GUIDE_DISMISSED, "true".into());
	storage.flush();
}

/// Summarize the verified linked reference, retaining secondary-source attribution.
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

	#[derive(Default)]
	struct Memory
	{
		value: Option<String>,
		flushed: bool,
	}

	impl eframe::Storage for Memory
	{
		fn get_string(&self, key: &str) -> Option<String>
		{
			assert_eq!(key, GUIDE_DISMISSED);
			self.value.clone()
		}

		fn set_string(&mut self, key: &str, value: String)
		{
			assert_eq!(key, GUIDE_DISMISSED);
			self.value = Some(value);
		}

		fn remove_string(&mut self, key: &str)
		{
			assert_eq!(key, GUIDE_DISMISSED);
			self.value = None;
		}

		fn flush(&mut self)
		{
			self.flushed = true;
		}
	}

	#[test]
	fn guide_dismissal_is_persisted_and_missing_or_invalid_storage_shows_help()
	{
		let mut storage = Memory::default();
		assert!(guide_visible(None));
		assert!(guide_visible(Some(&storage)));
		storage.value = Some("invalid".into());
		assert!(guide_visible(Some(&storage)));
		dismiss_guide(&mut storage);
		assert!(storage.flushed);
		assert!(!guide_visible(Some(&storage)));
	}

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

fn section(
	ui: &mut egui::Ui,
	title: &str,
	open: bool,
	explanation: &str,
	body: impl FnOnce(&mut egui::Ui),
)
{
	let id = ui.make_persistent_id(title);
	egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open)
		.show_header(ui, |ui| {
			ui.strong(title);
			if explanation.is_empty() { ui.set_min_height(ui.spacing().interact_size.y); }
			else { question(ui, explanation); }
		})
		.body(body);
}

pub(crate) fn logo(ui: &mut egui::Ui)
{
	let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
	ui.painter()
		.circle_filled(rect.center(), 9.0, egui::Color32::from_rgb(132, 169, 193));
	ui.painter().circle_filled(
		rect.center() + egui::vec2(4.0, -2.0),
		7.5,
		ui.visuals().panel_fill,
	);
	// Keep the title small enough for a phone header.
	ui.strong("Karaya Explorer");
}

impl MoonApp
{
	/// Render the settings as a floating overlay so the canvas keeps the full width.
	pub(crate) fn settings(&mut self, root: &mut egui::Ui, phone: bool)
	{
		let ctx = root.ctx().clone();
		egui::Panel::top("brand").show(root, |ui| { ui.horizontal(|ui| {
			logo(ui);
			if ui.button(if self.menu_open { "Close menu" } else { "Menu" }).clicked() { self.menu_open = !self.menu_open; }
			if ui.button("?").on_hover_text("How to use").clicked()
			{
				self.guide_open = true;
			}
		}); });
		// A floating menu overlays the main views on every screen size instead of reserving a sidebar.
		let mut open = self.menu_open;
		egui::Window::new("Menu")
			.id(egui::Id::unique("settings-menu"))
			.open(&mut open)
			.collapsible(false)
			.movable(true)
			.constrain_to(ctx.content_rect().shrink(8.0))
			.default_pos([8.0, 66.0])
			.default_width(if phone { 300.0 } else { 440.0 })
			.min_width(if phone { 96.0 } else { 420.0 })
			.max_width(if phone { 320.0 } else { (ctx.content_rect().width() - 48.0).min(460.0) })
			.max_height((ctx.content_rect().height() - 90.0).max(100.0))
			.vscroll(true)
			.show(&ctx, |ui| {
				if !phone
				{
					ui.spacing_mut().item_spacing = egui::vec2(14.0, 12.0);
				}
				self.menu_contents(ui);
			});
		self.menu_open = open;
	}

	pub(crate) fn welcome(&mut self, ctx: &egui::Context)
	{
		if self.guide_open
		{
			egui::Modal::new(egui::Id::unique("welcome-guide")).show(ctx, |ui| {
				ui.set_width((ctx.content_rect().width() - 56.0).clamp(1.0, 360.0));
				ui.heading("How to use");
				for step in [
					"1. Pick a site, or click the Moon.",
					"2. Choose a period, date and step.",
					"3. Read Sun/Earth % and the graph.",
					"4. Use Change options for settings.",
				]
				{
					ui.label(step);
				}
				if ui.button("Got it").clicked()
				{
					self.guide_open = false;
					self.guide_dismissal_pending = true;
				}
			});
		}
	}

	fn menu_contents(&mut self, ui: &mut egui::Ui)
	{
		// Wrap every explanation to the sidebar width.
		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
		self.site_section(ui);
		ui.add(egui::Separator::default().spacing(0.0));
		section(
			ui,
			"Display & more",
			false,
			"Optional camera, lighting, ray and Earth link settings.",
			|ui| self.advanced_sections(ui),
		);
		ui.add_space(6.0);
		section(
			ui,
			"Explanations",
			false,
			"A short guide to what each part of the screen shows.",
			explanations,
		);
	}

	/// Hide specialist settings until requested, retaining established groups.
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
			"Yellow rays show sunlight and blue rays show the Earth link.",
			|ui| self.ray_controls(ui),
		);
		section(
			ui,
			"Earth link",
			false,
			"Earth must be at least this many degrees above the hills to count as linked.",
			|ui| {
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
			"Show the same sky as a graph of direction from north against height.",
			|ui| {
				if let Some(p) = &mut self.panoramas[self.selected]
				{
					p.technical_controls(ui);
					p.paths_controls(ui);
				}
			},
		);
	}

	fn ray_controls(&mut self, ui: &mut egui::Ui)
	{
		self.globe.surface_ray_controls(ui);
		ui.horizontal_wrapped(|ui| {
			ui.radio_value(&mut self.all_rays, false, "Selected site only");
			question(ui, "Show rays from the selected site only; a dim ray is blocked by hills.");
		});
		ui.horizontal_wrapped(|ui| {
			ui.radio_value(&mut self.all_rays, true, "All sites");
			question(ui, "Show rays from every site at once.");
		});
		if self.all_rays
		{
			let ready = self
				.panoramas
				.iter()
				.flatten()
				.filter(|p| p.site().is_some())
				.count();
			globe::help::annotated(
				ui,
				format!("Site tracks ready: {ready}/{}", self.catalog.len()),
				"Number of sites whose data has loaded and can show rays.",
			);
			for (id, p) in self.panoramas.iter().enumerate()
			{
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

fn explanations(ui: &mut egui::Ui)
{
	for (title, text) in [
		(
			"Standing on the Moon",
			"The full circle around your site; gray hills hide the Sun or Earth when they are behind them.",
		),
		(
			"Sun",
			"The Sun percentage is how much of the Sun shows above the hills right now.",
		),
		(
			"Earth link",
			"Earth is linked when you can see it directly, at least the safety margin above the hills.",
		),
		(
			"Time",
			"Times are UTC, and the summary cards cover the whole chosen period.",
		),
		(
			"Rays",
			"Yellow rays show sunlight and blue rays show the Earth link; a dim ray is blocked by hills.",
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
		ui.strong(title);
		ui.label(text);
		ui.add_space(6.0);
	}
}

impl MoonApp
{
	fn site_section(&mut self, ui: &mut egui::Ui)
	{
		section(
			ui,
			"Site",
			true,
			"Choose the published landing site you want to look from.",
			|ui| {
				ui.horizontal(|ui| {
					let width = (ui.available_width() - 54.0).max(100.0);
					ui.scope(|ui| {
						ui.set_max_width(width);
						egui::ComboBox::from_id_salt("published-site")
							.width(width)
							.wrap_mode(egui::TextWrapMode::Truncate)
							.selected_text(&self.catalog[self.selected].name)
							.show_ui(ui, |ui| {
								for (id, entry) in self.catalog.iter().enumerate()
								{
									ui.selectable_value(&mut self.selected, id, &entry.name);
								}
							});
					});
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
	}
}
