# Grouped settings and explanations

## 1. Reading the interface

The small crescent and Karaya Explorer name remain visible in the header. On a desktop, settings occupy a bounded left sidebar. On a phone, Menu opens a scrollable floating drawer, so opening settings does not squeeze the stacked globe and panorama into a narrow strip. Close menu or the drawer close button hides it again.

The top level has four groups: Site, Time, More options and Explanations. Only Site and Time start expanded. More options contains the collapsed View / camera, Lighting, Rays, Radio margin and Technical groups. The separate Source section is removed. The site selector's question mark includes one short linked citation; secondary coordinates explicitly cite the verified Glaser 2017 table rather than implying primary-paper recovery. A triangle opens each group. The whole-Moon field is labeled Sun rays.

Every setting, status and summary card has a nearby question mark. Hover or tap displays at most 25 words, generally one or two plain sentences. Shared word limiting also bounds future callers. Help content is capped at 232 logical pixels, approximately 260 pixels including window chrome, and cannot be resized wider. Longer context belongs only in the trimmed Explanations section. The phone drawer remains independently scrollable.

The shared dark palette uses one blue interaction accent, separate from the scientific yellow/blue body colors and green/red availability statuses. Heading, body, button and caption sizes are 24, 15, 14 and 12 logical pixels. Controls reserve at least 36 pixels of height on desktop and 44 on phones, with 10 pixels between items, 12 by 8 pixels of button padding and 8 to 10 pixel corner radii. A small circular question-mark glyph sits inside the full interaction target; it is muted until hovered. Summary surfaces use low-contrast fills without border boxes.

A question mark beside Sky height explains the local reference: 0 degrees is eye level over perfectly flat ground, positive is above it and negative below. This is an angle from the observer's horizontal plane, not an absolute elevation or a claim that the computed terrain lies at zero. Window height follows its short content instead of preserving an empty scroll area.

Explanations defines the horizon, angular panorama, visible solar area, direct Earth communication, clearance margin, shared UTC clock, full-period statistics, rays, camera modes, real lighting, earthshine, artificial fill and published data sources. It explicitly excludes relays, thermal predictions, landing safety, equipment performance and events missed between hourly samples.

## 2. State and immediate mode

The panel does not copy the scientific model. Site selection still indexes the lazy panorama cache. Time edits update the shared UTC epoch. Radio clearance still recalculates the same core statistics. Camera and lighting settings change only presentation. The selected-site ray default, Earth-following camera default, 0.5 degree radio margin and disabled display aids are unchanged.

`menu::section` uses `CollapsingState` with a stable identifier. `show_header` paints the triangle, title and real help button; `body` submits controls only when expanded. This is immediate-mode UI: functions describe this frame, while the UI context retains expansion state. The C++ equivalent is a callback-based widget builder plus a persistent state table, not retained widget ownership.

`globe::help::question` stores only a dialog-open boolean under the widget's stable identifier. `Window::open` writes back the close-button state. A width-bounded `on_hover_ui` supplies the same ordinary-language text to mouse users, while the explicit button supports touch. `brief_words` splits at word boundaries without cutting Unicode characters. Linked help shares the 25-word total across prose and caption; its publication link has a full-height touch target and does not display the raw URL. `annotated` reserves a narrow help column so long status text wraps rather than pushing the question mark off screen. No help action mutates scientific data.

The angular panorama now has a visible Sun/Earth/ground legend, degree signs on elevation and compass ticks, and an emphasized zero-height reference. Faint paths still represent the selected window, while summaries cover the labeled full period. Height stretching remains explicitly quantified. Green/red statuses describe sunlight and Earth clearance without promising electrical output or radio service. The scientific predicates and prepared results are unchanged.

The vector mark uses two filled circles: one blue-gray disk and a smaller background-colored cutout. It is decorative branding, not a lunar phase display. It uses logical screen units and needs no remote asset or font.

Earth view labels the approximate geometric phase and illuminated fraction. Near new Moon, the face toward Earth is naturally dark; the label does not promise that the faint earthshine is bright enough to see. Below 10 percent illuminated area, a Boost earthshine shortcut enables the existing 1,000 times display aid without changing scientific results. The camera is now five lunar radii from the center instead of 3.3, and projection magnification can decrease to 0.5 times. CPU picking, ribbon projection and both shader stages share this distance.

