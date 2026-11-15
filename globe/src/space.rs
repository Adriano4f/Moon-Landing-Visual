use crate::{Lighting, math};
use egui::{Color32, Pos2, Vec2};

fn ecliptic(direction: [f32; 3]) -> Option<Vec2>
{
	let (s, c) = 23.439_f32.to_radians().sin_cos();
	let v = egui::vec2(direction[2], -(c * direction[0] + s * direction[1]));
	let length = v.length();
	(length.is_finite() && length > 1e-6).then(|| v / length)
}

fn directions(rotation: math::Mat3, light: Option<Lighting>) -> Option<[Vec2; 2]>
{
	let light = light?;
	let moon = math::transform(rotation, light.earth.map(|x| -x));
	let sun = math::transform(rotation, light.sun);
	Some([ecliptic(moon)?, ecliptic(sun)?])
}

fn lit_half(center: Pos2, direction: Vec2, radius: f32) -> Vec<Pos2>
{
	let across = egui::vec2(-direction.y, direction.x);
	(0..=24)
		.map(|i| {
			let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 24.0;
			center + radius * (direction * a.cos() + across * a.sin())
		})
		.collect()
}

pub(super) fn paint(
	ui: &mut egui::Ui,
	globe: egui::Rect,
	rotation: math::Mat3,
	light: Option<Lighting>,
)
{
	let width = if globe.width() < 450.0 { 140.0 } else { 168.0 };
	let rect = egui::Rect::from_min_size(
		globe.right_top() + egui::vec2(-width - 6.0, 6.0),
		egui::vec2(width, 160.0),
	);
	ui.interact(
		rect,
		ui.scope_id().with("space-inset"),
		egui::Sense::click_and_drag(),
	);
	let painter = ui.painter_at(rect);
	painter.rect_filled(rect, 8.0, Color32::from_rgba_unmultiplied(16, 23, 31, 245));
	let text = |position, anchor, label: &str, size, color| {
		painter.text(
			position,
			anchor,
			label,
			egui::FontId::proportional(size),
			color,
		);
	};
	text(
		rect.center_top() + egui::vec2(0.0, 8.0),
		egui::Align2::CENTER_TOP,
		"From space",
		12.0,
		Color32::WHITE,
	);
	text(
		rect.center_bottom() - egui::vec2(0.0, 7.0),
		egui::Align2::CENTER_BOTTOM,
		"Not to scale",
		10.0,
		Color32::GRAY,
	);
	let center = rect.center() + egui::vec2(0.0, 5.0);
	let radius = 38.0;
	painter.circle_stroke(
		center,
		radius,
		egui::Stroke::new(1.0, Color32::from_gray(85)),
	);
	painter.circle_filled(center, 6.0, Color32::from_rgb(90, 175, 250));
	text(
		center + egui::vec2(0.0, 10.0),
		egui::Align2::CENTER_TOP,
		"Earth",
		10.0,
		Color32::from_rgb(140, 200, 255),
	);
	let Some([moon, sun]) = directions(rotation, light)
	else
	{
		text(
			rect.center_top() + egui::vec2(0.0, 30.0),
			egui::Align2::CENTER_TOP,
			"Sky unavailable",
			10.0,
			Color32::GRAY,
		);
		return;
	};
	let point = center + moon * radius;
	painter.circle_filled(point, 6.0, Color32::from_gray(55));
	painter.add(egui::Shape::convex_polygon(
		lit_half(point, sun, 6.0),
		Color32::from_rgb(240, 239, 216),
		egui::Stroke::NONE,
	));
	let yellow = Color32::from_rgb(245, 215, 85);
	painter.arrow(
		center + sun * 47.0,
		sun * 14.0,
		egui::Stroke::new(1.5, yellow),
	);
	let label = center + sun * 58.0 + egui::vec2(-sun.y, sun.x) * 12.0;
	let label = Pos2::new(
		label.x.clamp(rect.left() + 15.0, rect.right() - 15.0),
		label.y.clamp(rect.top() + 29.0, rect.bottom() - 24.0),
	);
	text(label, egui::Align2::CENTER_CENTER, "Sun", 10.0, yellow);
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn equatorial_to_ecliptic_and_invalid_directions()
	{
		assert_eq!(ecliptic([0.0, 0.0, 1.0]), Some(egui::vec2(1.0, 0.0)));
		let (s, c) = 23.439_f32.to_radians().sin_cos();
		let y = ecliptic([c, s, 0.0]).unwrap();
		assert!((y - egui::vec2(0.0, -1.0)).length() < 1e-6);
		for v in [[0.0; 3], [f32::NAN, 0.0, 1.0], [f32::INFINITY, 0.0, 1.0]]
		{
			assert!(ecliptic(v).is_none());
		}
		assert!(directions(math::IDENTITY, None).is_none());
	}
	#[test]
	fn moon_is_opposite_earth_and_illuminated_toward_sun()
	{
		let light = Lighting {
			sun: [0.0, 0.0, 1.0],
			earth: [0.0, 0.0, 1.0],
			ratio: 0.0,
			boosted: false,
			fill: false,
		};
		let [moon, sun] = directions(math::IDENTITY, Some(light)).unwrap();
		assert!((moon + sun).length() < 1e-6);
		for point in lit_half(Pos2::ZERO, sun, 6.0)
		{
			assert!(point.to_vec2().dot(sun) > -1e-5);
		}
	}
}
