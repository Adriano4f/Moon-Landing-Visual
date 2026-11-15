//! A clipped angular panorama: draw sky and bodies first, then terrain as an occluder.

// Borrow validated data without copying hourly arrays; C++ project declaration.
use crate::data::SiteData;
// Use egui's own tessellator and painter on native and web; C++ drawing-library types.
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
// Reuse the verified spherical destination construction for angular circles; C++ scientific declarations.
use lunar_core::{MOON_RADIUS_M, SUN_RADIUS_DEG, destination};
// Describe a selected half-open sample window; C++ pair of iterator offsets.
use std::ops::Range;

// Generate an angular circle on the sky sphere, unwrapped around its center; C++ vector-returning geometry helper.
fn disk(az: f64, el: f64, radius: f64) -> Vec<[f64; 2]>
{
	// Reject invalid or pole-crossing inputs instead of generating nonfinite mesh vertices; C++ preconditions.
	if !az.is_finite()
		|| !el.is_finite()
		|| !radius.is_finite()
		|| radius <= 0.0
		|| el.abs() + radius >= 90.0
	{
		return Vec::new();
	}
	// Sixty-four boundary pieces approximate the circle without changing its angular radius; C++ counted transform.
	(0..64)
		.map(|i| {
			// Sky elevation acts as spherical latitude, azimuth as longitude; C++ coordinate conversion.
			let (e, a) = destination(
				el,
				az,
				f64::from(i) * 360.0 / 64.0,
				radius.to_radians() * MOON_RADIUS_M,
			);
			// Keep adjacent vertices close across north; C++ positive-modulo unwrap.
			[az + (a - az + 180.0).rem_euclid(360.0) - 180.0, e]
		})
		.collect()
}

// Map angular coordinates to a bounded screen rectangle; C++ affine point transform.
fn project(rect: Rect, az: f64, el: f64) -> Pos2
{
	// Screen Y points down, opposite elevation; C++ two-component float construction.
	egui::pos2(
		rect.left() + rect.width() * az as f32 / 360.0,
		rect.bottom() - rect.height() * (el as f32 + 10.0) / 20.0,
	)
}

// Show a full 360-degree field with a labeled vertical stretch; C++ free drawing function.
pub(crate) fn show(ui: &mut egui::Ui, data: &SiteData, time:i64, range: Range<usize>)
{
	// Name body colors without requiring a legend click; C++ caption.
	legend(ui);
	let id = ui.scope_id().with("sky-true-scale");
	let mut true_scale = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
	ui.checkbox(&mut true_scale, "True scale 1:1")
		.on_hover_text("One vertical degree occupies the same screen distance as one horizontal degree.");
	ui.ctx().data_mut(|d| d.insert_temp(id, true_scale));
	let maximum = if ui.ctx().content_rect().width() < 700.0
	{
		210.0
	}
	else
	{
		(ui.available_height() - 240.0).clamp(75.0, 260.0)
	};
	let height = plot_height(ui.available_width(), maximum, true_scale);
	let (response, painter) = ui.allocate_painter(
		egui::vec2(ui.available_width(), height),
		egui::Sense::hover(),
	);
	// Reserve labeled margins rather than clipping compass/elevation text; C++ inset rectangle.
	let rect = Rect::from_min_max(
		response.rect.min + egui::vec2(30.0, 6.0),
		response.rect.max - egui::vec2(12.0, 24.0),
	);
	// Keep the scene physically black above the ground; C++ filled rectangle.
	painter.rect_filled(rect, 0.0, Color32::BLACK);
	// Limit every path, body and ground triangle to the same field; C++ clipped drawing context.
	let sky = painter.with_clip_rect(rect);
	guides(&painter, &sky, rect);
	bodies(&sky, rect, data, time, range);
	terrain(&sky, rect, data);
	// Calculate the actual nonuniform angular display scale from the allocated rectangle; C++ ratio of pixels per degree.
	let stretch = (rect.height() / 20.0) / (rect.width() / 360.0);
	// Warn about vertical exaggeration rather than depicting the ellipses as true visual aspect ratios; C++ caption.
	if !true_scale
	{
		ui.small(format!("Vertical scale ×{stretch:.1}"));
	}
	// Preserve explicit missing-terrain labeling for legacy data; C++ optional guard.
	if data.horizon.is_none()
	{
		ui.colored_label(
			Color32::LIGHT_RED,
			"Flat horizon: hills are not loaded yet.",
		);
	}
}

fn plot_height(width: f32, maximum: f32, true_scale: bool) -> f32
{
	if true_scale
	{
		(width - 42.0).max(1.0) / 18.0 + 30.0
	}
	else if maximum.is_finite()
	{
		maximum.max(31.0)
	}
	else
	{
		210.0
	}
}

