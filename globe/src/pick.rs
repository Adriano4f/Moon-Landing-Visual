//! Surface picking and colored dots for computed points.

use super::*;

impl Globe
{
	/// Return and clear the last custom point pick: (lat, east lon) in degrees.
	pub fn take_pick(&mut self) -> Option<(f64, f64)>
	{
		self.pick.take()
	}

	/// Color computed points by their percentage of time with any Sun (0..100).
	pub fn set_points(&mut self, points: Vec<([f64; 2], f32)>)
	{
		self.points = points;
	}

	pub(crate) fn surface_pick(
		&mut self,
		response: &egui::Response,
		rect: egui::Rect,
		view: math::View,
		marker_hit: bool,
	)
	{
		if !response.clicked_by(egui::PointerButton::Primary) || response.double_clicked()
		{
			return;
		}
		let Some(p) = response.interact_pointer_pos()
		else
		{
			return;
		};
		if let Some([lat, lon]) = self.point_at(p, rect, view)
		{
			self.pick = Some((lat, lon));
			return;
		}
		if marker_hit
		{
			return;
		}
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

	fn point_at(&self, pointer: egui::Pos2, rect: egui::Rect, view: math::View)
	-> Option<[f64; 2]>
	{
		self.points
			.iter()
			.filter_map(|&(point, _)| {
				let position = to_screen(rect, view, point[0], point[1])?;
				let distance = pointer.distance_sq(position);
				(rect.contains(position) && distance <= 10.0_f32.powi(2))
					.then_some((point, distance))
			})
			.min_by(|a, b| a.1.total_cmp(&b.1))
			.map(|(point, _)| point)
	}

	pub(crate) fn paint_points(&self, ui: &mut egui::Ui, rect: egui::Rect, view: math::View)
	{
		if self.points.is_empty()
		{
			return;
		}
		let painter = ui.painter_at(rect);
		for &(point, pct) in &self.points
		{
			if let Some(position) = to_screen(rect, view, point[0], point[1])
			{
				painter.circle_filled(position, 4.0, sun_color(pct));
				painter.circle_stroke(position, 4.0, egui::Stroke::new(1.0, egui::Color32::BLACK));
			}
		}
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
				"Dots show computed points, colored by how often the Sun is visible in the chosen period.",
			);
		});
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

/// Red at 0 %, yellow at 50 %, green at 100 %.
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
	fn dot_picking_is_exact_nearest_and_screen_sized()
	{
		let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
		let point = [0.123456789, 0.987654321];
		let mut globe = crate::scene_tests::unloaded();
		for zoom in [1.0, 8.0, 64.0]
		{
			let view = View {
				pan: [0.0; 2],
				rotation: super::math::IDENTITY,
				yaw: 0.0,
				tilt: 0.0,
				zoom,
				aspect: rect.aspect_ratio(),
				pixels: [800.0, 600.0],
				scale: 1.0,
			};
			let position = super::to_screen(rect, view, point[0], point[1]).unwrap();
			assert!(globe.point_at(position, rect, view).is_none());
			globe.set_points(vec![(point, 30.0), ([0.0, 180.0], 70.0)]);
			assert_eq!(globe.point_at(position, rect, view), Some(point));
			assert_eq!(
				globe.point_at(position + egui::vec2(9.0, 0.0), rect, view),
				Some(point)
			);
			assert!(
				globe
					.point_at(position + egui::vec2(11.0, 0.0), rect, view)
					.is_none()
			);
			let nearby = [point[0], point[1] + 0.001];
			globe.set_points(vec![(point, 30.0), (nearby, 60.0)]);
			let position = super::to_screen(rect, view, nearby[0], nearby[1]).unwrap();
			assert_eq!(globe.point_at(position, rect, view), Some(nearby));
			globe.set_points(vec![([0.0, 180.0], 70.0)]);
			assert!(globe.point_at(rect.center(), rect, view).is_none());
			globe.set_points(vec![]);
		}
	}

	#[test]
	fn unproject_inverts_project()
	{
		let view = View {
			pan: [0.0; 2],
			rotation: super::math::IDENTITY,
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

	#[test]
	fn picking_matches_surface_at_deep_zoom()
	{
		for utc in [
			crate::orientation::INITIAL_UTC,
			crate::orientation::INITIAL_UTC + 86400,
			2051222400,
		]
		{
			for zoom in [0.5, 1.0, 8.0, 64.0, 1024.0]
			{
				let view = View {
					pan: [0.0; 2],
					rotation: crate::orientation::at_utc(utc),
					yaw: 0.7,
					tilt: -1.56,
					zoom,
					aspect: 0.6,
					pixels: [600.0, 1000.0],
					scale: 1.0,
				};
				let ndc = if zoom >= 8.0
				{
					[0.7, -0.8]
				}
				else
				{
					[0.03, -0.04]
				};
				let point = view.unproject(ndc).unwrap();
				assert!((point.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-5);
				let back = view.project(point, true).unwrap();
				assert!(
					ndc.iter().zip(back).all(|(a, b)| (a - b).abs() < 4e-4),
					"zoom {zoom}: {back:?}"
				);
			}
		}
	}
}
