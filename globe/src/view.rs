use super::*;

impl Globe
{
	/// Fill the available UI rectangle with interactive controls and optional one-minute rotation.
	pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize>
	{
		self.loading_status(ui);
		let time = ui.input(|i| i.time);
		let dt = (time - self.last_time.replace(time).unwrap_or(time)).clamp(0.0, 0.1) as f32;
		let lighting = self.scene_lighting(ui);
		self.animate_camera(time, dt);
		self.pending_label(ui);
		let (rect, response) = Self::paint_rectangle(ui);
		if !rect.is_positive()
		{
			return None;
		}
		self.orbit_input(ui, &response);
		let scale = ui.ctx().pixels_per_point();
		let view = math::View {
			rotation: self.body_rotation,
			yaw: self.yaw,
			tilt: self.tilt,
			zoom: self.zoom,
			pan: self.pan,
			aspect: rect.aspect_ratio(),
			pixels: [rect.width() * scale, rect.height() * scale],
			scale,
		};
		if self.has_visible_moon() && polar::wanted(view)
		{
			polar::advance(
				&self.polar,
				self.target_tier <= 2,
				self.deep.lock().is_disabled(),
				ui.ctx(),
			);
		}
		if self.has_visible_moon() && self.zoom >= 2.0
		{
			deep::advance(&self.deep, view, ui.ctx());
		}
		self.queue_gl(ui, rect, view, lighting);
		self.paint_grid(ui, rect, view);
		self.paint_points(ui, rect, view);
		let clicked = self.paint_markers(ui, &response, rect, view);
		self.surface_pick(&response, rect, view, clicked.is_some());
		Self::legend(
			ui,
			rect,
			&self.polar.lock(),
			!self.deep.lock().resident.is_empty(),
		);
		if self.zoom >= 2.0
		{
			let deep = self.deep.lock();
			let visible = deep
				.wanted
				.iter()
				.filter(|tile| deep.resident.contains(tile))
				.count();
			let status = if deep.error.is_some()
			{
				"; some detail unavailable"
			}
			else
			{
				""
			};
			let bottom = if self.points.is_empty()
			{
				-8.0
			}
			else
			{
				-ui.spacing().interact_size.y.max(36.0) - 12.0
			};
			text_box(
				&ui.painter_at(rect),
				rect.left_bottom() + egui::vec2(8.0, bottom),
				egui::Align2::LEFT_BOTTOM,
				format!(
					"Satellite mosaic: {visible}/{} tiles{status}\nShadows are recorded; lighting follows date",
					deep.wanted.len()
				),
				egui::Color32::WHITE,
			);
		}
		if self.space_view { space::paint(ui,rect,self.body_rotation,lighting); }
		if !self.paused || self.focus.is_some()
		{
			ui.ctx().request_repaint();
		}
		else if self.renderer.lock().is_none() && self.loading.lock().error.is_none()
		{
			ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
		}
		clicked.filter(|_| self.pick.is_none())
	}
}

impl Globe
{
	/// Let short layouts grow rather than covering the Moon with the legend.
	fn paint_rectangle(ui: &mut egui::Ui) -> (egui::Rect, egui::Response)
	{
		let size = egui::vec2(ui.available_width(), ui.available_height().max(200.0));
		let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
		let response = ui.interact(rect, ui.scope_id().with("globe-orbit"), egui::Sense::click_and_drag());
		(rect, response)
	}

	fn loading_status(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal(|ui| {
			ui.small("Moon overview");
			help::question(
				ui,
				"Left-drag to turn the Moon, right-drag to move the view, scroll to zoom, double-click to recenter.",
			);
		});
		if self
			.renderer
			.lock()
			.as_ref()
			.is_some_and(|r| r.visible_announced)
			&& self.tier != self.target_tier
		{
			self.tier = self.target_tier;
			self.loading = assets::load(self.tier, Some(ui.ctx().clone()));
		}
		if let Some(error) = self.loading.lock().error.as_ref()
		{
			help::annotated(
				ui,
				format!("Texture detail unavailable: {error}"),
				"Sharper images failed to load, so the current Moon image stays on screen.",
			);
		}
	}

