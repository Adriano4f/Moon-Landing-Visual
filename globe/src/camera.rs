use super::*;

impl Globe
{
	pub fn view_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			if ui.add(egui::Button::selectable(self.earth_view,"From Earth").frame_when_inactive(true)).clicked()
			{
				self.earth_view = true;
				self.paused = true;
				self.zoom = 1.0;
				self.focus = None;
				self.pan = [0.0; 2];
			}
			if ui.add(egui::Button::selectable(!self.earth_view,"Free").frame_when_inactive(true)).clicked() {
				self.earth_view=false; self.paused=true; self.focus=None;
			}
		});
		if ui.add(egui::Button::selectable(self.space_view,"From space").frame_when_inactive(true)).clicked()
		{
			self.space_view = !self.space_view;
		}
		help::question(ui,"Follow Earth, drag to turn freely, or show the orbit from space.");
	}

	pub fn camera_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			if ui.button("- Zoom").clicked()
			{
				self.zoom = (self.zoom / 1.8).max(MIN_ZOOM);
				self.focus = None;
			}
			if ui.button("+ Zoom").clicked()
			{
				self.zoom = (self.zoom * 1.8).min(MAX_ZOOM);
				self.focus = None;
			}
			help::question(
				ui,
				"Scroll or pinch to approach the surface with satellite imagery, up to 1,024x.",
			);
		});
		self.site_camera(ui);
		let mut spin = !self.paused;
		ui.horizontal(|ui| {
			if ui.checkbox(&mut spin, "Auto-rotate").changed()
			{
				self.paused = !spin;
				self.earth_view = false;
				self.focus = None;
			}
			help::question(
				ui,
				"Slowly spin the camera around the Moon without changing the date.",
			);
		});
		ui.label(format!(
			"{:.1}x | {}",
			self.zoom,
			if self.earth_view
			{
				"From Earth"
			}
			else
			{
				"Free"
			}
		));
		self.texture_controls(ui);
		if self.zoom > 1.0
		{
			ui.small(format!(
				"Camera altitude: {:.1} km (illustrative sphere)",
				(4.0 / self.zoom) * 1737.4
			));
		}
	}

	pub fn lighting_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.selectable_value(&mut self.live_shadows, true, "Live shadows");
			ui.selectable_value(&mut self.live_shadows, false, "Satellite photos");
			help::question(ui, "Live shadows follow the Sun; satellite photos retain recorded shadows.");
		});
		// Explain that increased exposure is not increased power.
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.boosted, "Earthshine boosted (1000x)");
			help::question(
				ui,
				"Make sunlight reflected by Earth 1,000 times brighter in the picture only.",
			);
		});
	}

	/// Expose the independent surface field without requesting more site tracks.
	pub fn surface_ray_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.surface_rays, "Sun rays");
			help::question(
				ui,
				"Show rays of sunlight falling on the lit half of the Moon.",
			);
		});
	}
}

impl Globe
{
	fn site_camera(&mut self, ui: &mut egui::Ui)
	{
		let go = ui
			.horizontal(|ui| {
				let clicked = ui.button("Go to site").clicked();
				help::question(ui, "Center the camera on the selected site and zoom in.");
				clicked
			})
			.inner;
		let time = ui.input(|i| i.time);
		if go && let Some(site) = self.scene.iter().find(|s| s.id == self.selected)
		{
			self.pan = [0.0; 2];
			// Site-centered viewing is a free camera, not an Earth lock.
			self.earth_view = false;
			let point = math::direction(
				(site.lat_deg as f32).to_radians(),
				(site.lon_deg as f32).to_radians(),
			);
			let [target, latitude] =
				math::camera_angles(math::transform(self.body_rotation, point));
			let delta = (target - self.yaw + std::f32::consts::PI)
				.rem_euclid(std::f32::consts::TAU)
				- std::f32::consts::PI;
			self.focus = Some((
				time,
				[self.yaw, self.tilt, self.zoom],
				[self.yaw + delta, latitude, 8.0],
			));
			self.paused = true;
		}
	}

	fn texture_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			ui.label("Texture detail");
			egui::ComboBox::from_id_salt("moon-quality")
				.width(62.0)
				.selected_text(format!("{}k", self.target_tier))
				.show_ui(ui, |ui| {
					for tier in [2, 4, 8].into_iter().filter(|t| *t <= self.max_tier)
					{
						ui.selectable_value(&mut self.target_tier, tier, format!("{tier}k"));
					}
				});
			help::question(
				ui,
				"Higher detail makes the Moon sharper but downloads more and uses more memory.",
			);
		});
		let displayed = self.renderer.lock().as_ref().map(|r| r.tier);
		ui.horizontal(|ui| {
			ui.small(format!(
				"Showing {} | selected {}k",
				displayed.map_or("loading".into(), assets::label),
				self.target_tier
			));
			help::question(
				ui,
				"Showing is the detail on screen now; the selected detail replaces it once downloaded.",
			);
		});
	}
}
