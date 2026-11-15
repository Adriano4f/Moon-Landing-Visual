//! An illustrative lunar renderer, independent of the landing-site scientific library.
//! See docs/03-moon.md for coordinate, photometric, color, and GL lifetime contracts.

// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod assets;
mod field;
// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod math;
// Share accessible explanatory widgets without introducing an app dependency; C++ public utility namespace.
pub mod help;

// Bound enlargement by the available polar texture detail, not GPU capacity; C++ constexpr float.
const MAX_ZOOM: f32 = 8.0;

// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui::mutex::Mutex;
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use egui_glow::glow::{self, HasContext as _};
// Bring these names into scope; C++ uses qualified declarations, using, and header inclusion.
use std::sync::Arc;

// Copy a local sky direction across the app/renderer boundary without a core dependency; C++ plain value struct.
#[derive(Clone, Copy, Debug)]
pub struct BodyDirection
{
	// Azimuth is measured clockwise from north, in degrees; C++ float.
	pub az_deg: f32,
	// Elevation is measured above the site's horizontal plane, in degrees; C++ float.
	pub el_deg: f32,
	// The app supplies its terrain/clearance availability predicate; C++ bool.
	pub visible: bool,
}

// Convert a topocentric sky angle to the renderer's Moon-fixed unit vector; C++ optional<array<float,3>>.
pub fn moon_fixed(lat_deg: f64, lon_deg: f64, body: BodyDirection) -> Option<[f32; 3]>
{
	// Reuse the tested east/north/up basis independently of local visibility; C++ checked coordinate transform.
	math::local_direction(
		(lat_deg as f32).to_radians(),
		(lon_deg as f32).to_radians(),
		body.az_deg.to_radians(),
		body.el_deg.to_radians(),
	)
}

// Snapshot real directions and explicitly optional display aids for one deferred draw; C++ value aggregate.
#[derive(Clone, Copy)]
struct Lighting
{
	// Store the selected epoch's incoming solar direction; C++ fixed-size vector.
	sun: [f32; 3],
	// Store the selected epoch's incoming Earth direction; C++ fixed-size vector.
	earth: [f32; 3],
	// Express visible reflected irradiance relative to sunlight; C++ float.
	ratio: f32,
	// Keep the optional Earth-only exposure multiplication explicit; C++ bool.
	boosted: bool,
	// Keep artificial far-side light independent of physical energy; C++ bool.
	fill: bool,
}

// Keep transformed directions distinct from local input angles; C++ internal plain value struct.
#[derive(Clone, Copy)]
struct Ray
{
	// Moon-fixed unit direction before presentation rotation; C++ array<float,3>.
	direction: [f32; 3],
	// False selects the dim material, independently of depth occlusion; C++ bool.
	visible: bool,
}

// Lightweight catalog metadata plus optional current scientific sample; C++ value struct.
#[derive(Clone)]
pub struct Marker
{
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub id: usize,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub name: String,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub lat_deg: f64,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub lon_deg: f64,
	// Store this public value without platform-specific ownership; C++ aggregate member.
	pub directions: Option<[BodyDirection; 2]>,
}

/// Owns the callback's shared renderer. Call destroy from eframe's on_exit with a current context.
// Declare named fields; Rust methods live in a separate impl, unlike inline C++ class definitions.
pub struct Globe
{
	// Shared optional GPU state, absent while loading; C++ shared_ptr<mutex-protected optional<Renderer>>.
	renderer: Arc<Mutex<Option<Renderer>>>,
	// Shared CPU-side completion state; C++ shared_ptr to a mutex-protected Loading.
	loading: Arc<Mutex<assets::Loading>>,
	// Selected size label 2/4/8, with width equal to 1,024 times the label; C++ uint32_t.
	tier: u32,
	// Requested final detail is independent of the first visible 2k pair; C++ unsigned members.
	target_tier: u32,
	max_tier: u32,
	// Optional illustrative parallel Sun rays; C++ bool.
	surface_rays: bool,
	// Interactive presentation state is independent of ephemeris time; C++ value members.
	scene: Vec<Marker>,
	selected: usize,
	yaw: f32,
	tilt: f32,
	zoom: f32,
	paused: bool,
	// Track the current ephemeris Earth line of sight instead of spinning; C++ bool.
	earth_view: bool,
	// Multiply only Earthshine for display when explicitly requested; C++ bool.
	boosted: bool,
	// Add an explicitly artificial far-side fill only on request; C++ bool.
	fill: bool,
	last_time: Option<f64>,
	focus: Option<(f64, [f32; 3], [f32; 3])>,
}

// Exercise scene validation without opening an OpenGL context; C++ isolated test namespace.
#[cfg(test)]
mod scene_tests
{
	// Reuse the production state and aggregate definitions; C++ qualified test access.
	use super::*;

