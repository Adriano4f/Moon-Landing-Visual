use super::*;

impl Globe
{
	pub fn view_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			if ui.selectable_label(self.earth_view,"From Earth").clicked() {
				self.earth_view=true; self.paused=true; self.zoom=1.0; self.focus=None;
			}
			if ui.selectable_label(!self.earth_view,"Free").clicked() {
				self.earth_view=false; self.paused=true; self.focus=None;
			}
			help::question(ui,"Follow Earth, drag to turn freely, or show the orbit from space.");
		});
		ui.toggle_value(&mut self.space_view,"From space");
	}

	// Draw camera controls independently of the illustration; C++ member function.
	pub fn camera_controls(&mut self, ui: &mut egui::Ui)
	{
		// Apply a texture-resolution limit instead of inventing polar detail; C++ bounded edits.
		ui.horizontal(|ui| {
			if ui.button("- Zoom").clicked() { self.zoom = (self.zoom / 1.8).max(MIN_ZOOM); self.focus = None; }
			if ui.button("+ Zoom").clicked() { self.zoom = (self.zoom * 1.8).min(MAX_ZOOM); self.focus = None; }
			help::question(ui, "Enlarge the Moon up to eight times to see south-polar detail.");
		});
		self.site_camera(ui);
		// Keep synthetic camera spin optional; C++ bool edit.
		let mut spin = !self.paused;
		ui.horizontal(|ui| {
			if ui.checkbox(&mut spin, "Auto-rotate").changed() { self.paused = !spin; self.earth_view = false; self.focus = None; }
			help::question(ui, "Slowly spin the camera around the Moon without changing the date.");
		});
		// Identify the mode and magnification without hidden controls; C++ label.
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
	}

	// Draw physical-light display aids separately from camera actions; C++ member function.
	pub fn lighting_controls(&mut self, ui: &mut egui::Ui)
	{
		// Explain that increased exposure is not increased power; C++ checkbox row.
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.boosted, "Earthshine boosted (1000x)");
			help::question(ui, "Make sunlight reflected by Earth 1,000 times brighter in the picture only.");
		});
		// Keep the nonphysical aid explicitly labeled; C++ checkbox row.
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.fill, "Far side fill light");
			help::question(ui, "Add soft artificial light so the dark side stays visible.");
		});
	}

	// Expose the independent surface field without requesting more site tracks; C++ checkbox.
	pub fn surface_ray_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.surface_rays, "Sun rays");
			help::question(ui, "Draw rays showing the direction sunlight hits the lit half of the Moon.");
		});
	}
}

impl Globe
{
	fn site_camera(&mut self, ui: &mut egui::Ui)
	{
		// Request an eased transition at the real UI clock time; C++ event flag.
		let go = ui.horizontal(|ui| {
			let clicked = ui.button("Go to site").clicked();
			help::question(ui, "Center the camera on the selected site and zoom in.");
			clicked
		}).inner;
		// Animation time is independent of the scientific date; C++ double local.
		let time = ui.input(|i| i.time);
		if go && let Some(site) = self.scene.iter().find(|s| s.id == self.selected)
		{
			// Site-centered viewing is a free camera, not an Earth lock; C++ camera mode switch.
			self.earth_view = false;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let point = math::direction((site.lat_deg as f32).to_radians(),(site.lon_deg as f32).to_radians());
			let [target, latitude] = math::camera_angles(math::transform(self.body_rotation,point));
			// Bind this intermediate value for the current operation; C++ local initialization.
			let delta = (target - self.yaw + std::f32::consts::PI)
				.rem_euclid(std::f32::consts::TAU)
				- std::f32::consts::PI;
			// Update this persistent member explicitly; C++ member assignment.
			self.focus = Some((
				time,
				[self.yaw, self.tilt, self.zoom],
				[
					self.yaw + delta,
					latitude,
					MAX_ZOOM,
				],
			));
			// Update this persistent member explicitly; C++ member assignment.
			self.paused = true;
		}
	}

	fn texture_controls(&mut self, ui: &mut egui::Ui)
	{
		// Let desktop and phone users explicitly trade download cost for map detail; C++ combo-box row.
		ui.horizontal(|ui| {
			ui.label("Texture detail");
			egui::ComboBox::from_id_salt("moon-quality").width(62.0).selected_text(format!("{}k", self.target_tier)).show_ui(ui, |ui| {
				for tier in [2,4,8].into_iter().filter(|t| *t <= self.max_tier) { ui.selectable_value(&mut self.target_tier, tier, format!("{tier}k")); }
			});
			help::question(ui, "Sharper Moon images that take longer to download and use more memory.");
		});
		// Distinguish displayed quality from a download still in progress; C++ optional snapshot.
		let displayed = self.renderer.lock().as_ref().map(|r| r.tier);
		ui.horizontal(|ui| {
			ui.small(format!("Showing {} | selected {}k",displayed.map_or("loading".into(), assets::label),self.target_tier));
			help::question(ui,"Showing is the detail on screen now; selected replaces it once downloaded.");
		});
	}
}
