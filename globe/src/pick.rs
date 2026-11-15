//! Surface picking and tile outline for arbitrary points.

use super::*;

impl Globe
{
	/// Return and clear the last click on the Moon that did not hit a marker: (lat, east lon) in degrees.
	pub fn take_pick(&mut self) -> Option<(f64, f64)>
	{
		self.pick.take()
	}

	/// Highlight a tile as [center lat, center lon, half lat, half lon] in degrees, or clear it.
	pub fn set_tile(&mut self, tile: Option<[f64; 4]>)
	{
		self.tile = tile;
	}

	pub(crate) fn surface_pick(
		&mut self,
		response: &egui::Response,
		rect: egui::Rect,
		view: math::View,
		marker_hit: bool,
	)
	{
		if marker_hit || !response.clicked()
		{
			return;
		}
		let Some(p) = response.interact_pointer_pos()
		else
		{
			return;
		};
		let ndc = [
			(p.x - rect.center().x) / (rect.width() * 0.5),
			-(p.y - rect.center().y) / (rect.height() * 0.5),
		];
		if let Some(point) = view.unproject(ndc)
		{
			let lat = f64::from(point[1].clamp(-1.0, 1.0).asin().to_degrees());
			let lon = f64::from(point[0].atan2(point[2]).to_degrees()).rem_euclid(360.0);
			self.pick = Some((lat, lon));
		}
	}

	pub(crate) fn paint_tile(&self, ui: &egui::Ui, rect: egui::Rect, view: math::View)
	{
		let Some(tile) = self.tile
		else
		{
			return;
		};
		let painter = ui.painter_at(rect);
		let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 210, 80));
		let points = outline(rect, view, tile, 16);
		for pair in points.windows(2)
		{
			if let [Some(a), Some(b)] = pair
			{
				painter.line_segment([*a, *b], stroke);
			}
		}
	}

	/// Color finished tiles by their percentage of time with any Sun (0..100).
	pub fn set_tiles(&mut self, tiles: Vec<([f64; 4], f32)>)
	{
		self.tiles = tiles;
	}

	/// Snap (lat, east lon) to a tile [center lat, center lon, half lat, half lon], for the zoomed grid.
	pub fn set_grid(&mut self, grid: fn(f64, f64) -> [f64; 4])
	{
		self.grid = Some(grid);
	}

	pub(crate) fn paint_tiles(&self, ui: &mut egui::Ui, rect: egui::Rect, view: math::View)
	{
		if self.tiles.is_empty()
		{
			return;
		}
		let painter = ui.painter_at(rect);
		for &(tile, pct) in &self.tiles
		{
			let points: Option<Vec<_>> = outline(rect, view, tile, 4).into_iter().collect();
			if let Some(mut points) = points
			{
				points.pop();
				let color = sun_color(pct);
				painter.add(egui::Shape::convex_polygon(
					points,
					color.gamma_multiply(0.55),
					egui::Stroke::new(1.0, color),
				));
			}
		}
		// Legend: red (no Sun) to green (always Sun), bottom left.
		let origin = rect.left_bottom() + egui::vec2(8.0, -30.0);
		painter.rect_filled(
			egui::Rect::from_min_size(origin - egui::vec2(3.0, 3.0), egui::vec2(170.0, 24.0)),
			3.0,
			egui::Color32::from_black_alpha(200),
		);
		for i in 0..40
		{
			let x = origin.x + 34.0 + i as f32 * 2.0;
			painter.rect_filled(
				egui::Rect::from_min_size(egui::pos2(x, origin.y + 4.0), egui::vec2(2.0, 10.0)),
				0.0,
				sun_color(i as f32 * 100.0 / 39.0),
			);
		}
		let font = egui::FontId::proportional(11.0);
		painter.text(
			origin + egui::vec2(0.0, 9.0),
			egui::Align2::LEFT_CENTER,
			"Sun 0%",
			font.clone(),
			egui::Color32::LIGHT_GRAY,
		);
		painter.text(
			origin + egui::vec2(118.0, 9.0),
			egui::Align2::LEFT_CENTER,
			"100%",
			font,
			egui::Color32::LIGHT_GRAY,
		);
		let help =
			egui::Rect::from_min_size(origin + egui::vec2(146.0, -1.0), egui::vec2(20.0, 20.0));
		ui.scope_builder(egui::UiBuilder::new().max_rect(help), |ui| {
			help::question(
				ui,
				"Your points, colored by how often the Sun is visible in the chosen period.",
			);
		});
	}

	// Faint tile grid around the view center once zoomed in enough for tiles to be legible.
	pub(crate) fn paint_grid(&self, ui: &egui::Ui, rect: egui::Rect, view: math::View)
	{
		let Some(grid) = self.grid
		else
		{
			return;
		};
		if view.zoom < 3.0
		{
			return;
		}
		let painter = ui.painter_at(rect);
		let stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(28));
		let mut lat = -89.0;
		while lat < 90.0
		{
			let half_lon = grid(lat, 0.0)[3];
			let cells = (180.0 / half_lon).round() as usize;
			for c in 0..cells
			{
				let lon = (c as f64 + 0.5) * 2.0 * half_lon;
				let tile = [lat, lon, 1.0, half_lon];
				let center = to_screen(rect, view, lat, lon);
				if !center.is_some_and(|p| rect.expand(60.0).contains(p))
				{
					continue;
				}
				// South and west edges; neighbours supply the rest.
				let points = outline(rect, view, tile, 2);
				for pair in points[..3].windows(2).chain(points[6..].windows(2))
				{
					if let [Some(a), Some(b)] = pair
					{
						painter.line_segment([*a, *b], stroke);
					}
				}
			}
			lat += 2.0;
		}
	}
}

