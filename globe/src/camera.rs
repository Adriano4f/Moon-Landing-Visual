use super::*;

impl Globe
{
	// Draw camera controls independently of the illustration; C++ member function.
	pub fn camera_controls(&mut self, ui: &mut egui::Ui)
	{
		// Apply a texture-resolution limit instead of inventing polar detail; C++ bounded edits.
		ui.horizontal_wrapped(|ui| {
			if ui.button("- Zoom").clicked() { self.zoom = (self.zoom / 1.8).max(1.0); self.focus = None; }
			help::question(ui, "Zoom out to see more of the Moon. This changes the camera, not the landing site.");
			if ui.button("+ Zoom").clicked() { self.zoom = (self.zoom * 1.8).min(MAX_ZOOM); self.focus = None; }
			help::question(ui, "Zoom in, up to 8 times. Higher magnification enlarges misleading polar texture streaks. Choose close neighbors in Site; the panorama uses the detailed terrain horizon.");
		});
		self.site_camera(ui);
		// Require real Earth directions before entering Earth-following mode; C++ optional guard.
		let ready = self
			.scene
			.iter()
			.any(|s| s.id == self.selected && s.directions.is_some());
		ui.horizontal(|ui| {
			if ui.add_enabled(ready, egui::Button::new("View from Earth")).clicked()
			{ self.earth_view = true; self.paused = true; self.zoom = 1.0; self.focus = None; }
			help::question(ui, "Look along the current Earth direction. The apparent rocking changes with time. This is a direction-correct illustration, not a camera at Earth's true distance.");
		});
		// Keep synthetic camera spin optional; C++ bool edit.
		let mut spin = !self.paused;
		ui.horizontal(|ui| {
			if ui.checkbox(&mut spin, "Auto-rotate").changed() { self.paused = !spin; self.earth_view = false; self.focus = None; }
			help::question(ui, "Turn the camera slowly for inspection. This artificial one-minute spin is not a real orbital period and does not change the scientific date.");
		});
		// Identify the mode and magnification without hidden controls; C++ label.
		ui.label(format!(
			"{:.1}x | {}",
			self.zoom,
			if self.earth_view
			{
				"Earth-following view"
			}
			else
			{
				"Free orbit"
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
			help::question(ui, "Earthshine is sunlight reflected by Earth onto the Moon. It is normally thousands of times weaker than sunlight. This exposure boost changes only the illustration, not solar power or communication.");
		});
		// Keep the nonphysical aid explicitly labeled; C++ checkbox row.
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.fill, "Far side fill light");
			help::question(ui, "Soft gray-blue light helps you inspect the mean far side after dragging around the Moon. It is a visual aid, not real light, and changes no scientific result.");
		});
	}

	// Expose the independent surface field without requesting more site tracks; C++ checkbox.
	pub fn surface_ray_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.surface_rays, "Rays across the whole Moon");
			help::question(ui, "Parallel yellow rays arrive from the real Sun direction at this date and end on the sunlit sphere. About 256 of 512 evenly distributed points are lit. Lengths are illustrative; this is not terrain shadow tracing. The Moon hides rays behind its surface.");
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
			help::question(ui, "Center the selected point and zoom in. Close published points can overlap on this overview; select them individually in Site.");
			clicked
		}).inner;
		// Animation time is independent of the scientific date; C++ double local.
		let time = ui.input(|i| i.time);
		if go && let Some(site) = self.scene.iter().find(|s| s.id == self.selected)
		{
			// Site-centered viewing is a free camera, not an Earth lock; C++ camera mode switch.
			self.earth_view = false;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let target = -(site.lon_deg as f32).to_radians();
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
					(site.lat_deg as f32).to_radians(),
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
		ui.horizontal_wrapped(|ui| {
			ui.label("Texture detail");
			egui::ComboBox::from_id_salt("moon-quality").selected_text(format!("{}k", self.target_tier)).show_ui(ui, |ui| {
				for tier in [2,4,8].into_iter().filter(|t| *t <= self.max_tier) { ui.selectable_value(&mut self.target_tier, tier, format!("{tier}k")); }
			});
			help::question(ui, "The Moon always appears in 2k first. Desktop browsers normally improve to 4k; phones keep 2k. Choose 8k only if you want a larger download and have enough memory. Higher detail changes only the globe map, not scientific data. The old Moon remains visible if an upgrade fails.");
		});
		// Distinguish displayed quality from a download still in progress; C++ optional snapshot.
		let displayed = self.renderer.lock().as_ref().map(|r| r.tier);
		help::annotated(
			ui,
			format!(
				"Showing {} | requested {}k",
				displayed.map_or("loading".into(), |t| format!("{t}k")),
				self.target_tier
			),
			"The first number is the currently visible map. The requested detail replaces it only after both images have decoded and uploaded successfully.",
		);
	}
}
