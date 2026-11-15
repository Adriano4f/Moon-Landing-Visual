# Grouped settings and explanations

## 1. Reading the interface

The small crescent and Karaya Explorer name remain visible in the header. On a desktop, settings occupy a bounded left sidebar. On a phone, Menu opens a scrollable floating drawer, so opening settings does not squeeze the stacked globe and panorama into a narrow strip. Close menu or the drawer close button hides it again.

Site and Time start expanded. View / camera, Lighting, Rays, Radio margin, Technical and Explanations start collapsed. A triangle opens each group. Every setting has a nearby question mark. Hovering gives a short explanation; clicking or tapping opens the same text in a closable window. Statuses and all four summary cards have their own question marks. These describe both the meaning and the limits of the result.

Explanations defines the horizon, angular panorama, visible solar area, direct Earth communication, clearance margin, shared UTC clock, full-period statistics, rays, camera modes, real lighting, earthshine, artificial fill and published data sources. It explicitly excludes relays, thermal predictions, landing safety, equipment performance and events missed between hourly samples.

## 2. State and immediate mode

The panel does not copy the scientific model. Site selection still indexes the lazy panorama cache. Time edits update the shared UTC epoch. Radio clearance still recalculates the same core statistics. Camera and lighting settings change only presentation. The selected-site ray default, Earth-following camera default, 0.5 degree radio margin and disabled display aids are unchanged.

`menu::section` uses `CollapsingState` with a stable identifier. `show_header` paints the triangle, title and real help button; `body` submits controls only when expanded. This is immediate-mode UI: functions describe this frame, while the UI context retains expansion state. The C++ equivalent is a callback-based widget builder plus a persistent state table, not retained widget ownership.

`globe::help::question` stores only a dialog-open boolean under the widget's stable identifier. `Window::open` writes back the close-button state. `on_hover_text` supplies the same ordinary-language text to mouse users, while the explicit button supports touch. `annotated` reserves a narrow help column so long status text wraps rather than pushing the question mark off screen. No help action mutates scientific data.

The vector mark uses two filled circles: one blue-gray disk and a smaller background-colored cutout. It is decorative branding, not a lunar phase display. It uses logical screen units and needs no remote asset or font.

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
```

### 4.2. globe/src/help.rs

```rust
//! Shared plain-language help widgets, independent of scientific data or app ownership.

// Draw a hover explanation that can also be opened by touch; C++ immediate-mode helper.
pub fn question(ui: &mut egui::Ui, text: &str)
{
	// Namespace persistent state by this widget's location and explanation; C++ stable widget identifier.
	let id = ui.make_persistent_id(text);
	// Restore only presentation state, never scientific values; C++ optional bool with default.
	let mut open = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
	// A question mark supports both pointer hover and a phone tap; C++ button event.
	if ui.small_button("?").on_hover_text(text).clicked()
	{
		// Toggle the modeless explanation without consuming other controls; C++ bool assignment.
		open = !open;
	}
	// Keep long explanations readable and scrollable on a phone; C++ modeless dialog.
	egui::Window::new("Explanation")
		.id(id)
		.open(&mut open)
		.default_width(280.0)
		.vscroll(true)
		.show(ui.ctx(), |ui| {
			// Wrap to the available screen rather than widening the application; C++ text layout policy.
			ui.set_max_width(280.0);
			// Display static, ordinary-language help without executing its content; C++ text widget.
			ui.label(text);
		});
	// Preserve the close button or tap result for the next frame; C++ widget-state store.
	ui.ctx().data_mut(|d| d.insert_temp(id, open));
}

