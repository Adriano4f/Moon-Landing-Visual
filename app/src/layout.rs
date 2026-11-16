use super::*;
use eframe::egui;

impl MoonApp
{
	pub(super) fn update_sites(&mut self, ui: &mut egui::Ui)
	{
		// Let the preview reach the screen before competing for download bandwidth.
		if !self.globe.has_visible_moon() && !self.globe.has_load_error()
		{
			return;
		}
		// Load only the selected site, or the remaining sites when all-rays is explicitly requested.
		for i in 0..self.catalog.len()
		{
			if (i == self.selected || self.all_rays)
				&& i < self.published
				&& self.panoramas[i].is_none()
			{
				self.panoramas[i] = Some(panorama::Panorama::new(
					ui.ctx().clone(),
					if i == 0
					{
						data::FILE.into()
					}
					else
					{
						filename(&self.catalog[i].name)
					},
					std::sync::Arc::clone(&self.timeline),
				));
			}
			if let Some(p) = &mut self.panoramas[i]
			{
				p.update(ui.ctx());
			}
		}
	}

	pub(super) fn synchronize_scene(&mut self)
	{
		// Keep one UTC epoch across switches and all-site rays.
		if let (Some(t), Some(p)) = (self.time, &mut self.panoramas[self.selected])
		{
			p.set_time(t);
		}
		let margin = self.panoramas[self.selected]
			.as_ref()
			.map_or(0.5, |p| p.margin());
		let epoch = self.time.or_else(|| {
			self.panoramas[self.selected]
				.as_ref()
				.and_then(|p| p.time())
		});
		let scene = self
			.catalog
			.iter()
			.enumerate()
			.map(|(i, s)| {
				let directions = if i == self.selected || self.all_rays
				{
					self.panoramas[i]
						.as_mut()
						.and_then(|p| epoch.and_then(|t| p.directions_at(t, margin)))
				}
				else
				{
					None
				};
				globe::Marker {
					id: i,
					name: s.name.clone(),
					lat_deg: s.lat_deg,
					lon_deg: s.lon_deg,
					directions,
				}
			})
			.collect();
		if let Some(epoch)=epoch { self.globe.set_time(epoch); }
		self.globe.set_scene(scene, self.selected);
	}

	pub(super) fn central_view(&mut self, ui: &mut egui::Ui)
	{
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK).inner_margin(8.0))
			.show(ui, |ui| {
				let available = ui.available_size();
				if available.x < 700.0
				{
					egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
						ui.set_width((ui.available_width() - 16.0).max(1.0));
						self.phone_views(ui, available);
					});
				}
				else
				{
					self.desktop_views(ui, available);
				}
			});
	}
}

impl MoonApp
{
	fn phone_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		let size = eframe::egui::vec2(ui.available_width(), (available.y * 0.4).clamp(300.0, 360.0));
		ui.allocate_ui(size, |ui| {
			if let Some(id) = self.globe.show(ui)
			{
				self.select_marker(ui.ctx(), id);
			}
		});
		self.show_panorama(ui);
	}

	fn desktop_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		let bounds = egui::Rect::from_min_size(ui.cursor().min, available);
		ui.allocate_rect(bounds, egui::Sense::hover());
		let id = ui.scope_id().with("information-width");
		let mut ratio = ui.ctx().data(|d| d.get_temp::<f32>(id)).unwrap_or(0.5);
		let (_, handle, _) = split_rects(bounds, ratio);
		let response = ui.interact(handle, id, egui::Sense::drag())
			.on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
			.on_hover_text("Drag to resize the globe and information panel.");
		if response.dragged() && let Some(pointer) = response.interact_pointer_pos()
		{
			ratio = ((pointer.x - bounds.left()) / bounds.width()).clamp(0.3, 0.7);
		}
		ui.ctx().data_mut(|d| d.insert_temp(id, ratio));
		let (left, handle, right) = split_rects(bounds, ratio);
		let thickness = if response.hovered() || response.dragged()
		{
			3.0
		}
		else
		{
			1.0
		};
		ui.painter().vline(handle.center().x, handle.y_range(),
			egui::Stroke::new(thickness, ui.visuals().weak_text_color()));
		let mut globe_ui = ui.new_child(egui::UiBuilder::new().id_salt("globe-panel")
			.max_rect(left).layout(egui::Layout::top_down(egui::Align::Min)));
		globe_ui.set_clip_rect(left);
		if let Some(selected) = self.globe.show(&mut globe_ui)
		{
			self.select_marker(ui.ctx(), selected);
		}
		let right = right.shrink2(egui::vec2(12.0, 0.0));
		let mut info_ui = ui.new_child(egui::UiBuilder::new().id_salt("information-panel")
			.max_rect(right).layout(egui::Layout::top_down(egui::Align::Min)));
		info_ui.set_clip_rect(right);
		self.show_panorama(&mut info_ui);
	}
}

fn split_rects(bounds: egui::Rect, ratio: f32) -> (egui::Rect, egui::Rect, egui::Rect)
{
	let ratio = if ratio.is_finite()
	{
		ratio.clamp(0.3, 0.7)
	}
	else
	{
		0.5
	};
	let x = bounds.left() + bounds.width() * ratio;
	(
		egui::Rect::from_min_max(bounds.min, egui::pos2(x - 5.0, bounds.bottom())),
		egui::Rect::from_min_max(egui::pos2(x - 5.0, bounds.top()), egui::pos2(x + 5.0, bounds.bottom())),
		egui::Rect::from_min_max(egui::pos2(x + 5.0, bounds.top()), bounds.max),
	)
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn split_is_bounded_for_common_viewports_and_invalid_ratios()
	{
		for size in [egui::vec2(1280.0, 720.0), egui::vec2(1440.0, 1000.0)]
		{
			let bounds = egui::Rect::from_min_max(egui::pos2(8.0, 60.0), size.to_pos2() - egui::vec2(8.0, 8.0));
			for ratio in [-1.0, 0.3, 0.5, 0.7, 2.0, f32::NAN]
			{
				let (left, handle, right) = split_rects(bounds, ratio);
				assert!(bounds.contains_rect(left) && bounds.contains_rect(right));
				assert!(left.width() >= bounds.width() * 0.3 - 5.1);
				assert!(right.width() >= bounds.width() * 0.3 - 5.1);
				assert_eq!(handle.width(), 10.0);
			}
		}
	}
}