The globe paint area reserves at least 200 logical pixels vertically. Phase controls therefore expand the scrolling phone layout rather than letting the instruction legend cover the Moon.

## 3. Polar texture limit

The accurate low-cost choice is to limit globe magnification to 8 times, including wheel, pinch, buttons and Go to site. This replaces the previous 1024 times maximum and 128 times site target. At extreme magnification, longitude rows of the equirectangular normal/color maps fan into radial streaks near the pole. Enlarging those texels does not reveal reliable local terrain. The limit avoids presenting that enlargement as terrain detail without adding a new image source, guessed surface or extra download.

The initial 8 times cap alone still showed radial streaks in the visual check. Therefore the shader also suppresses unreliable polar normal and color detail: it smoothly transitions from the original maps at fifteen degrees from either pole to a schematic smooth sphere with the documented mean reflectance within ten degrees. The caption explicitly labels the polar cap schematic. This does not represent a geological plain or a new NASA measurement. No replacement surface data is invented or downloaded.

This is a display-resolution tradeoff, not a scientific resolution change. Very close published markers overlap at this scale; the Site list remains the reliable way to choose them individually. Positions are not displaced for readability. Detailed local terrain is represented by the computed panorama horizon. The conservative zoom cap and explicitly schematic fallback avoid presenting unresolved map detail as terrain.

## 4. Source and API map

The complete quotations below preserve the annotated Rust source. Existing panorama and globe quotations in the preceding guides contain the moved control methods. `settings`, `menu_contents`, `ray_controls`, `section`, `logo`, `explanations`, `question` and `annotated` are presentation helpers; their C++ equivalents are member functions, callback templates and ordinary drawing helpers. Data validation, ephemeris conversion, horizon calculations and statistics remain separate.

### 4.1. app/src/menu.rs

```rust
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
```

### 4.2. globe/src/help.rs