	// Verify invalid entries cannot shift stable identifiers or retain a stale transition; C++ test registration.
	#[test]
	fn scene_identity_empty_and_invalid()
	{
		// Supply uninitialized GPU state so no renderer or network is needed; C++ aggregate test fixture.
		let mut globe = Globe {
			renderer: Arc::new(Mutex::new(None)),
			loading: Arc::new(Mutex::new(assets::Loading {
				images: [None, None],
				error: None,
			})),
			tier: 2,
			target_tier: 2,
			max_tier: 8,
			surface_rays: false,
			scene: vec![],
			selected: 0,
			yaw: 0.0,
			tilt: 0.0,
			zoom: 1.0,
			paused: true,
			earth_view: true,
			boosted: false,
			fill: false,
			last_time: None,
			focus: Some((0.0, [0.0; 3], [1.0; 3])),
		};
		// Keep the valid identifier deliberately different from its vector index; C++ aggregate fixture.
		let valid = Marker {
			id: 17,
			name: "Published".into(),
			lat_deg: -89.0,
			lon_deg: 220.0,
			directions: None,
		};
		// Represent malformed external metadata without corrupting the valid entry; C++ aggregate copy override.
		let invalid = Marker {
			id: 8,
			lat_deg: f64::NAN,
			..valid.clone()
		};
		// Apply the same filtering path used on every application redraw; C++ scene setter.
		globe.set_scene(vec![invalid, valid], 17);
		// Filtering must not renumber the published marker identifier; C++ collection assertion.
		assert_eq!(
			globe.scene.iter().map(|s| s.id).collect::<Vec<_>>(),
			vec![17]
		);
		// A changed selection cancels the previous site's camera animation; C++ optional-state assertion.
		assert!(globe.focus.is_none());
		// Empty data clears markers and rays instead of retaining old state; C++ empty-container setter.
		globe.set_scene(vec![], 17);
		// No stale scientific direction survives scene replacement; C++ collection assertion.
		assert!(globe.scene.is_empty());
	}
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Globe
{
	// Construct and validate owned state, returning an error instead of throwing; C++ uses a factory returning expected.
	pub fn new(gl: &Arc<glow::Context>) -> Result<Self, String>
	{
		// Record whether the browser looks mobile; C++ stores the corresponding bool.
		let (mobile, forced) = assets::preferences();
		// SAFETY: eframe makes this context current before calling the constructor.
		// Clamp the requested tier to the current GPU's maximum width; C++ passes a const reference to preferences.
		let tier = assets::select(
			// Read an integer capability into Rust; C++ glGetIntegerv writes through an output pointer.
			// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
			unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
			mobile,
			forced.as_deref(),
		)?;
		// Format a visible diagnostic naming the chosen tier; C++ std::format returns a string.
		let status = format!("Moon tier {tier}k (mobile={mobile})");
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		eprintln!("{status}");
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		web_sys::console::log_1(&status.into());
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(Self {
			// Shared optional GPU state, absent while loading; C++ shared_ptr<mutex-protected optional<Renderer>>.
			renderer: Arc::new(Mutex::new(None)),
			// Shared CPU-side completion state; C++ shared_ptr to a mutex-protected Loading.
			surface_rays: false,
			loading: assets::load(2, None),
			tier: 2,
			target_tier: tier,
			max_tier: assets::select(
				unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
				false,
				Some("8"),
			)?,
			// Default to the current Earth line of sight, without artificial spin or tilt; C++ scalar initialization.
			scene: Vec::new(),
			selected: 0,
			yaw: 0.0,
			tilt: 0.0,
			zoom: 1.0,
			paused: true,
			earth_view: true,
			boosted: false,
			fill: false,
			last_time: None,
			focus: None,
		})
	}

	// Copy metadata for every marker and scientific directions only for requested sites; C++ scene value transfer.
	pub fn set_scene(&mut self, scene: Vec<Marker>, selected: usize)
	{
		// Cancel an in-flight transition if another site is chosen; C++ identity guard.
		if self.selected != selected
		{
			// Update this persistent member explicitly; C++ member assignment.
			self.focus = None;
		}
		// Update this persistent member explicitly; C++ member assignment.
		self.scene = scene
			.into_iter()
			.filter(|m| {
				m.lat_deg.is_finite()
					&& m.lon_deg.is_finite()
					&& (-90.0..=90.0).contains(&m.lat_deg)
			})
			.collect();
		// Update this persistent member explicitly; C++ member assignment.
		self.selected = selected;
	}

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

	// Expose the independent surface field without requesting more site tracks; C++ checkbox.
	pub fn surface_ray_controls(&mut self, ui: &mut egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.checkbox(&mut self.surface_rays, "Rays across the whole Moon");
			help::question(ui, "Parallel yellow rays arrive from the real Sun direction at this date and end on the sunlit sphere. About 256 of 512 evenly distributed points are lit. Lengths are illustrative; this is not terrain shadow tracing. The Moon hides rays behind its surface.");
		});
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

