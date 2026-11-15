//! Shared plain-language help widgets, independent of scientific data or app ownership.

// Draw a hover explanation that can also be opened by touch; C++ immediate-mode helper.
pub fn question(ui: &mut egui::Ui, text: &str)
{
	// Bound touch windows and hover tooltips, including future callers; C++ capped text and width.
	let brief = brief(text);
	let width = (ui.ctx().content_rect().width() - 48.0).clamp(60.0, 232.0);
	// Namespace persistent state by this widget's location and explanation; C++ stable widget identifier.
	let id = ui.make_persistent_id(text);
	// Restore only presentation state, never scientific values; C++ optional bool with default.
	let mut open = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
	// A question mark supports both pointer hover and a phone tap; C++ button event.
	if icon(ui)
		.on_hover_ui(|ui| {
			ui.set_max_width(width);
			ui.add(egui::Label::new(&brief).wrap());
		})
		.clicked()
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
		.show(ui.ctx(), |ui| {
			// Wrap to the available screen rather than widening the application; C++ text layout policy.
			ui.set_max_width(width);
			// Display static, ordinary-language help without executing its content; C++ text widget.
			ui.add(egui::Label::new(&brief).wrap());
		});
	// Preserve the close button or tap result for the next frame; C++ widget-state store.
	ui.ctx().data_mut(|d| d.insert_temp(id, open));
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
fn brief(text: &str) -> String
{
	let mut words = text.split_whitespace();
	let mut result = words.by_ref().take(25).collect::<Vec<_>>().join(" ");
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
		assert_eq!(brief(""), "");
		assert_eq!(brief("Sun above hills."), "Sun above hills.");
		let long = brief(&"Moon lumière ".repeat(30));
		assert_eq!(long.split_whitespace().count(), 25);
		assert!(long.ends_with("..."));
	}

	#[test]
	fn compact_window_on_desktop_and_phone()
	{
		for screen_width in [1440.0, 412.0]
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
					let text = "Look around your chosen point. Gray hills hide the sky below their edge; yellow is Sun, blue is Earth, and heights are stretched.";
					let id = ui.make_persistent_id(text);
					ui.ctx().data_mut(|d| d.insert_temp(id, true));
					question(ui, text);
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