```rust
//! Shared plain-language help widgets, independent of scientific data or app ownership.

// Draw a hover explanation that can also be opened by touch; C++ immediate-mode helper.
pub fn question(ui: &mut egui::Ui, text: &str)
{
	question_contents(ui, text, None);
}

// Keep a short publication link inside the same bounded help surface; C++ optional linked caption.
pub fn question_with_link(ui: &mut egui::Ui, text: &str, caption: &str, url: &str)
{
	question_contents(ui, text, Some((caption, url)));
}

// Share word and width limits between hover, touch and linked help; C++ internal widget helper.
fn question_contents(ui: &mut egui::Ui, text: &str, link: Option<(&str, &str)>)
{
	// Bound touch windows and hover tooltips, including future callers; C++ capped text and width.
	let link = link.map(|(caption, url)| (brief_words(caption, 12), url));
	let words = link
		.as_ref()
		.map_or(0, |(caption, _)| caption.split_whitespace().count());
	let brief = brief_words(text, 25 - words);
	let width = (ui.ctx().content_rect().width() - 48.0).clamp(60.0, 232.0);
	let content = |ui: &mut egui::Ui| {
		ui.set_max_width(width);
		ui.add(egui::Label::new(&brief).wrap());
		if let Some((caption, url)) = &link
		{
			publication_link(ui, caption, url);
		}
	};
	// Namespace persistent state by this widget's location and explanation; C++ stable widget identifier.
	let id = ui.make_persistent_id(text);
	// Restore only presentation state, never scientific values; C++ optional bool with default.
	let mut open = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
	// A question mark supports both pointer hover and a phone tap; C++ button event.
	if icon(ui).on_hover_ui(content).clicked()
	{
		// Toggle the modeless explanation without consuming other controls; C++ bool assignment.
		open = !open;
	}
	// Keep short explanations inside a phone-sized window; C++ bounded modeless dialog.
	egui::Window::new("Help")
		.id(id)
		.open(&mut open)
		.collapsible(false)
		.auto_sized()
		.default_width(width)
		.max_width(width)
		.resizable(false)
		.show(ui.ctx(), content);
	// Preserve the close button or tap result for the next frame; C++ widget-state store.
	ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

// Use a full touch target while keeping a one-line citation visually small; C++ linked text button.
fn publication_link(ui: &mut egui::Ui, caption: &str, url: &str)
{
	let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y.max(36.0));
	if ui
		.add_sized(
			size,
			egui::Link::new(
				egui::RichText::new(caption)
					.small()
					.color(ui.visuals().hyperlink_color),
			),
		)
		.on_hover_text("Open publication")
		.clicked()
	{
		ui.ctx().open_url(egui::OpenUrl::new_tab(url));
	}
}

// Keep a small visual icon inside a full touch target; C++ custom button drawing.
fn icon(ui: &mut egui::Ui) -> egui::Response
{
	let size = ui.spacing().interact_size.y.max(36.0);
	let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
	let color = if response.hovered()
	{
		ui.visuals().hyperlink_color
	}
	else
	{
		ui.visuals().weak_text_color()
	};
	ui.painter()
		.circle_filled(rect.center(), 10.0, ui.visuals().window_fill);
	ui.painter().circle_stroke(
		rect.center(),
		10.0,
		egui::Stroke::new(0.7, color.gamma_multiply(0.5)),
	);
	ui.painter().text(
		rect.center(),
		egui::Align2::CENTER_CENTER,
		"?",
		egui::FontId::proportional(13.0),
		color,
	);
	response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// Keep every explanation within 25 words without cutting a Unicode character; C++ bounded word join.
fn brief_words(text: &str, limit: usize) -> String
{
	let mut words = text.split_whitespace();
	let mut result = words.by_ref().take(limit).collect::<Vec<_>>().join(" ");
	if words.next().is_some()
	{
		result.push_str("...");
	}
	result
}

// Keep a question mark adjacent to wrapping status text; C++ labeled-row helper.
pub fn annotated(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, explanation: &str)
{
	// Reserve a small help column even when the prose spans multiple lines; C++ row layout.
	ui.horizontal_top(|ui| {
		// Bound the label before submitting it so the help cannot be pushed off-screen; C++ width calculation.
		let width = (ui.available_width()
			- ui.spacing().interact_size.y.max(36.0)
			- ui.spacing().item_spacing.x)
			.max(40.0);
		// Use a vertical child to obtain automatic wrapping height; C++ child text container.
		ui.vertical(|ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.set_width(width);
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			ui.label(text);
		});
		// Draw the same accessible hover/tap control as the settings panel; C++ helper call.
		question(ui, explanation);
	});
}

// Exercise empty and long help without a GPU or a browser; C++ UI unit-test fixture.
#[cfg(test)]
mod tests
{
	// Import the helpers under test without exporting test state; C++ test namespace import.
	use super::*;
	#[test]
	fn short_empty_and_overlong_explanations()
	{
		assert_eq!(brief_words("", 25), "");
		assert_eq!(brief_words("Sun above hills.", 25), "Sun above hills.");
		let long = brief_words(&"Moon lumière ".repeat(30), 25);
		assert_eq!(long.split_whitespace().count(), 25);
		assert!(long.ends_with("..."));
		let caption = brief_words(&"Source ".repeat(20), 12);
		let text = brief_words(&"Moon ".repeat(30), 25 - caption.split_whitespace().count());
		assert_eq!(format!("{text} {caption}").split_whitespace().count(), 25);
	}

	#[test]
	fn compact_window_on_desktop_and_phone()
	{
		for (screen_width, linked) in [
			(1440.0, false),
			(412.0, false),
			(1440.0, true),
			(412.0, true),
		]
		{
			let ctx = egui::Context::default();
			let mut bounds = None;
			for _ in 0..2
			{
				let input = egui::RawInput {
					screen_rect: Some(egui::Rect::from_min_size(
						egui::Pos2::ZERO,
						egui::vec2(screen_width, 850.0),
					)),
					..Default::default()
				};
				let mut output = ctx.run_ui(input, |ui| {
					let text = if linked { "Choose a published point. Nearby places can have very different hills." } else { "Look around your chosen point. Gray hills hide the sky below their edge; yellow is Sun, blue is Earth, and heights are stretched." };
					let id = ui.make_persistent_id(text);
					ui.ctx().data_mut(|d| d.insert_temp(id, true));
					if linked
					{
						question_with_link(ui, text, "Coordinates via Glaser 2017, Table 1", "https://example.com");
					}
					else
					{
						question(ui, text);
					}
					bounds = ui.ctx().memory(|m| m.area_rect(id));
				});
				output.textures_delta.clear();
			}
			let rect = bounds.unwrap();
			assert!(rect.width() <= 260.0, "{rect:?}");
			assert!(rect.height() <= 180.0, "{rect:?}");
		}
	}
	// Verify rendering does not depend on application data or nonempty text; C++ test registration.
	#[test]
	fn empty_and_long_help()
	{
		// Build an isolated immediate-mode context; C++ UI test harness.
		let ctx = egui::Context::default();
		// Submit both empty and wrapping cases within one real layout pass; C++ frame callback.
		let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
			// Submit this immediate-mode widget or explanation; C++ UI helper call.
			question(ui, "");
			annotated(ui, "Status", &"Plain language explanation. ".repeat(30));
		});
		// This headless test intentionally uploads no font atlas to a GPU; C++ discard pending texture updates.
		output.textures_delta.clear();
	}
}
```

