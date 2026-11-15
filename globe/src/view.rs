use super::*;

impl Globe
{
	/// Fill the available UI rectangle with interactive controls and optional one-minute rotation.
	// Queue a paint callback for this frame and request the next; C++ an immediate-mode UI calls this each frame.
	pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize>
	{
		self.loading_status(ui);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let time = ui.input(|i| i.time);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dt = (time - self.last_time.replace(time).unwrap_or(time)).clamp(0.0, 0.1) as f32;
		// Bind this intermediate value for the current operation; C++ local initialization.
		let _ = time;
		let lighting = self.scene_lighting(ui);
		self.animate_camera(time, dt);
		self.pending_label(ui);
		// Reserve the available paint rectangle; C++ would receive a rectangle/response pair.
		let (rect, response) = Self::paint_rectangle(ui);
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if !rect.is_positive()
		{
			// Stop this frame's work without a value; C++ return exits a void function.
			return None;
		}
		self.orbit_input(ui, &response);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let scale = ui.ctx().pixels_per_point();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let view = math::View {
			yaw: self.yaw,
			tilt: self.tilt,
			zoom: self.zoom,
			aspect: rect.aspect_ratio(),
			pixels: [rect.width() * scale, rect.height() * scale],
			scale,
		};
		self.queue_gl(ui, rect, view, lighting);
		let clicked = self.paint_markers(ui, &response, rect, view);
		Self::legend(ui, rect);
		// Request another frame so loading and animation progress; C++ asks the event loop to redraw.
		ui.ctx().request_repaint();
		clicked
	}
}

impl Globe
{
	// Let short layouts grow rather than covering the Moon with the legend; C++ bounded child size.
	fn paint_rectangle(ui: &mut egui::Ui) -> (egui::Rect, egui::Response)
	{
		let size = egui::vec2(ui.available_width(), ui.available_height().max(200.0));
		ui.allocate_exact_size(size, egui::Sense::click_and_drag())
	}

	fn loading_status(&mut self, ui: &mut egui::Ui)
	{
		// Keep direct manipulation discoverable when the settings group is closed; C++ explanatory heading row.
		ui.horizontal(|ui| {
			ui.small("Moon overview");
			help::question(ui, "Drag to turn the Moon; pinch or scroll to zoom. Click a marker to select it; the globe is an illustration.");
		});
		// Display the initial pair before starting an optional upgrade; keep the current Moon while waiting.
		if self
			.renderer
			.lock()
			.as_ref()
			.is_some_and(|r| r.visible_announced)
			&& self.tier != self.target_tier
		{
			// Start one pair per explicit target change, with UI wakeups on completion; C++ state transition.
			self.tier = self.target_tier;
			self.loading = assets::load(self.tier, Some(ui.ctx().clone()));
		}
		// A failed upgrade leaves the last successful Moon on screen; C++ optional diagnostic display.
		if let Some(error) = self.loading.lock().error.as_ref()
		{
			help::annotated(
				ui,
				format!("Texture detail unavailable: {error}"),
				"A failed detail upgrade keeps the current Moon visible. Try another detail setting; sky results do not depend on these pictures.",
			);
		}
	}

	pub(super) fn light_sources(&mut self) -> Option<[[f32; 3]; 2]>
	{
		// Copy the selected site's actual directions, never infer illumination from availability flags; C++ optional transform.
		let selected = self
			.scene
			.iter()
			.find(|s| s.id == self.selected)
			.and_then(|s| {
				// Missing or invalid tracks have no real illumination fallback; C++ optional propagation.
				let [sun, earth] = s.directions?;
				// Propagate either invalid sky direction without inventing a light; C++ optional-return aggregate.
				Some([
					moon_fixed(s.lat_deg, s.lon_deg, sun)?,
					moon_fixed(s.lat_deg, s.lon_deg, earth)?,
				])
			});
		// The initial epoch is never reused for a later missing track or changed scientific time; C++ one-shot fallback.
		if selected.is_some()
		{
			self.startup_light = None;
		}
		selected.or(self.startup_light)
	}

