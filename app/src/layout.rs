use super::*;
use eframe::egui;

impl MoonApp
{
	pub(super) fn update_sites(&mut self, ui: &mut egui::Ui)
	{
		// Let the real preview reach the screen before competing for download bandwidth; C++ readiness gate.
		if !self.globe.has_visible_moon() && !self.globe.has_load_error()
		{
			return;
		}
		// Load only the selected site, or the remaining sites when all-rays is explicitly requested; C++ lazy cache initialization.
		for i in 0..self.catalog.len()
		{
			if (i == self.selected || self.all_rays) && self.panoramas[i].is_none()
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
				));
			}
			// Check this state or input before acting; C++ conditional branch.
			if let Some(p) = &mut self.panoramas[i]
			{
				p.update();
			}
		}
	}

	pub(super) fn synchronize_scene(&mut self)
	{
		// Keep one UTC epoch across switches and all-site rays; C++ optional epoch restoration.
		if let (Some(t), Some(p)) = (self.time, &mut self.panoramas[self.selected])
		{
			p.set_time(t);
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let margin = self.panoramas[self.selected]
			.as_ref()
			.map_or(0.5, |p| p.margin());
		// Bind this intermediate value for the current operation; C++ local initialization.
		let epoch = self.time.or_else(|| {
			self.panoramas[self.selected]
				.as_ref()
				.and_then(|p| p.time())
		});
		// Bind this intermediate value for the current operation; C++ local initialization.
		let scene = self
			.catalog
			.iter()
			.enumerate()
			.map(|(i, s)| {
				// Bind this intermediate value for the current operation; C++ local initialization.
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
		self.globe.set_scene(scene, self.selected);
	}

	pub(super) fn central_view(&mut self, ui: &mut egui::Ui)
	{
		// Preserve black behind the Moon in both layouts; C++ immediate-mode central container.
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK))
			.show(ui, |ui| {
				// Capture finite viewport dimensions before entering the unbounded scroll axis; C++ copy a size vector.
				let available = ui.available_size();
				// Scroll only if a short screen cannot fit both views; C++ immediate-mode scroll container.
				eframe::egui::ScrollArea::vertical().show(ui, |ui| {
					self.main_views(ui, available);
				});
			});
	}
}

impl MoonApp
{
	fn main_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		// Stack on phones rather than crushing a 360-degree plot; C++ viewport-width branch.
		if available.x < 700.0
		{
			self.phone_views(ui, available);
		}
		// Use equal-width columns on larger windows; C++ else with two child containers.
		else
		{
			self.desktop_views(ui, available);
		}
	}
}

impl MoonApp
{
	fn phone_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		// Reserve room for phone camera and lighting controls; C++ clamp(height*0.32,220,290).
		let size = eframe::egui::vec2(available.x, (available.y * 0.32).clamp(220.0, 290.0));
		// Constrain the globe callback to the top rectangle; C++ child container with explicit size.
		ui.allocate_ui(size, |ui| {
			// Check this state or input before acting; C++ conditional branch.
			if let Some(id) = self.globe.show(ui)
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.selected = id;
			}
		});
		// Draw the scientific view below it; C++ member draw call.
		ui.allocate_ui(
			eframe::egui::vec2(available.x, (available.y - size.y - 8.0).max(460.0)),
			|ui| self.show_panorama(ui),
		);
	}

	fn desktop_views(&mut self, ui: &mut egui::Ui, available: egui::Vec2)
	{
		// Rebuild columns without retaining widget handles; C++ immediate-mode callback.
		ui.allocate_ui(available, |ui| {
			// Divide only the bounded native/desktop viewport, not the unbounded scroll axis; C++ column container.
			ui.columns(2, |columns| {
				// Keep the rotating Moon on black at left; C++ first child draw.
				if let Some(id) = self.globe.show(&mut columns[0])
				{
					// Update this persistent member explicitly; C++ member assignment.
					self.selected = id;
				}
				// Disable columns' cross-axis justification so the plot can reserve tick-label space; C++ child layout override.
				columns[1].with_layout(
					eframe::egui::Layout::top_down(eframe::egui::Align::Min),
					|ui| self.show_panorama(ui),
				);
			})
		});
	}
}