	/// Fill the available UI rectangle with interactive controls and optional one-minute rotation.
	// Queue a paint callback for this frame and request the next; C++ an immediate-mode UI calls this each frame.
	pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize>
	{
		// Keep direct manipulation discoverable when the settings group is closed; C++ explanatory heading row.
		ui.horizontal(|ui| {
			ui.small("Moon overview");
			help::question(ui, "Drag to orbit the Moon; wheel or pinch to zoom. Magnification stops at 8 times to avoid misleading polar texture streaks. Markers are published points; click one or use Site. This globe is an illustration, not the detailed terrain silhouette shown in the panorama.");
		});
		// Display 2k before starting an optional upgrade; never blank the existing renderer while waiting.
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
				"The last successful Moon remains visible if higher detail fails. Try another detail level in View / camera after an upgrade fails, or reload the page if the first images fail. Scientific data is independent of globe textures.",
			);
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let time = ui.input(|i| i.time);
		// Bind this intermediate value for the current operation; C++ local initialization.
		let dt = (time - self.last_time.replace(time).unwrap_or(time)).clamp(0.0, 0.1) as f32;
		// Bind this intermediate value for the current operation; C++ local initialization.
		let _ = time;
		// Copy the selected site's actual directions, never infer illumination from availability flags; C++ optional transform.
		let directions = self
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
		// Explain unavailable data rather than silently restoring a fixed presentation Sun; C++ conditional label.
		if lighting.is_none()
		{
			// State why the Moon has no physical illumination instead of fabricating data; C++ immediate-mode text.
			ui.label("Real lighting unavailable: selected track not loaded");
		}
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
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut clicked = None;
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
				.unwrap_or_else(|| format!("Loading Moon {}k...", self.tier));
			// Paint the pending/error message on the black panel; C++ immediate-mode UIs similarly submit a label each frame.
			ui.label(label);
		}
		// Reserve the available paint rectangle; C++ would receive a rectangle/response pair.
		let (rect, response) =
			ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if !rect.is_positive()
		{
			// Stop this frame's work without a value; C++ return exits a void function.
			return None;
		}
		// Wheel and touch pinch operate only inside the globe, not the panorama scroll area; C++ scoped event handling.
		if response.hovered()
		{
			// Bind this intermediate value for the current operation; C++ local initialization.
			let (wheel, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
			// Check this state or input before acting; C++ conditional branch.
			if wheel != 0.0 || pinch != 1.0
			{
				// Update this persistent member explicitly; C++ member assignment.
				self.zoom = (self.zoom * (wheel * 0.003).exp() * pinch).clamp(1.0, MAX_ZOOM);
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
		// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
		let renderer = Arc::clone(&self.renderer);
		// Keep the loader alive until this callback finishes; C++ copies its shared_ptr.
		let loading = Arc::clone(&self.loading);
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
		// Capture shared state for egui's later current-context draw; C++ stores a value-capturing std::function.
		let surface_rays = self.surface_rays;
		let ctx = ui.ctx().clone();
		let callback = egui_glow::CallbackFn::new(move |info, painter| {
			// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
			let mut renderer = renderer.lock();
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
							current.begin_upgrade(painter.gl(), images);
						}
						else
						{
							match Renderer::new(painter.gl(), images)
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
				match current.advance_upgrade(painter.gl())
				{
					Ok(true) => ctx.request_repaint(),
					Ok(false) =>
					{}
					Err(error) => loading.lock().error = Some(error),
				}
			}
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
					// Announce a real lit draw, not merely decoded textures with pending sky data; C++ frame diagnostic.
					if lighting.is_some() && !renderer.visible_announced
					{
						let status = format!("Moon visible: {}k", renderer.tier);
						#[cfg(target_arch = "wasm32")]
						web_sys::console::log_1(&status.into());
						#[cfg(not(target_arch = "wasm32"))]
						eprintln!("{status}");
						renderer.visible_announced = true;
						// Give the first visible frame priority before requesting an upgrade; C++ event-loop wake.
						ctx.request_repaint();
					}
				}
			}
		});
		// Append the custom GL callback to egui's paint list; C++ queues an owned rendering command.
		ui.painter().add(egui::PaintCallback {
			rect,
			callback: Arc::new(callback),
		});
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
		// Keep the legend readable when the optional field crosses it; C++ translucent overlay rectangle.
		painter.rect_filled(
			egui::Rect::from_min_size(
				rect.left_top() + egui::vec2(5.0, 5.0),
				egui::vec2(258.0, 59.0),
			),
			3.0,
			egui::Color32::from_black_alpha(225),
		);
		painter.text(rect.left_top()+egui::vec2(8.0,8.0),egui::Align2::LEFT_TOP,"Drag to orbit; wheel / pinch to zoom\nYellow Sun | blue Earth | dim = blocked\nReal light; fill is a visual aid; rays not to scale\nPolar cap schematic; terrain detail in panorama",egui::FontId::proportional(11.0),egui::Color32::LIGHT_GRAY);
		// Request another frame so loading and animation progress; C++ asks the event loop to redraw.
		ui.ctx().request_repaint();
		clicked
	}

	// Release registered GPU objects using the current context; C++ explicit teardown avoids a late invalid-context destructor.
	pub fn destroy(&mut self, gl: &glow::Context)
	{
		// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
		if let Some(mut renderer) = self.renderer.lock().take()
		{
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			renderer.destroy(gl);
		}
	}
}

// Declare named fields; Rust methods live in a separate impl, unlike inline C++ class definitions.
struct Renderer
{
	// Optional owned linked-program handle; C++ optional<GLuint>.
	program: Option<glow::Program>,
	// Own any not-yet-deleted shader handles during construction; C++ vector<GLuint>.
	shaders: Vec<glow::Shader>,
	// Optional owned vertex-array handle; C++ optional<GLuint>.
	vao: Option<glow::VertexArray>,
	// Own a separate position-only layout for the two direction lines; C++ optional<GLuint>.
	ray_vao: Option<glow::VertexArray>,
	// Alias a buffer owned by buffers for dynamic ray uploads; C++ nonowning optional<GLuint>.
	ray_buffer: Option<glow::Buffer>,
	// Own vertex and index buffer handles; C++ vector<GLuint>.
	buffers: Vec<glow::Buffer>,
	// Own the albedo and normal textures in sampler order; C++ vector<GLuint>.
	textures: Vec<glow::Texture>,
	// Number of indices passed to the draw, not the number of triangles; C++ GLsizei.
	index_count: i32,
	// Record the quality actually uploaded, independently of the requested upgrade; C++ unsigned member.
	tier: u32,
	// Signal only after the renderer has drawn with real sky directions; C++ bool.
	visible_announced: bool,
	// Retain decoded upgrades between bounded GPU-upload frames; C++ optional pair with image iterator.
	pending: Option<(u32, std::vec::IntoIter<assets::TextureImage>)>,
}