// Keep a question mark adjacent to wrapping status text; C++ labeled-row helper.
pub fn annotated(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>, explanation: &str)
{
	// Reserve a small help column even when the prose spans multiple lines; C++ row layout.
	ui.horizontal_top(|ui| {
		// Bound the label before submitting it so the help cannot be pushed off-screen; C++ width calculation.
		let width = (ui.available_width() - 28.0).max(40.0);
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

## 5. Alphabetical presentation API index

- `active_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `add`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `add_enabled`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `add_space`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `advance_upgrade`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `allocate_exact_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `allocate_ui`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `allow_double_click_reset`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_drag`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_scroll`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `allow_zoom`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `and_then`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `animate_camera`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `annotated`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `announce`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `any`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `Arc::clone`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Arc::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `as_chunks`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `as_deref`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `as_mut`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `as_raw`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `as_ref`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `asin`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `aspect_ratio`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `assets::load`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `at`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `atan2`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `attach_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `available_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `available_width`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `begin_frame`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `begin_upgrade`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_attrib_location`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `bind_buffer`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_texture`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `bind_vertex_array`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `blend_func`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `body`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `browser_filter`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `buffer_data_u8_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `build_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_ray_layout`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `build_sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `button`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
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
- `clamp`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `clear`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `clear_depth_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `clicked`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `clone`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `collapsible`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `collect`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `color`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Color32::from_gray`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `columns`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `compile_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `compile_stages`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `contains`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `content_rect`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `count`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `crate::sky::show`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `create_buffer`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_program`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_shader`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `create_texture`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `create_vertex_array`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `ctx`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `cull_face`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `data`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `data_mut`: [annotated globe/src/help.rs](#42-globesrchelprs)
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
- `egui::Button::new`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `egui::collapsing_header::CollapsingState::load_with_default_open`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Color32::from_black_alpha`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Color32::from_rgb`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::ComboBox::from_id_salt`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `egui::Context::default`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `egui::FontId::proportional`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Frame::group`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::Grid::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::Id::unique`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Panel::left`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Panel::top`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::RawInput::default`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `egui::Rect::from_min_size`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::RichText::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `egui::ScrollArea::vertical`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Sense::click_and_drag`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Sense::hover`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui::Stroke::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::vec2`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui::Window::new`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `egui_glow::CallbackFn::new`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `egui_glow::ShaderVersion::get`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `empty`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `empty_and_long_help`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `enable`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `enable_vertex_attrib_array`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `end_row`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
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
- `format!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `frame`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `front_face`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
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
- `glow::PixelUnpackData::Slice`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `heading`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `height`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `help::annotated`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `help::question`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `hline`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `HLine::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `horizon_at`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `horizontal`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `horizontal_top`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `horizontal_wrapped`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `hovered`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `id`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `include_str!`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `include_y`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `initialize`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `input_mut`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `insert_temp`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `interact_pointer_pos`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `into`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `into_iter`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_new_shader_interface`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `is_none`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_positive`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `is_some`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `is_some_and`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `iter`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `last`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `left_top`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `legend`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Legend::default`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `len`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
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
- `main_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `make_persistent_id`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `map`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `map_or`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
- `margin`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `math::direction`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::earthshine_ratio`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::local_direction`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `math::sphere`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `max`: [annotated globe/src/camera.rs](04-globe-code.md#424-globesrccamerars)
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
- `on_hover_text`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `open`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `or_else`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `orbit_input`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_markers`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `paint_opaque`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_site_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `paint_surface_rays`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `painter`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `painter_at`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `panorama::Panorama::new`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `pending_label`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `phone_views`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `pixel_store_i32`: [annotated globe/src/renderer/textures.rs](04-globe-code.md#422-globesrcrenderertexturesrs)
- `pixels_per_point`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `Plot::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `points`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `Points::new`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `powi`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `project`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `projection`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `push`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `question`: [annotated globe/src/help.rs](#42-globesrchelprs)
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
- `ribbon`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `run_ui`: [annotated globe/src/help.rs](#42-globesrchelprs)
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
- `Self::process_textures`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `set_margin_fraction`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `set_max_width`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `set_scene`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `set_time`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `set_width`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `settings`: [annotated app/src/menu.rs](#41-appsrcmenurs)
- `shader_source`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
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
- `small_button`: [annotated globe/src/help.rs](#42-globesrchelprs)
- `solar_status`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `spacing`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `std::array::from_fn`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `strong`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `style`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `style_mut`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
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
- `to_radians`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `truncate`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_1_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_1_i32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_3_f32`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_matrix_3_f32_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `uniform_matrix_4_f32_slice`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `unwrap`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `unwrap_or`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `unwrap_or_else`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
- `update`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
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
- `web_sys::console::log_1`: [annotated globe/src/renderer/setup.rs](04-globe-code.md#421-globesrcrenderersetuprs)
- `width`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `with_layout`: [annotated app/src/layout.rs](04-globe-code.md#425-appsrclayoutrs)
- `x_axis_formatter`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `x_axis_label`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `x_grid_spacer`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `y_axis_label`: [annotated app/src/panorama/view.rs](04-globe-code.md#426-appsrcpanoramaviewrs)
- `zip`: [annotated globe/src/renderer.rs](04-globe-code.md#420-globesrcrendererrs)
- `zoom_delta`: [annotated globe/src/view.rs](04-globe-code.md#423-globesrcviewrs)
