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
