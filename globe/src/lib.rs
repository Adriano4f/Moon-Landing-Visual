//! An illustrative lunar renderer, independent of the landing-site scientific library.
//! See docs/03-moon.md for coordinate, photometric, color, and GL lifetime contracts.

mod assets;
mod camera;
mod deep;
mod phase;
mod polar;
mod renderer;
mod view;
use renderer::Renderer;
mod field;
mod math;
mod orientation;
mod pick;
mod space;
pub mod help;

/// Source pixels eventually overzoom; the camera remains above the illustrative sphere.
const MAX_ZOOM: f32 = 1024.0;
const MIN_ZOOM: f32 = 0.5;

use egui::mutex::Mutex;
use egui_glow::glow::{self, HasContext as _};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct BodyDirection
{
	/// Azimuth is measured clockwise from north, in degrees.
	pub az_deg: f32,
	/// Elevation is measured above the site's horizontal plane, in degrees.
	pub el_deg: f32,
	/// The app supplies its terrain/clearance availability predicate.
	pub visible: bool,
}

/// Convert a topocentric sky angle to the renderer's Moon-fixed unit vector.
pub fn moon_fixed(lat_deg: f64, lon_deg: f64, body: BodyDirection) -> Option<[f32; 3]>
{
	math::local_direction(
		(lat_deg as f32).to_radians(),
		(lon_deg as f32).to_radians(),
		body.az_deg.to_radians(),
		body.el_deg.to_radians(),
	)
}

#[derive(Clone, Copy)]
struct Lighting
{
	sun: [f32; 3],
	earth: [f32; 3],
	/// Express visible reflected irradiance relative to sunlight.
	ratio: f32,
	/// Keep the optional Earth-only exposure multiplication explicit.
	boosted: bool,
	fill: bool,
}

#[derive(Clone, Copy)]
struct Ray
{
	/// Moon-fixed unit direction before presentation rotation.
	direction: [f32; 3],
	/// False selects the dim material, independently of depth occlusion.
	visible: bool,
}

#[derive(Clone)]
pub struct Marker
{
	pub id: usize,
	pub name: String,
	pub lat_deg: f64,
	pub lon_deg: f64,
	pub directions: Option<[BodyDirection; 2]>,
}

/// Owns the callback's shared renderer. Call destroy from eframe's on_exit with a current context.
pub struct Globe
{
	renderer: Arc<Mutex<Option<Renderer>>>,
	loading: Arc<Mutex<assets::Loading>>,
	polar: polar::Shared,
	deep: deep::Shared,
	/// Selected size label 2/4/8, or zero for the private 512-pixel browser preview.
	tier: u32,
	target_tier: u32,
	max_tier: u32,
	surface_rays: bool,
	scene: Vec<Marker>,
	selected: usize,
	startup_light: Option<[[f32; 3]; 2]>,
	last_light: Option<[[f32; 3]; 2]>,
	yaw: f32,
	epoch: i64,
	body_rotation: math::Mat3,
	tilt: f32,
	zoom: f32,
	pan: [f32; 2],
	paused: bool,
	/// Track the current ephemeris Earth line of sight instead of spinning.
	earth_view: bool,
	space_view: bool,
	/// Multiply only Earthshine for display when explicitly requested.
	boosted: bool,
	/// Add an explicitly artificial far-side fill only on request.
	fill: bool,
	last_time: Option<f64>,
	focus: Option<(f64, [f32; 3], [f32; 3])>,
	pick: Option<(f64, f64)>,
	tile: Option<[f64; 4]>,
	tiles: Vec<([f64; 4], f32)>,
	grid: Option<fn(f64, f64) -> [f64; 4]>,
}

#[cfg(test)]
mod scene_tests
{
	use super::*;

	pub(super) fn unloaded() -> Globe
	{
		Globe {
			renderer: Arc::new(Mutex::new(None)),
			loading: Arc::new(Mutex::new(assets::Loading {
				images: [None, None],
				error: None,
			})),
			polar: Arc::new(Mutex::new(polar::State::default())),
			deep: Arc::new(Mutex::new(deep::State::default())),
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
			pan: [0.0; 2],
			paused: true,
			earth_view: true,
			space_view: false,
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

	#[test]
	fn scene_identity_empty_and_invalid()
	{
		let mut globe = unloaded();
		let valid = Marker {
			id: 17,
			name: "Published".into(),
			lat_deg: -89.0,
			lon_deg: 220.0,
			directions: None,
		};
		// Represent malformed external metadata without corrupting the valid entry.
		let invalid = Marker {
			id: 8,
			lat_deg: f64::NAN,
			..valid.clone()
		};
		globe.set_scene(vec![invalid, valid], 17);
		// Filtering must not renumber the published marker identifier.
		assert_eq!(
			globe.scene.iter().map(|s| s.id).collect::<Vec<_>>(),
			vec![17]
		);
		assert!(globe.focus.is_none());
		// Empty data clears markers and rays instead of retaining old state.
		globe.set_scene(vec![], 17);
		assert!(globe.scene.is_empty());
	}
}

impl Globe
{
	pub fn new(gl: &Arc<glow::Context>) -> Result<Self, String>
	{
		let (mobile, forced) = assets::preferences();
		// SAFETY: eframe makes this context current before calling the constructor.
		let tier = assets::select(
			unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
			mobile,
			forced.as_deref(),
		)?;
		// Format a visible diagnostic naming the chosen tier.
		let status = format!("Moon tier {tier}k (mobile={mobile})");
		#[cfg(not(target_arch = "wasm32"))]
		eprintln!("{status}");
		#[cfg(target_arch = "wasm32")]
		web_sys::console::log_1(&status.into());
		Ok(Self {
			renderer: Arc::new(Mutex::new(None)),
			surface_rays: false,
			loading: assets::load(assets::first_tier(), None),
			polar: Arc::new(Mutex::new(polar::State::default())),
			deep: Arc::new(Mutex::new(deep::State::default())),
			tier: assets::first_tier(),
			target_tier: tier,
			max_tier: assets::select(
				unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) },
				false,
				Some("8"),
			)?,
			// Default to the current Earth direction without artificial spin.
			scene: Vec::new(),
			selected: 0,
			startup_light: None,
			last_light: None,
			yaw: 0.0,
			epoch: orientation::INITIAL_UTC,
			body_rotation: orientation::at_utc(orientation::INITIAL_UTC),
			tilt: 0.0,
			zoom: 1.0,
			pan: [0.0; 2],
			paused: true,
			earth_view: true,
			space_view: false,
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

	/// Copy metadata for every marker and scientific directions only for requested sites.
	pub fn set_scene(&mut self, scene: Vec<Marker>, selected: usize)
	{
		if self.selected != selected
		{
			self.focus = None;
		}
		self.scene = scene
			.into_iter()
			.filter(|m| {
				m.lat_deg.is_finite()
					&& m.lon_deg.is_finite()
					&& (-90.0..=90.0).contains(&m.lat_deg)
			})
			.collect();
		self.selected = selected;
	}

	/// Release registered GPU objects using the current context.
	pub fn destroy(&mut self, gl: &glow::Context)
	{
		if let Some(mut renderer) = self.renderer.lock().take()
		{
			renderer.destroy(gl);
		}
	}
}