// Attach methods to this type; C++ declares these as member functions inside the class.
impl Renderer
{
	// Construct and validate owned state, returning an error instead of throwing; C++ uses a factory returning expected.
	fn new(gl: &glow::Context, [albedo, normals]: [assets::TextureImage; 2])
	-> Result<Self, String>
	{
		// Inspect GLSL capabilities rather than assuming desktop syntax; C++ queries GL strings.
		let version = egui_glow::ShaderVersion::get(gl);
		// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
		if !version.is_new_shader_interface()
		{
			// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
			return Err("The Moon requires modern OpenGL or WebGL2".into());
		}
		// Own or temporarily access renderer state without copying GPU resources; C++ shared_ptr or lock_guard.
		let mut renderer = Self {
			// Optional owned linked-program handle; C++ optional<GLuint>.
			program: None,
			// Own any not-yet-deleted shader handles during construction; C++ vector<GLuint>.
			shaders: Vec::new(),
			// Optional owned vertex-array handle; C++ optional<GLuint>.
			vao: None,
			// Ray resources participate in the same partial-construction cleanup; C++ nullopt initialization.
			ray_vao: None,
			ray_buffer: None,
			// Own vertex and index buffer handles; C++ vector<GLuint>.
			buffers: Vec::new(),
			// Own the albedo and normal textures in sampler order; C++ vector<GLuint>.
			textures: Vec::new(),
			// Number of indices passed to the draw, not the number of triangles; C++ GLsizei.
			index_count: 0,
			tier: albedo.width() / 1024,
			visible_announced: false,
			pending: None,
		};
		// Run fallible GPU construction in a local closure so partial resources can be cleaned; C++ try/catch is analogous.
		let initialized = (|| -> Result<(), String> {
			// SAFETY: eframe calls the constructor with its GL context current. Every handle is
			// registered immediately so the error branch can clean partial construction.
			// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
			unsafe {
				// Allocate a linkable program; C++ glCreateProgram returns a GLuint, while glow can return an error.
				let program = gl.create_program()?;
				// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
				renderer.program = Some(program);
				// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
				for (kind, source) in [
					// Embed only the vertex shader's small UTF-8 source, never image tiers; C++ embeds a string literal.
					(glow::VERTEX_SHADER, include_str!("moon.vert")),
					// Embed the matching fragment shader; C++ embeds another shader source string.
					(glow::FRAGMENT_SHADER, include_str!("moon.frag")),
				]
				{
					// Allocate one shader stage; C++ glCreateShader takes the same stage enum.
					let shader = gl.create_shader(kind)?;
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.shaders.push(shader);
					// Supply GLSL text with the selected desktop/WebGL version; C++ glShaderSource takes pointer arrays.
					gl.shader_source(
						shader,
						&format!("{}\n{source}", version.version_declaration()),
					);
					// Compile the stage now; C++ glCompileShader uses the same handle.
					gl.compile_shader(shader);
					// Test compilation before attaching; C++ glGetShaderiv with GL_COMPILE_STATUS.
					if !gl.get_shader_compile_status(shader)
					{
						// Return the compiler diagnostic as String; C++ glGetShaderInfoLog fills a character buffer.
						return Err(gl.get_shader_info_log(shader));
					}
					// Attach this compiled stage to the program; C++ glAttachShader.
					gl.attach_shader(program, shader);
				}
				// Fix the input location to match the vertex layout; C++ glBindAttribLocation before linking.
				gl.bind_attrib_location(program, 0, "a_position");
				// Fix the input location to match the vertex layout; C++ glBindAttribLocation before linking.
				gl.bind_attrib_location(program, 1, "a_uv");
				gl.bind_attrib_location(program, 2, "a_coverage");
				// Connect vertex outputs to fragment inputs; C++ glLinkProgram.
				gl.link_program(program);
				// Reject incompatible stages; C++ glGetProgramiv with GL_LINK_STATUS.
				if !gl.get_program_link_status(program)
				{
					// Recover the linker's diagnostic; C++ glGetProgramInfoLog.
					return Err(gl.get_program_info_log(program));
				}
				// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
				for shader in renderer.shaders.drain(..)
				{
					// Release the program's stage attachment after a successful link; C++ glDetachShader.
					gl.detach_shader(program, shader);
					// Delete the no-longer-needed shader object; C++ glDeleteShader.
					gl.delete_shader(shader);
				}
				// Own interleaved position/UV values on the CPU; C++ vector<float>.
				let (vertices, indices) = math::sphere(256, 128);
				// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
				renderer.index_count = indices.len() as i32;
				// Allocate vertex-layout state; C++ glGenVertexArrays.
				renderer.vao = Some(gl.create_vertex_array()?);
				// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
				gl.bind_vertex_array(renderer.vao);
				// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
				for (target, bytes) in [
					// Borrow the float vertex vector as bytes without copying; C++ reinterpret_cast to a const byte span.
					(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices)),
					// Borrow the u32 index vector as upload bytes; C++ a const byte span preserves the original storage.
					(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&indices)),
				]
				{
					// Allocate storage ownership for vertices or indices; C++ glGenBuffers.
					let buffer = gl.create_buffer()?;
					// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
					renderer.buffers.push(buffer);
					// Select the buffer receiving the next operation; C++ glBindBuffer(target, handle or 0).
					gl.bind_buffer(target, Some(buffer));
					// Copy CPU bytes to static-draw GPU storage; C++ glBufferData(target, byte_count, data, GL_STATIC_DRAW).
					gl.buffer_data_u8_slice(target, bytes, glow::STATIC_DRAW);
				}
				// Enable a vertex input array at this location; C++ glEnableVertexAttribArray.
				gl.enable_vertex_attrib_array(0);
				// Describe floats, byte stride, and byte offset in each vertex; C++ glVertexAttribPointer.
				gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 20, 0);
				// Enable a vertex input array at this location; C++ glEnableVertexAttribArray.
				gl.enable_vertex_attrib_array(1);
				// Describe floats, byte stride, and byte offset in each vertex; C++ glVertexAttribPointer.
				gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 20, 12);
				// Allocate a second VAO without disturbing the Moon's indexed mesh; C++ glGenVertexArrays.
				renderer.ray_vao = Some(gl.create_vertex_array()?);
				// Configure only this layout for position triples; C++ glBindVertexArray.
				gl.bind_vertex_array(renderer.ray_vao);
				// Allocate a tiny dynamic buffer for four ray vertices; C++ glGenBuffers.
				let ray_buffer = gl.create_buffer()?;
				// Register ownership immediately for cleanup on later failure; C++ owned handle vector append.
				renderer.buffers.push(ray_buffer);
				// Retain a nonowning handle for subsequent uploads; C++ handle copy.
				renderer.ray_buffer = Some(ray_buffer);
				// Make the new buffer the source of position attribute zero; C++ glBindBuffer.
				gl.bind_buffer(glow::ARRAY_BUFFER, Some(ray_buffer));
				// Enable only position; unused UV defaults to zero for the constant-color branch; C++ glEnableVertexAttribArray.
				gl.enable_vertex_attrib_array(0);
				// Three tightly packed floats describe each ray vertex; C++ glVertexAttribPointer.
				gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 28, 0);
				gl.enable_vertex_attrib_array(1);
				gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 28, 12);
				gl.enable_vertex_attrib_array(2);
				gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 28, 20);
				// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
				gl.bind_vertex_array(None);
				// Select the buffer receiving the next operation; C++ glBindBuffer(target, handle or 0).
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
			// Upload each image once; the browser keeps decoded pixels in its native image source.
			for (image, srgb) in [(albedo, true), (normals, false)]
			{
				#[cfg(not(target_arch = "wasm32"))]
				renderer.upload_texture(gl, image.width(), image.height(), image.as_raw(), srgb)?;
				#[cfg(target_arch = "wasm32")]
				renderer.upload_browser_texture(gl, &image, srgb)?;
			}
			// Wrap a successful result without throwing; C++ expected stores its value.
			Ok(())
		})();
		// Detect a failed initialization so partial GPU ownership can be cleaned; C++ checks expected's error.
		if let Err(error) = initialized
		{
			// Update or use renderer-owned resources rather than letting handles escape; C++ accesses the corresponding owner object.
			renderer.destroy(gl);
			// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
			return Err(error);
		}
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(not(target_arch = "wasm32"))]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		eprintln!("Moon textures ready");
		// Compile only for this target or test configuration; C++ uses #if, but Rust removes this entire syntax item.
		#[cfg(target_arch = "wasm32")]
		// Print tier/readiness diagnostics for verification; C++ uses its logging or stream API.
		web_sys::console::log_1(&"Moon textures ready".into());
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(renderer)
	}

	// Begin a new quality request while retaining the last complete pair; C++ cancellation and staging.
	fn begin_upgrade(&mut self, gl: &glow::Context, images: [assets::TextureImage; 2])
	{
		// Discard only abandoned replacement textures, never the visible pair; C++ GL owner cleanup.
		for texture in self.textures.drain(2..)
		{
			unsafe {
				gl.delete_texture(texture);
			}
		}
		// Store the real image width and consume the pair in order; C++ pair<unsigned, image iterator>.
		self.pending = Some((images[0].width() / 1024, Vec::from(images).into_iter()));
	}

	// Advance one native/browser upload per frame, then atomically expose the complete pair; C++ state machine.
	fn advance_upgrade(&mut self, gl: &glow::Context) -> Result<bool, String>
	{
		// Temporarily own the in-progress upload, restoring it only after success; C++ optional exchange.
		let Some((tier, mut images)) = self.pending.take()
		else
		{
			return Ok(false);
		};
		// Process only one image in this frame; C++ iterator increment and optional extraction.
		if let Some(image) = images.next()
		{
			// The first replacement is albedo, the second is directional data; C++ bool.
			let srgb = self.textures.len() == 2;
			#[cfg(not(target_arch = "wasm32"))]
			let result =
				self.upload_texture(gl, image.width(), image.height(), image.as_raw(), srgb);
			#[cfg(target_arch = "wasm32")]
			let result = self.upload_browser_texture(gl, &image, srgb);
			if let Err(error) = result
			{
				// SAFETY: this callback owns the current GL context; C++ delete only registered replacement handles.
				for texture in self.textures.drain(2..)
				{
					unsafe {
						gl.delete_texture(texture);
					}
				}
				return Err(error);
			}
			// Resume at the next image in a later frame; C++ iterator move into optional.
			self.pending = Some((tier, images));
			return Ok(true);
		}
		// SAFETY: old textures are no longer sampled after this callback-local swap; C++ owner retirement.
		for texture in self.textures.drain(..2)
		{
			unsafe {
				gl.delete_texture(texture);
			}
		}
		// Expose the completed pair and schedule a new actual-visibility diagnostic; C++ member assignments.
		self.tier = tier;
		self.visible_announced = false;
		Ok(false)
	}

	// Upload RGB bytes and configure mipmapped sampling; C++ receives a const byte span plus dimensions.
	#[cfg(not(target_arch = "wasm32"))]
	fn upload_texture(
		&mut self,
		gl: &glow::Context,
		width: u32,
		height: u32,
		pixels: &[u8],
		srgb: bool,
	) -> Result<(), String>
	{
		// WebGL2 can generate mipmaps for SRGB8_ALPHA8, but not for bare SRGB8.
		// Expand only albedo to opaque RGBA; C++ builds the same vector from RGB triples.
		let rgba: Option<Vec<u8>> = srgb.then(|| {
			pixels
				.as_chunks::<3>()
				.0
				.iter()
				.flat_map(|p| [p[0], p[1], p[2], 255])
				.collect()
		});
		// Borrow expanded bytes when present, otherwise the original normal-map bytes; C++ span.
		let upload = rgba.as_deref().unwrap_or(pixels);
		// SAFETY: the context is current during initialization; decoded RGB storage is
		// width * height * 3 bytes. SRGB8_ALPHA8 decodes albedo before filtering; RGB8 stores normals.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Read an integer capability into Rust; C++ glGetIntegerv writes through an output pointer.
			let max_side = gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as u32;
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if width > max_side || height > max_side
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err("Moon texture exceeds the device texture-size limit".into());
			}
			// Select which texture unit subsequent binds affect; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Allocate a texture handle owned by this renderer; C++ glGenTextures.
			let texture = gl.create_texture()?;
			// Register ownership before any later upload operation can fail; C++ vector::push_back records the GLuint.
			self.textures.push(texture);
			// Bind or unbind an image at the active unit; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Set row-byte alignment for RGB upload, then restore egui's default; C++ glPixelStorei.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
			// Upload level zero with explicit color-space storage; C++ glTexImage2D receives the pixel pointer.
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				width as i32,
				height as i32,
				0,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				glow::PixelUnpackData::Slice(Some(upload)),
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as i32,
			);
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			// Build all lower-resolution levels from level zero; C++ glGenerateMipmap.
			gl.generate_mipmap(glow::TEXTURE_2D);
			// Keep the texture's default anisotropy ratio of one: textureGrad with anisotropy made dotted limb arcs on llvmpipe.
			// Trilinear mipmaps and MSAA remain enabled; C++ likewise leaves GL_TEXTURE_MAX_ANISOTROPY_EXT at its default.
			// Set integer sampling/wrapping state; C++ glTexParameteri.
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_WRAP_T,
				glow::CLAMP_TO_EDGE as i32,
			);
			// Set row-byte alignment for RGB upload, then restore egui's default; C++ glPixelStorei.
			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);
			// Bind or unbind an image at the active unit; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, None);
			// Retrieve and clear a GL error flag after upload; C++ glGetError.
			let error = gl.get_error();
			// Evaluate this guard before using unavailable state or invalid dimensions; C++ if has the same branching semantics.
			if error != glow::NO_ERROR
			{
				// Return or store the diagnostic instead of continuing with invalid state; C++ expected::unexpected carries the error.
				return Err(format!("Moon texture upload GL error: {error:#x}"));
			}
		}
		// Wrap a successful result without throwing; C++ expected stores its value.
		Ok(())
	}

	// Upload browser-decoded pixels directly, without a WASM RGB buffer; C++ DOM image upload.
	#[cfg(target_arch = "wasm32")]
	fn upload_browser_texture(
		&mut self,
		gl: &glow::Context,
		image: &web_sys::HtmlImageElement,
		srgb: bool,
	) -> Result<(), String>
	{
		// SAFETY: callback runs on the current GL context and the image's creation thread.
		unsafe {
			// Register ownership before uploading, so construction failure cleans it; C++ owned GLuint.
			let texture = gl.create_texture()?;
			// Register cleanup ownership before using the handle; C++ vector::push_back.
			self.textures.push(texture);
			// Select the upload unit explicitly; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Bind this replacement without modifying the visible texture pair; C++ glBindTexture.
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			// Normal maps are data, not browser color-corrected display images; C++ pixel-store policy.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				glow::NONE as i32,
			);
			// Keep the original top-row order, matching the native byte upload; C++ pixel-store policy.
			gl.pixel_store_i32(web_sys::WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
				0,
			);
			// Upload the browser-owned decoded source without copying through WASM; C++ DOM texture source.
			gl.tex_image_2d_with_html_image(
				glow::TEXTURE_2D,
				0,
				if srgb { glow::SRGB8_ALPHA8 } else { glow::RGB8 } as i32,
				if srgb { glow::RGBA } else { glow::RGB },
				glow::UNSIGNED_BYTE,
				image,
			);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR_MIPMAP_LINEAR as i32,
			);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as i32,
			);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			gl.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_WRAP_T,
				glow::CLAMP_TO_EDGE as i32,
			);
			gl.generate_mipmap(glow::TEXTURE_2D);
			// Restore the WebGL default expected by other eframe painters; C++ state restoration.
			gl.pixel_store_i32(
				web_sys::WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
				web_sys::WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
			);
			// Leave no upload binding behind; C++ glBindTexture with zero.
			gl.bind_texture(glow::TEXTURE_2D, None);
			// Convert driver failures into the normal recoverable loader error; C++ glGetError.
			let error = gl.get_error();
			if error != glow::NO_ERROR
			{
				return Err(format!("Browser texture upload GL error {error}"));
			}
		}
		Ok(())
	}

	// Draw one opaque sphere inside egui's already configured viewport; C++ issues the equivalent GL commands.
	fn paint(
		&self,
		gl: &glow::Context,
		view: math::View,
		rays: &[([f32; 3], [Ray; 2])],
		lighting: Option<Lighting>,
		surface_rays: bool,
	)
	{
		// Borrow the optional program handle or return from an already-destroyed renderer; C++ checks optional before dereferencing.
		let Some(program) = self.program
		// Handle the complementary case; C++ uses the same else branch.
		else
		{
			// Stop this frame's work without a value; C++ return exits a void function.
			return;
		};
		// SAFETY: CallbackFn runs on eframe's current GL context, after its viewport/scissor
		// setup. The app requests a 24-bit depth buffer on native and web. egui_glow 0.37
		// draws to the default framebuffer, disables framebuffer-sRGB, and restores UI state.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Enable the selected fixed-function test; C++ glEnable.
			gl.enable(glow::DEPTH_TEST);
			// Keep fragments closer than the stored depth; C++ glDepthFunc(GL_LESS).
			gl.depth_func(glow::LESS);
			// Allow surviving fragments to update depth; C++ glDepthMask(GL_TRUE).
			gl.depth_mask(true);
			// Set the farthest normalized depth as the clear value; C++ glClearDepth or WebGL clearDepth.
			gl.clear_depth_f32(1.0);
			// Reset depth for this callback's scissored viewport; C++ glClear(GL_DEPTH_BUFFER_BIT).
			gl.clear(glow::DEPTH_BUFFER_BIT);
			// Disable blending for opaque drawing, or restore depth/culling after it; C++ glDisable.
			gl.disable(glow::BLEND);
			// Enable the selected fixed-function test; C++ glEnable.
			gl.enable(glow::CULL_FACE);
			// Discard back-facing triangles; C++ glCullFace(GL_BACK).
			gl.cull_face(glow::BACK);
			// Treat counterclockwise vertex order as front-facing; C++ glFrontFace(GL_CCW).
			gl.front_face(glow::CCW);
			// Select linked shaders, or unbind them; C++ glUseProgram(handle or 0).
			gl.use_program(Some(program));
			// Supply harmless finite directions when ephemerides are absent, but no invented sunlight; C++ optional fallback.
			let light = lighting.unwrap_or(Lighting {
				sun: [0.0, 0.0, 1.0],
				earth: [0.0, 0.0, 1.0],
				ratio: 0.0,
				boosted: false,
				fill: false,
			});
			// Send both Moon-fixed light directions into the shared model transform; C++ uniform vector loop.
			for (name, direction) in [("u_sun", light.sun), ("u_earth", light.earth)]
			{
				gl.uniform_3_f32(
					gl.get_uniform_location(program, name).as_ref(),
					direction[0],
					direction[1],
					direction[2],
				);
			}
			// Keep physical illumination and artificial display aids separate; C++ scalar uniforms.
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_sun_scale").as_ref(),
				if lighting.is_some() { 1.0 } else { 0.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_earthshine").as_ref(),
				light.ratio * if light.boosted { 1000.0 } else { 1.0 },
			);
			gl.uniform_1_f32(
				gl.get_uniform_location(program, "u_far_fill").as_ref(),
				if light.fill { 1.0 } else { 0.0 },
			);
			// Always restore the textured material before drawing the Moon; C++ glUniform1i.
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 0);
			// Send a column-major projection without transposing; C++ glUniformMatrix4fv.
			gl.uniform_matrix_4_f32_slice(
				// Look up an active uniform, returning None if absent; C++ glGetUniformLocation returns -1.
				gl.get_uniform_location(program, "u_projection").as_ref(),
				false,
				&view.projection(),
			);
			// Share one column-major rigid model matrix with geometry, normals, and CPU marker projection; C++ glUniformMatrix3fv.
			gl.uniform_matrix_3_f32_slice(
				gl.get_uniform_location(program, "u_model").as_ref(),
				false,
				&view.model(),
			);
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for (unit, name) in ["u_albedo", "u_normal"].iter().enumerate()
			{
				// Select which texture unit subsequent binds affect; C++ glActiveTexture.
				gl.active_texture(glow::TEXTURE0 + unit as u32);
				// Bind or unbind an image at the active unit; C++ glBindTexture.
				gl.bind_texture(glow::TEXTURE_2D, Some(self.textures[unit]));
				// Tell a sampler which texture-unit index to use; C++ glUniform1i.
				gl.uniform_1_i32(gl.get_uniform_location(program, name).as_ref(), unit as i32);
			}
			// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
			gl.bind_vertex_array(self.vao);
			// Submit indexed triangles with 32-bit indices starting at byte zero; C++ glDrawElements.
			gl.draw_elements(glow::TRIANGLES, self.index_count, glow::UNSIGNED_INT, 0);
			// Smooth alpha coverage needs blending on native and WebGL; C++ glEnable/glBlendFunc.
			gl.enable(glow::BLEND);
			gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
			gl.depth_mask(false);
			if surface_rays
			{
				self.paint_surface_rays(gl, program, view, lighting);
			}
			// These are direction diagrams, not physical beams: retain blocked rays on the visible surface; C++ depth-state change.
			gl.disable(glow::DEPTH_TEST);
			// Draw real object-space direction lines against the sphere's existing depth buffer; C++ optional guard.
			for &(origin, bodies) in rays
			{
				// Never expose diagrams from the back of the Moon; C++ visibility guard.
				if view.project(origin, true).is_none()
				{
					continue;
				}
				// Reuse the exact model/projection uniforms, but skip all texture/photometry work; C++ material switch.
				gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 1);
				// Select the independent, position-only line layout; C++ glBindVertexArray.
				gl.bind_vertex_array(self.ray_vao);
				// Upload two endpoints per ray without modifying the sphere buffers; C++ glBindBuffer.
				gl.bind_buffer(glow::ARRAY_BUFFER, self.ray_buffer);
				// Preserve Sun/yellow then Earth/blue order from the app; C++ array zip iteration.
				for (body, color) in bodies
					.into_iter()
					.zip([[1.0, 0.9, 0.05], [0.15, 0.65, 1.0]])
				{
					// Direction rays are 0.6 lunar radii, not body distances; C++ endpoint vector arithmetic.
					let end: [f32; 3] =
						std::array::from_fn(|i| origin[i] + 0.6 / view.zoom * body.direction[i]);
					// Build screen-width triangle ribbons and a circular tip; C++ dynamic interleaved vertex array.
					let vertices = view.ribbon(origin, end);
					// Stream the ribbon and endpoint triangles without persistent mapping; C++ glBufferData.
					gl.buffer_data_u8_slice(
						glow::ARRAY_BUFFER,
						bytemuck::cast_slice(&vertices),
						glow::STREAM_DRAW,
					);
					// Dim failed terrain/radio predicates on the visible-side direction diagram; C++ conditional scalar.
					let strength = if body.visible { 1.0 } else { 0.35 };
					// Send an unlit display color rather than illuminating rays with the globe's illustration light; C++ glUniform3f.
					gl.uniform_3_f32(
						gl.get_uniform_location(program, "u_ray_color").as_ref(),
						color[0] * strength,
						color[1] * strength,
						color[2] * strength,
					);
					// Draw both ribbon faces and the endpoint fan as portable triangles; C++ glDrawArrays(GL_TRIANGLES).
					gl.disable(glow::CULL_FACE);
					gl.draw_arrays(glow::TRIANGLES, 0, (vertices.len() / 7) as i32);
				}
				// Do not leak a dynamic buffer binding into egui; C++ glBindBuffer(...,0).
				gl.bind_buffer(glow::ARRAY_BUFFER, None);
			}
			// Restore write/blend policy after transparent diagrams; C++ GL state restoration.
			gl.depth_mask(true);
			gl.disable(glow::BLEND);
			// Select vertex-layout state, or unbind with None; C++ glBindVertexArray(handle or 0).
			gl.bind_vertex_array(None);
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for unit in 0..2
			{
				// Select which texture unit subsequent binds affect; C++ glActiveTexture.
				gl.active_texture(glow::TEXTURE0 + unit);
				// Bind or unbind an image at the active unit; C++ glBindTexture.
				gl.bind_texture(glow::TEXTURE_2D, None);
			}
			// Select which texture unit subsequent binds affect; C++ glActiveTexture.
			gl.active_texture(glow::TEXTURE0);
			// Select linked shaders, or unbind them; C++ glUseProgram(handle or 0).
			gl.use_program(None);
			// Disable blending for opaque drawing, or restore depth/culling after it; C++ glDisable.
			gl.disable(glow::DEPTH_TEST);
			// Disable blending for opaque drawing, or restore depth/culling after it; C++ glDisable.
			gl.disable(glow::CULL_FACE);
		}
	}

	// Batch the entire Sun field into one cheap draw; C++ dynamic interleaved vertex vector.
	fn paint_surface_rays(
		&self,
		gl: &glow::Context,
		program: glow::Program,
		view: math::View,
		lighting: Option<Lighting>,
	)
	{
		let Some(light) = lighting
		else
		{
			return;
		};
		let mut vertices = Vec::new();
		for point in field::illuminated_points(light.sun)
		{
			let origin = std::array::from_fn(|i| point[i] + 0.6 / view.zoom * light.sun[i]);
			let mut ribbon = view.ribbon(origin, point);
			ribbon.truncate(42);
			vertices.extend(ribbon);
		}
		// SAFETY: the callback owns the current GL context; the ray layout matches seven floats per vertex.
		unsafe {
			gl.enable(glow::DEPTH_TEST);
			gl.uniform_1_i32(gl.get_uniform_location(program, "u_ray_mode").as_ref(), 1);
			gl.uniform_3_f32(
				gl.get_uniform_location(program, "u_ray_color").as_ref(),
				1.0,
				0.9,
				0.05,
			);
			gl.bind_vertex_array(self.ray_vao);
			gl.bind_buffer(glow::ARRAY_BUFFER, self.ray_buffer);
			gl.buffer_data_u8_slice(
				glow::ARRAY_BUFFER,
				bytemuck::cast_slice(&vertices),
				glow::STREAM_DRAW,
			);
			gl.disable(glow::CULL_FACE);
			gl.draw_arrays(glow::TRIANGLES, 0, (vertices.len() / 7) as i32);
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
	}

	// Release registered GPU objects using the current context; C++ explicit teardown avoids a late invalid-context destructor.
	fn destroy(&mut self, gl: &glow::Context)
	{
		// SAFETY: called with the owning context current, either on_exit or failed construction.
		// Taking/draining handles makes repeated cleanup harmless.
		// The caller must keep this GL context current; Rust cannot verify external driver state, just as C++ cannot.
		unsafe {
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for texture in self.textures.drain(..)
			{
				// Release GPU texture storage while the context is current; C++ glDeleteTextures.
				gl.delete_texture(texture);
			}
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for buffer in self.buffers.drain(..)
			{
				// Release vertex/index storage while the context is current; C++ glDeleteBuffers.
				gl.delete_buffer(buffer);
			}
			// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
			if let Some(vao) = self.vao.take()
			{
				// Release vertex-layout state while the context is current; C++ glDeleteVertexArrays.
				gl.delete_vertex_array(vao);
			}
			// Delete the ray layout once; its buffer is already owned by the shared buffer inventory; C++ optional extraction.
			if let Some(vao) = self.ray_vao.take()
			{
				// Release layout state while the context is current; C++ glDeleteVertexArrays.
				gl.delete_vertex_array(vao);
			}
			// Clear the nonowning alias after buffer destruction; C++ handle reset.
			self.ray_buffer = None;
			// Iterate the stated rows, columns, resources, or completions without ownership aliasing; C++ uses a range-for loop.
			for shader in self.shaders.drain(..)
			{
				// Delete the no-longer-needed shader object; C++ glDeleteShader.
				gl.delete_shader(shader);
			}
			// Enter only when the optional handle/value exists and bind it; C++ if(optional) then dereferences it.
			if let Some(program) = self.program.take()
			{
				// Release the linked executable while the context is current; C++ glDeleteProgram.
				gl.delete_program(program);
			}
		}
	}
}