	pub(super) fn light_sources(&mut self) -> Option<[[f32; 3]; 2]>
	{
		// Copy the selected site's actual directions, never infer illumination from availability flags.
		let selected = self
			.scene
			.iter()
			.find(|s| s.id == self.selected)
			.and_then(|s| {
				// Missing or invalid tracks have no real illumination fallback.
				let [sun, earth] = s.directions?;
				Some([
					moon_fixed(s.lat_deg, s.lon_deg, sun)?,
					moon_fixed(s.lat_deg, s.lon_deg, earth)?,
				])
			});
		if selected.is_some()
		{
			self.startup_light = None;
			self.last_light = selected;
		}
		// Moon-fixed Sun/Earth directions do not depend on the site, so a loading marker keeps the last light.
		let loading = self
			.scene
			.iter()
			.any(|s| s.id == self.selected && s.directions.is_none());
		selected
			.or(self.startup_light)
			.or(self.last_light.filter(|_| loading))
	}

	fn scene_lighting(&mut self, ui: &mut egui::Ui) -> Option<Lighting>
	{
		let directions = self.light_sources();
		// Holding this mode reveals real libration rather than artificial animation.
		if self.earth_view
			&& let Some([_, earth]) = directions
		{
			let earth = math::transform(self.body_rotation, earth);
			[self.yaw, self.tilt] = math::camera_angles(earth);
		}
		let lighting = directions.map(|[sun, earth]| Lighting {
			sun,
			earth,
			ratio: math::earthshine_ratio(sun, earth),
			boosted: self.boosted,
		});
		if self.earth_view
			&& let Some(light) = lighting
		{
			self.phase_status(ui, light);
		}
		if lighting.is_none()
		{
			ui.label("Real lighting unavailable: selected track not loaded");
		}
		lighting
	}

	fn animate_camera(&mut self, time: f64, dt: f32)
	{
		if let Some((start, from, to)) = self.focus
		{
			self.pan = [0.0; 2];
			let t = ((time - start) / 0.8).clamp(0.0, 1.0) as f32;
			let blend = t * t * (3.0 - 2.0 * t);
			self.yaw = from[0] + (to[0] - from[0]) * blend;
			self.tilt = from[1] + (to[1] - from[1]) * blend;
			self.zoom = (from[2].ln() + (to[2].ln() - from[2].ln()) * blend).exp();
			if t >= 1.0
			{
				self.focus = None;
			}
		}
		else if !self.paused
		{
			self.yaw += dt * std::f32::consts::TAU / 60.0;
		}
	}

	fn pending_label(&self, ui: &mut egui::Ui)
	{
		if self.renderer.lock().is_none()
		{
			let label = self
				.loading
				.lock()
				.error
				.clone()
				.unwrap_or_else(|| format!("Loading Moon {}...", assets::label(self.tier)));
			ui.label(label);
		}
	}