	fn scene_lighting(&mut self, ui: &mut egui::Ui) -> Option<Lighting>
	{
		let directions = self.light_sources();
		// Holding this mode reveals real libration rather than artificial animation; C++ conditional camera update.
		if self.earth_view
			&& let Some([_, earth]) = directions
		{
			// Cancel longitude so the Earth direction projects onto the camera's central meridian; C++ atan2.
			self.yaw = -earth[0].atan2(earth[2]);
			// Match camera latitude to sub-Earth latitude without a fixed presentation tilt; C++ asin.
			self.tilt = earth[1].clamp(-1.0, 1.0).asin();
		}
		// Pass an explicit absent light state while the selected track is unavailable; C++ optional snapshot.
		let lighting = directions.map(|[sun, earth]| Lighting {
			sun,
			earth,
			ratio: math::earthshine_ratio(sun, earth),
			boosted: self.boosted,
			fill: self.fill,
		});
		if self.earth_view
			&& let Some(light) = lighting
		{
			self.phase_status(ui, light);
		}
		// Explain unavailable data rather than silently restoring a fixed presentation Sun; C++ conditional label.
		if lighting.is_none()
		{
			// State why the Moon has no physical illumination instead of fabricating data; C++ immediate-mode text.
			ui.label("Real lighting unavailable: selected track not loaded");
		}
		lighting
	}