### 4.3. app/src/theme.rs

```rust
//! Shared dark palette, typography and responsive interaction sizes.

use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, TextStyle};

// Apply presentation only when the responsive size changes; C++ idempotent style initialization.
pub(crate) fn apply(ctx: &egui::Context, phone: bool)
{
	let height = if phone { 44.0 } else { 36.0 };
	if ctx.style_of(egui::Theme::Dark).spacing.interact_size.y == height
	{
		return;
	}
	ctx.style_mut_of(egui::Theme::Dark, |style| {
		style.spacing.interact_size = egui::vec2(height, height);
		style.spacing.item_spacing = egui::vec2(10.0, 10.0);
		style.spacing.button_padding = egui::vec2(12.0, 8.0);
		style.spacing.window_margin = egui::Margin::same(12);
		style.spacing.menu_margin = egui::Margin::same(8);
		for (kind, size) in [
			(TextStyle::Heading, 24.0),
			(TextStyle::Body, 15.0),
			(TextStyle::Button, 14.0),
			(TextStyle::Small, 12.0),
		]
		{
			style.text_styles.insert(kind, FontId::proportional(size));
		}
		palette(&mut style.visuals);
	});
}

// Reserve blue for interaction, retaining semantic Sun/Earth and status colors; C++ palette setup.
fn palette(visuals: &mut egui::Visuals)
{
	let accent = Color32::from_rgb(124, 193, 244);
	visuals.panel_fill = Color32::from_rgb(16, 21, 28);
	visuals.window_fill = Color32::from_rgb(22, 29, 38);
	visuals.extreme_bg_color = Color32::from_rgb(9, 13, 18);
	visuals.override_text_color = Some(Color32::from_rgb(225, 232, 240));
	visuals.hyperlink_color = accent;
	visuals.selection.bg_fill = Color32::from_rgb(35, 76, 109);
	visuals.selection.stroke = Stroke::new(1.0, accent);
	visuals.window_corner_radius = CornerRadius::same(10);
	visuals.menu_corner_radius = CornerRadius::same(10);
	visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(44, 57, 72));
	for widget in [
		&mut visuals.widgets.noninteractive,
		&mut visuals.widgets.inactive,
		&mut visuals.widgets.hovered,
		&mut visuals.widgets.active,
		&mut visuals.widgets.open,
	]
	{
		widget.corner_radius = CornerRadius::same(8);
		widget.bg_stroke = Stroke::NONE;
		widget.bg_fill = Color32::from_rgb(29, 39, 51);
		widget.weak_bg_fill = Color32::from_rgb(23, 31, 42);
	}
	visuals.widgets.hovered.bg_fill = Color32::from_rgb(39, 59, 79);
	visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, accent);
	visuals.widgets.active.bg_fill = visuals.selection.bg_fill;
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn responsive_sizes_and_stable_palette()
	{
		let ctx = egui::Context::default();
		for phone in [false, true, true, false]
		{
			apply(&ctx, phone);
			let style = ctx.style_of(egui::Theme::Dark);
			assert_eq!(
				style.spacing.interact_size.y,
				if phone { 44.0 } else { 36.0 }
			);
			assert_eq!(style.visuals.window_corner_radius, CornerRadius::same(10));
			assert_eq!(style.text_styles[&TextStyle::Body].size, 15.0);
			assert_eq!(style.visuals.panel_fill, Color32::from_rgb(16, 21, 28));
		}
	}
}
```