	fn orbit_input(&mut self, ui: &mut egui::Ui, response: &egui::Response)
	{
		if response.double_clicked_by(egui::PointerButton::Primary)
		{
			self.pan = [0.0; 2];
		}
		let (wheel, pinch, touch) =
			ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta(), i.multi_touch()));
		// Touch may lack hover; use the gesture center while respecting covering menus.
		let touching = touch.is_some_and(|touch| {
			response.interact_rect.contains(touch.center_pos)
				&& ui.ctx().layer_id_at(touch.center_pos) == Some(response.layer_id)
		});
		if response.hovered() || response.dragged() || touching
		{
			// Preserve focus animations unless an actual zoom gesture overrides them.
			if wheel != 0.0 || pinch != 1.0
			{
				self.zoom = (self.zoom * (wheel * 0.003).exp() * pinch).clamp(MIN_ZOOM, MAX_ZOOM);
				self.focus = None;
				// Consume the wheel before the surrounding page scroll area sees it.
				ui.input_mut(|i| i.smooth_scroll_delta.y = 0.0);
			}
		}
		if touch.is_none()
			&& (response.dragged_by(egui::PointerButton::Secondary)
				|| response.dragged_by(egui::PointerButton::Middle))
		{
			let delta = ui.input(|i| i.pointer.delta());
			self.pan[0] = (self.pan[0] + 2.0 * delta.x / response.rect.width()).clamp(-0.95, 0.95);
			self.pan[1] = (self.pan[1] - 2.0 * delta.y / response.rect.height()).clamp(-0.95, 0.95);
			self.focus = None;
		}
		else if response.dragged_by(egui::PointerButton::Primary) && touch.is_none()
		{
			self.earth_view = false;
			let delta = ui.input(|i| i.pointer.delta());
			self.yaw += delta.x * 0.006 / self.zoom;
			self.tilt = (self.tilt + delta.y * 0.006 / self.zoom)
				.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
			self.paused = true;
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
		let renderer = Arc::clone(&self.renderer);
		// Keep the loader alive until this callback finishes.
		let loading = Arc::clone(&self.loading);
		let polar = Arc::clone(&self.polar);
		let deep = Arc::clone(&self.deep);
		let rays = self.scene_rays();
		// Capture shared state for egui's later current-context draw.
		let surface_rays = self.surface_rays;
		let ctx = ui.ctx().clone();
		let callback = egui_glow::CallbackFn::new(move |info, painter| {
			let mut renderer = renderer.lock();
			Self::process_textures(&mut renderer, &loading, &ctx, painter.gl());
			let viewport = info.viewport_in_pixels();
			if viewport.width_px > 0 && viewport.height_px > 0
			{
				// Texture loading may still be pending; announce only a rendered frame.
				if let Some(renderer) = renderer.as_mut()
				{
					renderer.update_polar(painter.gl(), &polar);
					renderer.update_deep(painter.gl(), &deep);
					renderer.paint(
						painter.gl(),
						view,
						&rays,
						lighting,
						surface_rays,
						polar.lock().ready,
					);
					Self::announce(renderer, lighting, &ctx);
				}
			}
		});
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
		let mut clicked = None;
		let painter = ui.painter_at(rect);
		let pointer = response.interact_pointer_pos();
		let mut nearest: f32 = f32::INFINITY;
		for site in self
			.scene
			.iter()
			.filter(|s| s.id != self.selected)
			.chain(self.scene.iter().filter(|s| s.id == self.selected))
		{
			let point = math::direction(
				(site.lat_deg as f32).to_radians(),
				(site.lon_deg as f32).to_radians(),
			);
			if let Some(ndc) = view.project(point, true)
			{
				let position = rect.center()
					+ egui::vec2(ndc[0] * rect.width() * 0.5, -ndc[1] * rect.height() * 0.5);
				if !rect.expand(15.0).contains(position)
				{
					continue;
				}
				let selected = site.id == self.selected;
				Self::draw_marker(&painter, position, site, selected);
				if response.clicked_by(egui::PointerButton::Primary) && !response.double_clicked()
					&& let Some(p) = pointer
				{
					let distance = p.distance_sq(position);
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

	fn legend(ui: &mut egui::Ui, rect: egui::Rect, polar: &polar::State, satellite: bool)
	{
		let painter = ui.painter_at(rect);
		let caption = if satellite
		{
			"Satellite mosaic: shadows are recorded"
		}
		else if polar.ready
		{
			"Polar mosaic: shadows are recorded"
		}
		else if polar.error.is_some()
		{
			"Polar detail unavailable; overview retained"
		}
		else
		{
			"Illustration; satellite detail loads on zoom"
		};
		text_box(
			&painter,
			rect.left_top() + egui::vec2(8.0, 8.0),
			egui::Align2::LEFT_TOP,
			format!("Left-drag to turn; right/middle-drag to move\nScroll/pinch to zoom; double-click to recenter\n{caption}"),
			egui::Color32::LIGHT_GRAY,
		);
	}
}

fn text_box(
	painter: &egui::Painter,
	position: egui::Pos2,
	align: egui::Align2,
	text: String,
	color: egui::Color32,
) -> egui::Rect
{
	let galley = painter.layout_no_wrap(text, egui::FontId::proportional(11.0), color);
	let bounds = align.anchor_size(position, galley.size());
	let background = bounds.expand(3.0);
	painter.rect_filled(background, 3.0, egui::Color32::from_black_alpha(225));
	painter.galley(bounds.min, galley, color);
	background
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn overlay_backgrounds_contain_multiline_text()
	{
		let ctx = egui::Context::default();
		let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
			for align in [egui::Align2::LEFT_TOP, egui::Align2::LEFT_BOTTOM]
			{
				for text in ["", "Illustration; satellite detail loads on zoom",
					"Left-drag to turn; right/middle-drag to move\nScroll/pinch to zoom; double-click to recenter\nSatellite mosaic: shadows are recorded",
					"Satellite mosaic: 73/79 tiles; some detail unavailable\nShadows are recorded; lighting follows date"]
				{
					text_box(ui.painter(), egui::pos2(10.0, 100.0), align,
						text.into(), egui::Color32::WHITE);
				}
			}
		});
		output.textures_delta.clear();
		let mut background = egui::Rect::NOTHING;
		let mut count = 0;
		for shape in output.shapes
		{
			match shape.shape
			{
				egui::Shape::Rect(rect) => background = rect.rect,
				egui::Shape::Text(text) =>
				{
					assert!(background.contains_rect(egui::Rect::from_min_size(text.pos, text.galley.size())));
					count += 1;
				}
				_ => {}
			}
		}
		assert!(count >= 6);
	}

	#[test]
	fn primary_drag_orbits_and_secondary_and_middle_drag_pan_without_picking()
	{
		for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary, egui::PointerButton::Middle]
		{
			let ctx = egui::Context::default();
			let mut globe = crate::scene_tests::unloaded();
			globe.paused = false;
			globe.focus = Some((0.0, [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]));
			let mut rect = egui::Rect::NOTHING;
			let mut frame = |events| {
				let mut output = ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
					let (area, response) = Globe::paint_rectangle(ui);
					rect = area;
					globe.orbit_input(ui, &response);
					globe.surface_pick(&response, area, math::View {
						rotation: math::IDENTITY,
						yaw: globe.yaw,
						tilt: globe.tilt,
						zoom: globe.zoom,
						pan: globe.pan,
						aspect: area.aspect_ratio(),
						pixels: [area.width(), area.height()],
						scale: 1.0,
					}, false);
				});
				output.textures_delta.clear();
				rect
			};
			let rect = frame(vec![]);
			let start = rect.center();
			frame(vec![egui::Event::PointerMoved(start), egui::Event::PointerButton {
				pos: start, button, pressed: true, modifiers: egui::Modifiers::NONE,
			}]);
			let end = start + egui::vec2(30.0, 20.0);
			frame(vec![egui::Event::PointerMoved(end)]);
			frame(vec![egui::Event::PointerButton {
				pos: end, button, pressed: false, modifiers: egui::Modifiers::NONE,
			}]);
			if button == egui::PointerButton::Primary
			{
				assert_eq!(globe.pan, [0.0; 2]);
				assert!((globe.yaw - 0.18).abs() < 1e-5);
				assert!((globe.tilt - 0.12).abs() < 1e-5);
				assert!(globe.paused);
				assert!(!globe.earth_view);
			}
			else
			{
				assert!((globe.pan[0] - 60.0 / rect.width()).abs() < 1e-5);
				assert!((globe.pan[1] + 40.0 / rect.height()).abs() < 1e-5);
				assert_eq!([globe.yaw, globe.tilt], [0.0; 2]);
				assert!(!globe.paused);
			}
			assert!(globe.focus.is_none());
			assert!(globe.take_pick().is_none());
		}
	}

	#[test]
	fn pan_clamps_and_double_click_and_focus_recenter()
	{
		let ctx = egui::Context::default();
		let mut globe = crate::scene_tests::unloaded();
		let mut frame = |time, events| {
			let mut rect = egui::Rect::NOTHING;
			let mut output = ctx.run_ui(egui::RawInput { time: Some(time), events, ..Default::default() }, |ui| {
				let (area, response) = Globe::paint_rectangle(ui);
				rect = area;
				globe.orbit_input(ui, &response);
			});
			output.textures_delta.clear();
			(rect, globe.pan)
		};
		let (rect, _) = frame(0.0, vec![]);
		let start = rect.center();
		let button = |pos, button, pressed| egui::Event::PointerButton {
			pos, button, pressed, modifiers: egui::Modifiers::NONE,
		};
		frame(0.1, vec![egui::Event::PointerMoved(start), button(start, egui::PointerButton::Secondary, true)]);
		let end = start + rect.size() * 10.0;
		let (_, pan) = frame(0.2, vec![egui::Event::PointerMoved(end)]);
		assert_eq!(pan, [0.95, -0.95]);
		frame(0.3, vec![button(end, egui::PointerButton::Secondary, false)]);
		for (time, pressed) in [(1.0, true), (1.05, false), (1.1, true), (1.15, false)]
		{
			frame(time, vec![egui::Event::PointerMoved(start), button(start, egui::PointerButton::Primary, pressed)]);
		}
		assert_eq!(globe.pan, [0.0; 2]);
		globe.pan = [0.5, -0.5];
		globe.focus = Some((2.0, [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]));
		globe.animate_camera(2.2, 0.1);
		assert_eq!(globe.pan, [0.0; 2]);
	}

	#[test]
	fn orbit_interaction_id_survives_phase_controls_changing()
	{
		let ctx = egui::Context::default();
		let mut ids = Vec::new();
		for controls in [0, 3, 0]
		{
			let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
				egui::CentralPanel::default().show(root, |ui| {
					for _ in 0..controls { ui.label("Phase control"); }
					ids.push(Globe::paint_rectangle(ui).1.id);
				});
			});
			output.textures_delta.clear();
		}
		assert!(ids.windows(2).all(|pair| pair[0] == pair[1]));
	}

	#[test]
	fn pinch_uses_touch_center_without_pointer_hover()
	{
		let ctx = egui::Context::default();
		let mut globe = crate::scene_tests::unloaded();
		globe.zoom = 8.0;
		let touches = |phase, distance: f32| {
			[11, 22]
				.into_iter()
				.enumerate()
				.map(|(index, id)| egui::Event::Touch {
					device_id: egui::TouchDeviceId(0),
					id: egui::TouchId(id),
					phase,
					pos: egui::pos2(206.0 + (index as f32 - 0.5) * distance, 250.0),
					force: None,
				})
				.collect()
		};
		let mut frame = |events| {
			let mut output = ctx.run_ui(
				egui::RawInput {
					screen_rect: Some(egui::Rect::from_min_size(
						egui::Pos2::ZERO,
						egui::vec2(412.0, 839.0),
					)),
					events,
					..Default::default()
				},
				|root| {
					egui::CentralPanel::default().show(root, |ui| {
						let (_, response) = Globe::paint_rectangle(ui);
						globe.orbit_input(ui, &response);
					});
				},
			);
			output.textures_delta.clear();
			globe.zoom
		};
		frame(vec![egui::Event::PointerMoved(egui::Pos2::ZERO)]);
		frame(touches(egui::TouchPhase::Start, 40.0));
		assert!((frame(touches(egui::TouchPhase::Move, 280.0)) - 56.0).abs() < 0.01);
		assert!((frame(touches(egui::TouchPhase::Move, 40.0)) - 8.0).abs() < 0.01);
		frame(touches(egui::TouchPhase::End, 40.0));
		assert_eq!([globe.yaw, globe.tilt], [0.0, 0.0]);
		assert!(globe.earth_view);
	}

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
		let rays: Vec<_> = self
			.scene
			.iter()
			.filter_map(|site| {
				let origin = math::direction(
					(site.lat_deg as f32).to_radians(),
					(site.lon_deg as f32).to_radians(),
				);
				let bodies = site.directions?;
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
		if loading.lock().error.is_none()
		{
			let result = loading.lock().take();
			match result
			{
				Ok(Some(images)) =>
				{
					if let Some(current) = renderer.as_mut()
					{
						// Keep old textures until every replacement is uploaded.
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
		// Announce a real lit draw, not merely decoded textures with pending sky data.
		if lighting.is_some() && !renderer.visible_announced
		{
			let status = format!("Moon visible: {}", assets::label(renderer.tier));
			#[cfg(target_arch = "wasm32")]
			web_sys::console::log_1(&status.into());
			#[cfg(not(target_arch = "wasm32"))]
			eprintln!("{status}");
			renderer.visible_announced = true;
			// Give the first visible frame priority before requesting a texture upgrade.
			ctx.request_repaint();
		}
	}

	fn draw_marker(painter: &egui::Painter, position: egui::Pos2, site: &Marker, selected: bool)
	{
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