	fn animate_camera(&mut self, time: f64, dt: f32)
	{
		// Check this state or input before acting; C++ conditional branch.
		if let Some((start, from, to)) = self.focus
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let t = ((time - start) / 0.8).clamp(0.0, 1.0) as f32;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let blend = t * t * (3.0 - 2.0 * t);
			// Update this persistent member explicitly; C++ member assignment.
			self.yaw = from[0] + (to[0] - from[0]) * blend;
			// Update this persistent member explicitly; C++ member assignment.
			self.tilt = from[1] + (to[1] - from[1]) * blend;
			// Update this persistent member explicitly; C++ member assignment.
			self.zoom = (from[2].ln() + (to[2].ln() - from[2].ln()) * blend).exp();
			// Check this state or input before acting; C++ conditional branch.
			if t >= 1.0
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.focus = None;
			}
		}
		else if !self.paused
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.yaw += dt * std::f32::consts::TAU / 60.0;
		}
	}

	fn pending_label(&self, ui: &mut egui::Ui)
	{
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if self.renderer.lock().is_none()
		{
			// Choose an error message or the current loading label; C++ uses an owning string.
			let label = self
				.loading
				// Acquire exclusive access until the temporary guard is dropped; C++ lock_guard plus protected access.
				.lock()
				.error
				// Copy the optional diagnostic or increment a shared ownership count, not GPU resources; C++ value copy/shared_ptr copy.
				.clone()
				// Compute the fallback only when absent; C++ checks optional before calling a fallback lambda.
				.unwrap_or_else(|| format!("Loading Moon {}...", assets::label(self.tier)));
			// Paint the pending/error message on the black panel; C++ immediate-mode UIs similarly submit a label each frame.
			ui.label(label);
		}
	}

	fn orbit_input(&mut self, ui: &mut egui::Ui, response: &egui::Response)
	{
		// Wheel and touch pinch operate only inside the globe, not the panorama scroll area; C++ scoped event handling.
		if response.hovered()
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let (wheel, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
			// Check this state or input before acting; C++ conditional branch.
			if wheel != 0.0 || pinch != 1.0
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.zoom = (self.zoom * (wheel * 0.003).exp() * pinch).clamp(MIN_ZOOM, MAX_ZOOM);
				// Update this persistent member explicitly; C++ member assignment.
				self.focus = None;
				// Consume globe wheel input before the surrounding page scroll area; C++ event consumption.
				ui.input_mut(|i| i.smooth_scroll_delta.y = 0.0);
			}
		}
		// Check this state or input before acting; C++ conditional branch.
		if response.dragged()
		{
			// Direct orbit manipulation exits Earth lock while preserving the current orientation; C++ mode switch.
			self.earth_view = false;
			// Bind this intermediate value for the current operation; C++ local initialization.
			let delta = ui.input(|i| i.pointer.delta());
			// Update this persistent member explicitly; C++ member assignment.
			self.yaw += delta.x * 0.006 / self.zoom;
			// Update this persistent member explicitly; C++ member assignment.
			self.tilt = (self.tilt + delta.y * 0.006 / self.zoom)
				.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
			// Update this persistent member explicitly; C++ member assignment.
			self.paused = true;
			// Update this persistent member explicitly; C++ member assignment.
			self.focus = None;
		}
	}

	fn queue_gl(
		&self,
		ui: &mut egui::Ui,
		rect: egui::Rect,
		view: math::View,
		lighting: Option<Lighting>,
	)
	{
		// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
		let renderer = Arc::clone(&self.renderer);
		// Keep the loader alive until this callback finishes; C++ copies its shared_ptr.
		let loading = Arc::clone(&self.loading);
		let rays = self.scene_rays();
		// Capture shared state for egui's later current-context draw; C++ stores a value-capturing std::function.
		let surface_rays = self.surface_rays;
		let ctx = ui.ctx().clone();
		let callback = egui_glow::CallbackFn::new(move |info, painter| {
			// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
			let mut renderer = renderer.lock();
			Self::process_textures(&mut renderer, &loading, &ctx, painter.gl());
			// Convert egui's logical rectangle to actual pixel dimensions; C++ receives integer width/height.
			let viewport = info.viewport_in_pixels();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if viewport.width_px > 0 && viewport.height_px > 0
			{
				// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
				if let Some(renderer) = renderer.as_mut()
				{
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.paint(painter.gl(), view, &rays, lighting, surface_rays);
					Self::announce(renderer, lighting, &ctx);
				}
			}
		});
		// Append the custom GL callback to egui's paint list; C++ queues an owned rendering command.
		ui.painter().add(egui::PaintCallback {
			rect,
			callback: Arc::new(callback),
		});
	}

	fn paint_markers(
		&self,
		ui: &mut egui::Ui,
		response: &egui::Response,
		rect: egui::Rect,
		view: math::View,
	) -> Option<usize>
	{
		// Retain the nearest hit in the same marker order; C++ optional identifier.
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut clicked = None;
		// Selected markers are larger; every marker has a contrasting outline and nearest-hit selection; C++ projected hit testing.
		let painter = ui.painter_at(rect);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let pointer = response.interact_pointer_pos();
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut nearest: f32 = f32::INFINITY;
		// Visit each value without transferring the owner's resources; C++ range-for loop.
		for site in self
			.scene
			.iter()
			.filter(|s| s.id != self.selected)
			.chain(self.scene.iter().filter(|s| s.id == self.selected))
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let point = math::direction(
				(site.lat_deg as f32).to_radians(),
				(site.lon_deg as f32).to_radians(),
			);
			// Check this state or input before acting; C++ conditional branch.
			if let Some(ndc) = view.project(point, true)
			{
				// Bind this intermediate value for the current operation; C++ local initialization.
				let position = rect.center()
					+ egui::vec2(ndc[0] * rect.width() * 0.5, -ndc[1] * rect.height() * 0.5);
				if !rect.expand(15.0).contains(position)
				{
					continue;
				}
				// Bind this intermediate value for the current operation; C++ local initialization.
				let selected = site.id == self.selected;
				Self::draw_marker(&painter, position, site, selected);
				// Check this state or input before acting; C++ conditional branch.
				if response.clicked()
					&& let Some(p) = pointer
				{
					// Bind this intermediate value for the current operation; C++ local initialization.
					let distance = p.distance_sq(position);
					// Check this state or input before acting; C++ conditional branch.
					if distance < 14.0_f32.powi(2) && distance < nearest
					{
						nearest = distance;
						clicked = Some(site.id);
					}
				}
			}
		}
		clicked
	}

	fn legend(ui: &mut egui::Ui, rect: egui::Rect)
	{
		let painter = ui.painter_at(rect);
		// Keep the legend readable when the optional field crosses it; C++ translucent overlay rectangle.
		painter.rect_filled(
			egui::Rect::from_min_size(
				rect.left_top() + egui::vec2(5.0, 5.0),
				egui::vec2(258.0, 36.0),
			),
			3.0,
			egui::Color32::from_black_alpha(225),
		);
		painter.text(
			rect.left_top() + egui::vec2(8.0, 8.0),
			egui::Align2::LEFT_TOP,
			"Drag to orbit; scroll or pinch to zoom\nIllustration only; polar cap schematic",
			egui::FontId::proportional(11.0),
			egui::Color32::LIGHT_GRAY,
		);
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn short_phone_layout_grows_past_phase_controls()
	{
		for height in [80.0, 250.0, 800.0]
		{
			let ctx = egui::Context::default();
			let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
				egui::CentralPanel::default().show(root, |ui| {
					let mut bottom = 0.0;
					ui.allocate_ui(egui::vec2(412.0, height), |ui| {
						for _ in 0..3
						{
							ui.add_sized(
								egui::vec2(200.0, 44.0),
								egui::Label::new("Phase control"),
							);
						}
						let (rect, _) = Globe::paint_rectangle(ui);
						assert!(rect.height() >= 200.0);
						bottom = rect.bottom();
					});
					assert!(ui.cursor().top() >= bottom);
				});
			});
			output.textures_delta.clear();
		}
	}
}