### 4.4. globe/src/phase.rs

```rust
use super::*;

// Compute geometric lunar phase from real Moon-fixed directions; C++ checked vector geometry.
fn lunar_phase(sun: [f32; 3], earth: [f32; 3]) -> Option<(f32, f32, bool)>
{
	let norm =
		(sun.iter().map(|x| x * x).sum::<f32>() * earth.iter().map(|x| x * x).sum::<f32>()).sqrt();
	if !norm.is_finite() || norm < 1e-6
	{
		return None;
	}
	let cosine = (sun.iter().zip(earth).map(|(s, e)| s * e).sum::<f32>() / norm).clamp(-1.0, 1.0);
	let waxing = sun[2] * earth[0] - sun[0] * earth[2] < 0.0;
	Some((cosine.acos().to_degrees(), (1.0 + cosine) / 2.0, waxing))
}

impl Globe
{
	// Keep a naturally dark phase distinct from missing data or eclipses; C++ presentation helper.
	pub(super) fn phase_status(&mut self, ui: &mut egui::Ui, light: Lighting)
	{
		let Some((angle, fraction, waxing)) = lunar_phase(light.sun, light.earth)
		else
		{
			return;
		};
		let phase = phase_label(angle, fraction, waxing);
		help::annotated(
			ui,
			format!("{phase} · {:.1}% lit", fraction * 100.0),
			"Near new Moon, the face toward Earth is dark. This is a phase, not an eclipse; brightness differs from the lit percentage.",
		);
		if fraction < 0.1
		{
			ui.horizontal(|ui| {
				let label = if self.boosted { "Earthshine boosted" } else { "Boost earthshine" };
				if ui.add_enabled(!self.boosted, egui::Button::new(label)).clicked()
				{
					self.boosted = true;
				}
				help::question(ui, "Brighten faint sunlight reflected by Earth to reveal the dark face. This changes the picture, not the Sun or scientific results.");
			});
		}
	}
}

// Name approximate phases without equating their illuminated area with brightness; C++ classification.
fn phase_label(angle: f32, fraction: f32, waxing: bool) -> &'static str
{
	if angle > 168.0
	{
		"Near new Moon"
	}
	else if angle < 12.0
	{
		"Near full Moon"
	}
	else if (angle - 90.0).abs() < 2.0
	{
		"Half Moon"
	}
	else if fraction < 0.5
	{
		if waxing
		{
			"Waxing crescent"
		}
		else
		{
			"Waning crescent"
		}
	}
	else if waxing
	{
		"Waxing gibbous"
	}
	else
	{
		"Waning gibbous"
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn phases_normalized_and_invalid()
	{
		for (sun, expected) in [
			([0.0, 0.0, 2.0], 1.0),
			([0.0, 0.0, -2.0], 0.0),
			([2.0, 0.0, 0.0], 0.5),
		]
		{
			let (angle, fraction, _) = lunar_phase(sun, [0.0, 0.0, 3.0]).unwrap();
			assert!((fraction - expected).abs() < 1e-6);
			assert!((angle.to_radians().cos() - (2.0 * fraction - 1.0)).abs() < 1e-6);
		}
		assert!(lunar_phase([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap().2);
		assert!(!lunar_phase([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap().2);
		assert!(lunar_phase([0.0; 3], [1.0; 3]).is_none());
		assert!(lunar_phase([f32::NAN; 3], [1.0; 3]).is_none());
		assert!(lunar_phase([1.0; 3], [f32::INFINITY; 3]).is_none());
		assert_eq!(phase_label(180.0, 0.0, true), "Near new Moon");
		assert_eq!(phase_label(0.0, 1.0, false), "Near full Moon");
		assert_eq!(phase_label(90.0, 0.5, false), "Half Moon");
		assert_eq!(phase_label(120.0, 0.25, true), "Waxing crescent");
		assert_eq!(phase_label(120.0, 0.25, false), "Waning crescent");
		assert_eq!(phase_label(60.0, 0.75, true), "Waxing gibbous");
		assert_eq!(phase_label(60.0, 0.75, false), "Waning gibbous");
	}
}
```