// Keep colors, units and the occluding ground explicit beside the graph; C++ legend row.
fn legend(ui: &mut egui::Ui)
{
	ui.horizontal_wrapped(|ui| {
		ui.small("Sky height (°)");
		globe::help::question(
			ui,
			"0° is level with perfectly flat ground; positive is above, negative below.",
		);
		ui.colored_label(Color32::YELLOW, "● Sun");
		ui.colored_label(Color32::LIGHT_BLUE, "● Earth");
		ui.colored_label(Color32::GRAY, "■ Terrain");
	});
}

// Compile pure geometry tests without starting a window or a GL context; C++ test translation unit.
#[cfg(test)]
mod tests
{
	// Access internal helpers for deterministic numerical checks; C++ test declarations.
	use super::*;

	#[test]
	fn true_scale_has_equal_pixels_per_degree()
	{
		for width in [320.0, 600.0, 1000.0]
		{
			let height = plot_height(width, 210.0, true);
			let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width - 42.0, height - 30.0));
			let origin = project(rect, 0.0, 0.0);
			let horizontal = project(rect, 1.0, 0.0).x - origin.x;
			let vertical = origin.y - project(rect, 0.0, 1.0).y;
			assert!((horizontal - vertical).abs() < 1e-5);
		}
		for width in [0.0, f32::NAN]
		{
			assert!(plot_height(width, 210.0, true).is_finite());
		}
		assert_eq!(plot_height(320.0, f32::NAN, false), 210.0);
		assert_eq!(plot_height(320.0, 150.0, false), 150.0);
	}
	// Check angular size, wrap continuity, empty and invalid inputs; C++ unit test.
	#[test]
	fn angular_disks_and_projection()
	{
		// Include a north-crossing Sun and an Earth at ordinary azimuth; C++ fixture iteration.
		for (az, el, r) in [(0.05, 1.0, 0.267), (137.0, -5.0, 0.95)]
		{
			// The boundary must retain all prescribed pieces; C++ size assertion.
			let points = disk(az, el, r);
			assert_eq!(points.len(), 64);
			// Every point must lie at the requested great-circle angular radius; C++ range-for.
			for p in points
			{
				// Compute independent spherical separation from the dot product; C++ trigonometric expression.
				let cos = el.to_radians().sin() * p[1].to_radians().sin()
					+ el.to_radians().cos()
						* p[1].to_radians().cos()
						* (p[0] - az).to_radians().cos();
				// Tolerance covers ordinary floating-point inverse cosine rounding; C++ numerical assertion.
				assert!((cos.clamp(-1.0, 1.0).acos().to_degrees() - r).abs() < 1e-8);
				// No boundary crosses the panorama by a 360-degree chord; C++ wrap assertion.
				assert!((p[0] - az).abs() < 2.0);
			}
		}
		// Invalid geometry must never produce mesh coordinates; C++ error-case assertions.
		assert!(disk(f64::NAN, 0.0, 0.267).is_empty());
		assert!(disk(0.0, 0.0, 0.0).is_empty());
		assert!(disk(0.0, 90.0, 0.95).is_empty());
		// The angular origin and corners have an exact screen mapping; C++ affine-transform test.
		let rect = Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(370.0, 220.0));
		assert_eq!(project(rect, 0.0, 10.0), rect.min);
		assert_eq!(project(rect, 360.0, -10.0), rect.max);
	}
}

fn guides(painter: &egui::Painter, sky: &egui::Painter, rect: Rect)
{
	// Give vertical angles a modest reference grid; C++ range-for over tick elevations.
	for el in [-10.0, -5.0, 0.0, 5.0, 10.0]
	{
		// Place the zero-degree tangent plane consistently with the numerical view; C++ point transform.
		let p = project(rect, 0.0, el);
		// Draw faint horizontal guides behind the bodies; C++ line primitive.
		sky.line_segment(
			[p, egui::pos2(rect.right(), p.y)],
			Stroke::new(
				if el == 0.0 { 1.0 } else { 0.5 },
				Color32::from_gray(if el == 0.0 { 85 } else { 38 }),
			),
		);
		// Label degrees outside the scene's clipping rectangle; C++ positioned text.
		if rect.height() < 40.0 && el != -10.0 && el != 10.0
		{
			continue;
		}
		painter.text(
			egui::pos2(rect.left() - 4.0, p.y),
			egui::Align2::RIGHT_CENTER,
			format!("{el:+.0}°"),
			egui::FontId::proportional(10.0),
			Color32::GRAY,
		);
	}
	// Cardinal marks describe direction around the observer; C++ counted loop.
	for (i, label) in ["N 0°", "E 90°", "S 180°", "W 270°", "N 360°"]
		.iter()
		.enumerate()
	{
		// Include both north endpoints without joining paths across the wrap; C++ affine transform.
		let p = project(rect, i as f64 * 90.0, -10.0);
		// Compass labels stay readable against the outer panel; C++ text primitive.
		painter.text(
			p + egui::vec2(0.0, 13.0),
			match i
			{
				0 => egui::Align2::LEFT_CENTER,
				4 => egui::Align2::RIGHT_CENTER,
				_ => egui::Align2::CENTER_CENTER,
			},
			*label,
			egui::FontId::proportional(13.0),
			Color32::WHITE,
		);
	}
}