impl Globe
{
	fn scene_rays(&self) -> Vec<([f32; 3], [Ray; 2])>
	{
		// Snapshot the selected sample for the deferred GL callback; C++ copy small optional values into a lambda.
		let rays: Vec<_> = self
			.scene
			.iter()
			.filter_map(|site| {
				// Bind this intermediate value for the current operation; C++ local initialization.
				let origin = math::direction(
					(site.lat_deg as f32).to_radians(),
					(site.lon_deg as f32).to_radians(),
				);
				// Bind this intermediate value for the current operation; C++ local initialization.
				let bodies = site.directions?;
				// Bind this intermediate value for the current operation; C++ local initialization.
				let convert = |body: BodyDirection| {
					math::local_direction(
						(site.lat_deg as f32).to_radians(),
						(site.lon_deg as f32).to_radians(),
						body.az_deg.to_radians(),
						body.el_deg.to_radians(),
					)
					.map(|direction| Ray {
						direction,
						visible: body.visible,
					})
				};
				Some((origin, [convert(bodies[0])?, convert(bodies[1])?]))
			})
			.collect();
		rays
	}

	fn process_textures(
		renderer: &mut Option<Renderer>,
		loading: &Mutex<assets::Loading>,
		ctx: &egui::Context,
		gl: &glow::Context,
	)
	{
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if loading.lock().error.is_none()
		{
			// Consume the completed pair without rebuilding an existing mesh or shader; C++ optional result.
			let result = loading.lock().take();
			match result
			{
				Ok(Some(images)) =>
				{
					if let Some(current) = renderer.as_mut()
					{
						// Keep old textures until every replacement is uploaded; C++ staged owner transfer.
						current.begin_upgrade(gl, images);
					}
					else
					{
						match Renderer::new(gl, images)
						{
							Ok(ready) => *renderer = Some(ready),
							Err(error) => loading.lock().error = Some(error),
						}
					}
				}
				Ok(None) =>
				{}
				Err(error) => loading.lock().error = Some(error),
			}
		}
		// Upload at most one replacement image per frame, leaving the last Moon usable; C++ incremental task.
		if let Some(current) = renderer.as_mut()
		{
			match current.advance_upgrade(gl)
			{
				Ok(true) => ctx.request_repaint(),
				Ok(false) =>
				{}
				Err(error) => loading.lock().error = Some(error),
			}
		}
	}

	fn announce(renderer: &mut Renderer, lighting: Option<Lighting>, ctx: &egui::Context)
	{
		// Announce a real lit draw, not merely decoded textures with pending sky data; C++ frame diagnostic.
		if lighting.is_some() && !renderer.visible_announced
		{
			let status = format!("Moon visible: {}", assets::label(renderer.tier));
			#[cfg(target_arch = "wasm32")]
			web_sys::console::log_1(&status.into());
			#[cfg(not(target_arch = "wasm32"))]
			eprintln!("{status}");
			renderer.visible_announced = true;
			// Give the first visible frame priority before requesting an upgrade; C++ event-loop wake.
			ctx.request_repaint();
		}
	}

	fn draw_marker(painter: &egui::Painter, position: egui::Pos2, site: &Marker, selected: bool)
	{
		// Bind this intermediate value for the current operation; C++ local initialization.
		let radius = if selected { 9.0 } else { 6.0 };
		painter.circle_filled(
			position,
			radius,
			if selected
			{
				egui::Color32::LIGHT_RED
			}
			else
			{
				egui::Color32::from_rgb(70, 230, 160)
			},
		);
		painter.circle_stroke(
			position,
			radius,
			egui::Stroke::new(2.0, egui::Color32::WHITE),
		);
		if selected
		{
			painter.text(
				position + egui::vec2(12.0, -12.0),
				egui::Align2::LEFT_BOTTOM,
				&site.name,
				egui::FontId::proportional(13.0),
				egui::Color32::WHITE,
			);
		}
	}
}