## 5. Alphabetical presentation API index

- `abs`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `acos`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `active_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `add`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `add_enabled`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `add_sized`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `add_space`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `advance_upgrade`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `advanced_sections`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `allocate_exact_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `allocate_ui`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `allow_double_click_reset`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_drag`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_scroll`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_zoom`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `and_then`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `animate_camera`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `annotated`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `announce`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `any`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `apply`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `Arc::clone`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Arc::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `as_chunks`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `as_deref`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `as_mut`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `as_raw`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `as_ref`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `asin`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `aspect_ratio`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `assert!`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `assert_eq!`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `assets::label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `assets::load`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `at`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `atan2`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `attach_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `auto_sized`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `available_height`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `available_size`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `available_width`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `begin_frame`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `begin_upgrade`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_attrib_location`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `bind_buffer`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_polar`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_vertex_array`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `blend_func`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `body`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `bottom`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `brief_words`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `browser_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `buffer_data_u8_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `build_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_ray_layout`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `button`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `by_ref`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `bytemuck::cast_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `camera_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `card_grid`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `center`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `central_view`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `chain`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `changed`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `checkbox`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `circle_filled`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `circle_stroke`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `clamp`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `clear`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `clear_depth_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `clicked`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `clone`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `collapsible`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `collect`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `color`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Color32::from_rgb`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `columns`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `compact_window_on_desktop_and_phone`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `compile_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `compile_stages`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `contains`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `content_rect`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `corner_radius`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `CornerRadius::same`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `cos`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `count`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `crate::sky::show`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `create_buffer`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_texture`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `create_vertex_array`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `ctx`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `cull_face`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `cursor`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `data`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `data_mut`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `Default::default`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `default_pos`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `default_size`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `default_width`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `default_x_bounds`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `delete_buffer`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_program`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_shader`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delete_vertex_array`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `delta`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `depth_func`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `depth_mask`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `desktop_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `destroy`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `detach_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `directions_at`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `disable`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `distance_sq`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `dragged`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `drain`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `draw_arrays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `draw_elements`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `draw_marker`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `earth_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `eframe::egui::CentralPanel::default`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `eframe::egui::Layout::top_down`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `eframe::egui::ScrollArea::vertical`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `eframe::egui::vec2`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `egui::Button::new`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `egui::CentralPanel::default`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::collapsing_header::CollapsingState::load_with_default_open`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Color32::from_black_alpha`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Color32::from_rgb`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::ComboBox::from_id_salt`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `egui::Context::default`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::FontId::proportional`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Grid::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::Id::unique`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Label::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Link::new`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `egui::Margin::same`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `egui::OpenUrl::new_tab`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `egui::Panel::left`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Panel::top`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::RawInput::default`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Rect::from_min_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::RichText::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::ScrollArea::vertical`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Sense::click`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `egui::Sense::click_and_drag`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Sense::hover`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Stroke::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::vec2`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Window::new`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui_glow::CallbackFn::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui_glow::ShaderVersion::get`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `empty`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `enable`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `enable_vertex_attrib_array`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `end_row`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `ends_with`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `enumerate`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `eprintln!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `exp`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `expand`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `explanations`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `extend`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `f64::from`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `field::illuminated_points`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `fill`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `filter`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `filter_map`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `find`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `flat_map`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `flatten`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `FontId::proportional`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `format!`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `frame`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `front_face`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `gamma_multiply`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `generate_mipmap`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `get_error`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `get_parameter_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `get_program_info_log`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_program_link_status`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_shader_compile_status`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_shader_info_log`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `get_temp`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `get_uniform_location`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `gl`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `globe::help::annotated`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `globe::help::question`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `globe::help::question_with_link`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `Globe::paint_rectangle`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `glow::PixelUnpackData::Slice`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `has_load_error`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `has_visible_moon`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `heading`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `height`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `help::annotated`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `help::question`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `hline`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `HLine::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `horizon_at`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `horizontal`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `horizontal_top`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `horizontal_wrapped`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `hovered`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `icon`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `id`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `include_str!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `include_y`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `initialize`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `inner_margin`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `input_mut`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `insert`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `insert_temp`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `interact_pointer_pos`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `into`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `into_iter`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_finite`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `is_new_shader_interface`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `is_none`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `is_positive`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `is_some`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_some_and`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `iter`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `join`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `last`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `left_top`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `legend`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Legend::default`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `len`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `light_sources`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `light_uniforms`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `lighting_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `line`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Line::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `link_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `ln`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `load_error`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `loading_status`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `lock`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `logo`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `lunar_phase`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `main_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `make_persistent_id`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `map`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `map_or`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `margin`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `math::direction`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::earthshine_ratio`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::local_direction`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `max`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `max_height`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `max_width`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `menu_contents`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `min`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `model`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `native_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `native_pixels`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `new`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `next`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `num_columns`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `on_hover_cursor`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `on_hover_text`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `on_hover_ui`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `open`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `open_url`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `or`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `or_else`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `orbit_input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_markers`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint_opaque`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_rectangle`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint_site_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_surface_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `painter`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `painter_at`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `palette`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `panorama::Panorama::new`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `pending_label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `phase_label`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `phase_status`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `phases_normalized_and_invalid`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `phone_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `pixel_store_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `pixels_per_point`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Plot::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `points`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Points::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `polar::advance`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `polar::wanted`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `powi`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `project`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `projection`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `publication_link`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `push`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `push_str`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `question`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `question_contents`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `question_with_link`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `queue_gl`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `radio_controls`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `radio_value`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `radius`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `ray_controls`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `rect_filled`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `rem_euclid`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `Renderer::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `repeat`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `replace`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `request_repaint`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `resizable`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `responsive_sizes_and_stable_palette`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `ribbon`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `run_ui`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `scene_lighting`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `scene_rays`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `section`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `selectable_value`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `selected_text`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `Self::announce`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::begin_frame`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Self::browser_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `Self::draw_marker`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::empty`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `Self::legend`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::light_uniforms`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Self::native_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `Self::native_pixels`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `Self::paint_rectangle`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Self::process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `set_margin_fraction`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `set_max_width`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `set_scene`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `set_time`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `set_width`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `settings`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `shader_source`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `short_empty_and_overlong_explanations`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `short_phone_layout_grows_past_phase_controls`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `show`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `show_header`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `show_panorama`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `show_ui`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `site`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `site_camera`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `site_time_sections`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `size`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `size_range`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `small`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `solar_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `source_link`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `spacing`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `split_whitespace`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `sqrt`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `starts_with`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `std::array::from_fn`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Stroke::new`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `strong`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `style_mut`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `style_mut_of`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `style_of`: [annotated app/src/theme.rs](#43-appsrcthemers)
- `sum`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `summary_card`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `summary_cards`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `surface_ray_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `synchronize_scene`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `take`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `technical`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `technical_controls`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `technical_items`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `tex_image_2d`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `tex_image_2d_with_html_image`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `tex_parameter_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `text`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `texture_controls`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `then`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `time`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `time_controls`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `to_degrees`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `to_radians`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `top`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `truncate`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_1_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_1_i32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_3_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `unwrap`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `unwrap_or`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `unwrap_or_else`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `update`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `update_polar`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `update_sites`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `upload_browser_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `upload_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `use_program`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `vec!`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Vec::from`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `Vec::new`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `version_declaration`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `vertex_attrib_pointer_f32`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `vertical`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `viewport_in_pixels`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `visuals`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `vscroll`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `weak_text_color`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `web_sys::console::log_1`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `width`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `with_layout`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `wrap`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `wrap_mode`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `x_axis_formatter`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `x_axis_label`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `x_grid_spacer`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `y_axis_label`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `zip`: [annotated globe/src/phase.rs](11-menu-and-help.md#44-globesrcphasers)
- `zoom_delta`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
