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
		let Some([lat, lon, hl, ho]) = self.tile
		else
		{
			return;
		};
		// Closed boundary walk: bottom, right, top, left edges, 16 steps each.
		let n = 16;
		let corners = [
			(lat - hl, lon - ho),
			(lat - hl, lon + ho),
			(lat + hl, lon + ho),
			(lat + hl, lon - ho),
		];
		let painter = ui.painter_at(rect);
		let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 210, 80));
		let mut last: Option<egui::Pos2> = None;
		for k in 0..=4 * n
		{
			let (a, b) = (corners[(k / n) % 4], corners[(k / n + 1) % 4]);
			let t = (k % n) as f64 / n as f64;
			let la = (a.0 + (b.0 - a.0) * t).clamp(-90.0, 90.0) as f32;
			let lo = (a.1 + (b.1 - a.1) * t) as f32;
			let position = view
				.project(math::direction(la.to_radians(), lo.to_radians()), true)
				.map(|ndc| {
					rect.center()
						+ egui::vec2(ndc[0] * rect.width() * 0.5, -ndc[1] * rect.height() * 0.5)
				});
			if let (Some(a), Some(b)) = (last, position)
			{
				painter.line_segment([a, b], stroke);
			}
			last = position;
		}
	}
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
