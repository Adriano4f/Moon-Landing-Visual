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
		widget.bg_stroke = Stroke::new(1.0, Color32::from_rgb(74, 89, 108));
		widget.bg_fill = Color32::from_rgb(42, 52, 66);
		widget.weak_bg_fill = Color32::from_rgb(42, 52, 66);
	}
	visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 73, 94);
	visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(55, 73, 94);
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
			assert_eq!(style.visuals.widgets.inactive.bg_fill, Color32::from_rgb(42, 52, 66));
			assert_eq!(style.visuals.widgets.inactive.bg_stroke.width, 1.0);
		}
	}
}