fn bodies(sky: &egui::Painter, rect: Rect, data: &SiteData, time:i64, range: Range<usize>)
{
	// Draw time-window paths before the current disks and terrain; C++ pair iteration.
	for (track, color) in [
		(&data.sun, Color32::from_rgb(95, 85, 15)),
		(&data.earth, Color32::from_rgb(30, 70, 110)),
	]
	{
		// Reuse exactly the technical view's north-wrap splitting; C++ polyline iteration.
		for part in super::panorama::segments(track, range.clone())
		{
			// Convert only the selected window's coordinates; C++ transform into screen points.
			let points = part
				.iter()
				.map(|p| project(rect, p[0], p[1]))
				.collect::<Vec<_>>();
			// A single point needs no line primitive; C++ size guard.
			if points.len() > 1
			{
				sky.add(egui::Shape::line(points, Stroke::new(1.0, color)));
			}
		}
	}
	// Angular radii are scientific inputs, not arbitrary point-marker sizes; C++ tuple loop.
	for (track, radius, color, name) in [
		(&data.sun, f64::from(SUN_RADIUS_DEG), Color32::YELLOW, "Sun"),
		(&data.earth, 0.95, Color32::LIGHT_BLUE, "Earth"),
	]
	{
		// Read current direction once for all wrapped copies; C++ scalar locals.
		let Some([a,e])=crate::timeline::sample(track,time) else {continue;};
		let (a,e)=(f64::from(a),f64::from(e));
		// Preserve a partially visible disk at either edge of north; C++ three translated copies.
		for shift in [-360.0, 0.0, 360.0]
		{
			// Convert the spherical boundary rather than inflating a small Sun for visibility; C++ polygon transform.
			let points = disk(a, e, radius)
				.iter()
				.map(|p| project(rect, p[0] + shift, p[1]))
				.collect();
			// The finite disks are painted before ground, so covered parts vanish; C++ filled polygon.
			sky.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
			// A small caption identifies even the true-size subpixel-width Sun; C++ positioned label.
			if rect.height() >= 40.0
			{
				sky.text(
				project(rect, a + shift, e + radius) + egui::vec2(4.0, -2.0),
				egui::Align2::LEFT_BOTTOM,
				name,
				egui::FontId::proportional(11.0),
				color,
				);
			}
		}
	}
}

fn terrain(sky: &egui::Painter, rect: Rect, data: &SiteData)
{
	// Tessellate the nonconvex silhouette as independent adjacent trapezoids; C++ mesh value.
	let mut ground = egui::Mesh::default();
	// Preserve every supplied bearing bin and the cyclic endpoint; C++ maximum sample count.
	let bins = data
		.horizon
		.as_ref()
		.map_or(360, |h| h.h_deg.len().max(360));
	// Ground is intentionally submitted after both finite bodies; C++ counted loop.
	for i in 0..bins
	{
		// Convert neighboring bins to angular positions; C++ two double locals.
		let a = i as f64 * 360.0 / bins as f64;
		let b = (i + 1) as f64 * 360.0 / bins as f64;
		// Save the first vertex index before appending this trapezoid; C++ uint32_t offset.
		let first = ground.vertices.len() as u32;
		// The same Horizon::at interpolation drives the status and visible boundary; C++ array iteration.
		for p in [
			project(rect, a, -10.0),
			project(rect, b, -10.0),
			project(
				rect,
				b,
				f64::from(data.horizon_at(b as f32)).clamp(-10.0, 10.0),
			),
			project(
				rect,
				a,
				f64::from(data.horizon_at(a as f32)).clamp(-10.0, 10.0),
			),
		]
		{
			// Opaque grey blocks the hidden portions rather than merely drawing an outline; C++ colored vertex append.
			ground.colored_vertex(p, Color32::from_gray(92));
		}
		// Two triangles exactly cover the trapezoid; C++ indexed triangle append.
		ground.add_triangle(first, first + 1, first + 2);
		ground.add_triangle(first, first + 2, first + 3);
	}
	// Occlude sky paths and the Sun/Earth consistently with the current terrain profile; C++ mesh submission.
	sky.add(egui::Shape::mesh(ground));
}