fn to_screen(rect: egui::Rect, view: math::View, lat: f64, lon: f64) -> Option<egui::Pos2>
{
	let (la, lo) = (lat.clamp(-90.0, 90.0) as f32, lon as f32);
	view.project(math::direction(la.to_radians(), lo.to_radians()), true)
		.map(|ndc| {
			rect.center() + egui::vec2(ndc[0] * rect.width() * 0.5, -ndc[1] * rect.height() * 0.5)
		})
}

// Closed boundary walk (south, east, north, west edges), n steps per edge; None where hidden.
fn outline(rect: egui::Rect, view: math::View, tile: [f64; 4], n: usize)
-> Vec<Option<egui::Pos2>>
{
	let [lat, lon, hl, ho] = tile;
	let corners = [
		(lat - hl, lon - ho),
		(lat - hl, lon + ho),
		(lat + hl, lon + ho),
		(lat + hl, lon - ho),
	];
	(0..=4 * n)
		.map(|k| {
			let (a, b) = (corners[(k / n) % 4], corners[(k / n + 1) % 4]);
			let t = (k % n) as f64 / n as f64;
			to_screen(rect, view, a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
		})
		.collect()
}

// Red at 0 %, yellow at 50 %, green at 100 %.
fn sun_color(pct: f32) -> egui::Color32
{
	let p = (pct / 100.0).clamp(0.0, 1.0);
	let r = (2.0 * (1.0 - p)).min(1.0);
	let g = (2.0 * p).min(1.0);
	egui::Color32::from_rgb((r * 230.0) as u8, (g * 200.0) as u8, 60)
}

#[cfg(test)]
mod tests
{
	use super::math::View;

	#[test]
	fn unproject_inverts_project()
	{
		let view = View {
			yaw: 0.7,
			tilt: -0.4,
			zoom: 1.6,
			aspect: 1.3,
			pixels: [800.0, 600.0],
			scale: 1.0,
		};
		let ndc = [0.1, -0.2];
		let point = view.unproject(ndc).unwrap();
		assert!((point.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-4);
		let back = view.project(point, true).unwrap();
		assert!(
			ndc.iter().zip(back).all(|(a, b)| (a - b).abs() < 1e-4),
			"{back:?}"
		);
		assert!(view.unproject([0.99, 0.99]).is_none());
	}
}
