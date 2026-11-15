//! An illustrative lunar renderer, independent of the landing-site scientific library.
//! See docs/03-moon.md for coordinate, photometric, color, and GL lifetime contracts.

// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod assets;
mod camera;
mod phase;
mod polar;
mod renderer;
mod view;
use renderer::Renderer;
mod field;
// Compile a sibling Rust source module; C++ usually separates declarations and translation units.
mod math;
mod orientation;
mod pick;
// Share accessible explanatory widgets without introducing an app dependency; C++ public utility namespace.
pub mod help;

// Bound enlargement by the available polar texture detail, not GPU capacity; C++ constexpr float.
const MAX_ZOOM: f32 = 8.0;
const MIN_ZOOM: f32 = 0.5;

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
	polar: polar::Shared,
	// Selected size label 2/4/8, or zero for the private 512-pixel browser preview; C++ uint32_t.
	tier: u32,
	// Requested final detail is independent of the first visible preview; C++ unsigned members.
	target_tier: u32,
	max_tier: u32,
	// Optional illustrative parallel Sun rays; C++ bool.
	surface_rays: bool,
	// Interactive presentation state is independent of ephemeris time; C++ value members.
	scene: Vec<Marker>,
	selected: usize,
	// A caller-supplied real initial epoch, cleared on the first complete selected track; C++ optional pair.
	startup_light: Option<[[f32; 3]; 2]>,
	// Last real light, kept while the selected marker's track is still loading.
	last_light: Option<[[f32; 3]; 2]>,
	yaw: f32,
	epoch: i64,
	body_rotation: math::Mat3,
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
	// Last surface click off any marker, and the highlighted tile (lat, lon, half lat, half lon).
	pick: Option<(f64, f64)>,
	tile: Option<[f64; 4]>,
	// Finished tiles and their Sun percentage, plus the app's tile grid.
	tiles: Vec<([f64; 4], f32)>,
	grid: Option<fn(f64, f64) -> [f64; 4]>,
}

// Exercise scene validation without opening an OpenGL context; C++ isolated test namespace.
#[cfg(test)]
mod scene_tests
{
	// Reuse the production state and aggregate definitions; C++ qualified test access.
	use super::*;

	fn unloaded() -> Globe
	{
		// Supply uninitialized GPU state so no renderer or network is needed; C++ aggregate test fixture.
		Globe {
			renderer: Arc::new(Mutex::new(None)),
			loading: Arc::new(Mutex::new(assets::Loading {
				images: [None, None],
				error: None,
			})),
			polar: Arc::new(Mutex::new(polar::State::default())),
			tier: 2,
			target_tier: 2,
			max_tier: 8,
			surface_rays: false,
			scene: vec![],
			selected: 0,
			startup_light: None,
			last_light: None,
			yaw: 0.0,
			epoch: orientation::INITIAL_UTC,
			body_rotation: orientation::at_utc(orientation::INITIAL_UTC),
			tilt: 0.0,
			zoom: 1.0,
			paused: true,
			earth_view: true,
			boosted: false,
			fill: false,
			last_time: None,
			focus: Some((0.0, [0.0; 3], [1.0; 3])),
			pick: None,
			tile: None,
			tiles: Vec::new(),
			grid: None,
		}
	}

	#[test]
	fn initial_light_is_validated_and_never_reused_after_real_tracks()
	{
		let mut globe = unloaded();
		assert!(!globe.has_visible_moon());
		assert!(globe.light_sources().is_none());
		for bad in [[0.0; 3], [f32::NAN, 0.0, 1.0], [f32::INFINITY, 0.0, 1.0]]
		{
			assert!(globe.set_startup_light(bad, [0.0, 0.0, 1.0]).is_err());
			assert!(globe.light_sources().is_none());
		}
		globe
			.set_startup_light([0.0, 0.0, 2.0], [3.0, 0.0, 0.0])
			.unwrap();
		assert_eq!(
			globe.light_sources(),
			Some([[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]])
		);
		let direction = BodyDirection {
			az_deg: 0.0,
			el_deg: 90.0,
			visible: false,
		};
		globe.set_scene(
			vec![Marker {
				id: 0,
				name: "Real track".into(),
				lat_deg: 0.0,
				lon_deg: 0.0,
				directions: Some([direction; 2]),
			}],
			0,
		);
		assert!(globe.light_sources().is_some());
		assert!(globe.startup_light.is_none());
		globe.set_scene(vec![], 0);
		assert!(globe.light_sources().is_none());
	}

	#[test]
	fn failed_initial_images_do_not_claim_a_draw()
	{
		let globe = unloaded();
		assert!(!globe.has_load_error());
		assert!(!globe.has_visible_moon());
		globe.loading.lock().error = Some("Unavailable initial images".into());
		assert!(globe.has_load_error());
		assert!(!globe.has_visible_moon());
	}

	// Verify invalid entries cannot shift stable identifiers or retain a stale transition; C++ test registration.
	#[test]
	fn scene_identity_empty_and_invalid()
	{
		let mut globe = unloaded();
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
			loading: assets::load(assets::first_tier(), None),
			polar: Arc::new(Mutex::new(polar::State::default())),
			tier: assets::first_tier(),
			target_tier: tier,
			max_tier: assets::select(
				unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
				false,
				Some("8"),
			)?,
			// Default to a fixed inertial camera without artificial spin; C++ scalar initialization.
			scene: Vec::new(),
			selected: 0,
			startup_light: None,
			last_light: None,
			yaw: 0.0,
			epoch: orientation::INITIAL_UTC,
			body_rotation: orientation::at_utc(orientation::INITIAL_UTC),
			tilt: 0.0,
			zoom: 1.0,
			paused: true,
			earth_view: false,
			boosted: false,
			fill: false,
			last_time: None,
			focus: None,
			pick: None,
			tile: None,
			tiles: Vec::new(),
			grid: None,
		})
	}

	/// Set the scientific UTC epoch used for the display's lunar orientation.
	pub fn set_time(&mut self, utc: i64)
	{
		if self.epoch != utc {
			self.epoch=utc;
			self.body_rotation=orientation::at_utc(utc);
		}
	}

	/// Supply real Moon-fixed directions while the first scientific track is pending.
	pub fn set_startup_light(&mut self, sun: [f32; 3], earth: [f32; 3]) -> Result<(), String>
	{
		// Reject invalid vectors before changing the previous valid state; C++ checked normalized copy.
		let mut pair = [sun, earth];
		for direction in &mut pair
		{
			let norm = direction.iter().map(|v| v * v).sum::<f32>().sqrt();
			if !norm.is_finite() || norm < 1e-6
			{
				return Err("Invalid initial lighting direction".into());
			}
			*direction = direction.map(|v| v / norm);
		}
		self.startup_light = Some(pair);
		Ok(())
	}

	/// Report the first drawn, physically lit Moon rather than a decoded image or empty callback.
	pub fn has_visible_moon(&self) -> bool
	{
		self.renderer
			.lock()
			.as_ref()
			.is_some_and(|renderer| renderer.visible_announced)
	}

	/// Expose a failed visual load without preventing independent scientific data loading.
	pub fn has_load_error(&self) -> bool
	{
		self.loading.lock().error.is_some()
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
