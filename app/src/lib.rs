//! Native and browser UI for lunar site comparison, terrain visibility and timeline controls.

mod custom;
mod data;
mod panorama;
mod sky;
#[cfg(any(target_arch = "wasm32", test))]
mod fonts;
mod layout;
mod menu;
mod startup;
mod theme;
mod timeline;

/// Deserialize the small metadata catalog only, never embed the per-site ephemerides.
#[derive(serde::Deserialize)]
struct Entry
{
	name: String,
	lat_deg: f64,
	lon_deg: f64,
	source: String,
}
fn filename(name: &str) -> String
{
	format!(
		"{}.json",
		name.split(|c: char| !c.is_ascii_alphanumeric())
			.filter(|s| !s.is_empty())
			.map(str::to_ascii_lowercase)
			.collect::<Vec<_>>()
			.join("-")
	)
}

pub struct MoonApp
{
	globe: globe::Globe,
	/// Embed only published metadata.
	catalog: Vec<Entry>,
	panoramas: Vec<Option<panorama::Panorama>>,
	selected: usize,
	/// Default to the selected site's pair rather than launching all requests.
	all_rays: bool,
	/// Preserve a UTC epoch independently of each view's sample index.
	time: Option<i64>,
	menu_open: bool,
	guide_open: bool,
	guide_dismissal_pending: bool,
	published: usize,
	custom: Option<custom::Custom>,
	cached: Vec<custom::Cached>,
	timeline: timeline::Shared,
}

impl MoonApp
{
	/// Construct both views with the active GL context and event-loop handle.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, String>
	{
		let gl = cc
			.gl
			.as_ref()
			.ok_or("The Moon requires the glow renderer")?;
		cc.egui_ctx.set_theme(eframe::egui::ThemePreference::Dark);
		#[cfg(target_arch = "wasm32")]
		fonts::install(&cc.egui_ctx);
		let catalog: Vec<Entry> =
			serde_json::from_str(include_str!("../../sites.json")).map_err(|e| e.to_string())?;
		if catalog.is_empty()
		{
			return Err("Empty site catalog".into());
		}
		let panoramas: Vec<_> = (0..catalog.len()).map(|_| None).collect();
		let published = catalog.len();
		let mut globe = globe::Globe::new(gl)?;
		let [sun, earth] =
			startup::directions(include_bytes!("../assets/data/moon-ephemeris.initial.bin"))?;
		globe.set_startup_light(sun, earth)?;
		Ok(Self {
			globe,
			// Start one asynchronous data request, not one per frame.
			catalog,
			panoramas,
			selected: 0,
			all_rays: false,
			time: None,
			menu_open: false,
			guide_open: menu::guide_visible(cc.storage),
			guide_dismissal_pending: false,
			published,
			custom: None,
			cached: Vec::new(),
			timeline: std::sync::Arc::new(eframe::egui::mutex::Mutex::new(timeline::Timeline::default())),
		})
	}
	fn update_custom(&mut self, ctx: &eframe::egui::Context)
	{
		if let Some((lat, lon)) = self.globe.take_pick()
		{
			self.open_custom(ctx, [lat, lon]);
		}
		if let Some(job) = &mut self.custom
		{
			job.step(ctx);
		}
		let current = self
			.custom
			.as_ref()
			.zip(self.panoramas.get_mut(self.published).and_then(Option::as_mut));
		let done = self
			.cached
			.iter_mut()
			.map(|c| (c.point, &mut c.view))
			.chain(current.map(|(c, v)| (c.point, v)))
			.filter_map(|(point, v)| Some((point, v.sun_pct()?)))
			.collect();
		self.globe.set_points(done);
	}

	fn select_marker(&mut self, ctx: &eframe::egui::Context, id: usize)
	{
		self.selected = id;
		if id == self.published && let Some(c) = self.custom.as_ref().filter(|c| c.failed)
		{
			self.open_custom(ctx, c.point);
		}
	}

	fn open_custom(&mut self, ctx: &eframe::egui::Context, point: [f64; 2])
	{
		if self.custom.as_ref().is_none_or(|c| !c.can_reuse(point))
		{
			let old = self.panoramas.get_mut(self.published).and_then(Option::take);
			if let (Some(c), Some(mut view)) = (self.custom.take(), old)
				&& view.sun_pct().is_some()
			{
				self.cached.push(custom::Cached { point: c.point, view });
			}
			let timeline = std::sync::Arc::clone(&self.timeline);
			let (job, view) = match self.cached.iter().position(|c| c.point == point)
			{
				Some(i) =>
				{
					let c = self.cached.swap_remove(i);
					(custom::Custom::restored(c.point, timeline), c.view)
				}
				None =>
				{
					let job = custom::Custom::start(
						ctx, point[0], point[1], std::sync::Arc::clone(&timeline),
					);
					let view = panorama::Panorama::with_pending(job.pending.clone(), timeline);
					(job, view)
				}
			};
			let entry = Entry {
				name: job.name.clone(),
				lat_deg: job.point[0],
				lon_deg: job.point[1],
				source: String::new(),
			};
			self.catalog.truncate(self.published);
			self.panoramas.truncate(self.published);
			self.catalog.push(entry);
			self.panoramas.push(Some(view));
			self.custom = Some(job);
		}
		self.selected = self.published;
	}

	fn show_panorama(&mut self, ui: &mut eframe::egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
			ui.strong("View");
			if ui.button("Change options").clicked()
			{
				self.menu_open = true;
			}
			self.globe.view_controls(ui);
		});
		if self.selected >= self.published
			&& let Some(status) = self.custom.as_ref().and_then(|c| c.status())
		{
			ui.spinner();
			globe::help::annotated(
				ui,
				&status,
				"Hills around your point are computed from NASA LOLA elevation maps.",
			);
		}
		if self.time.is_none()
		{
			globe::help::annotated(
				ui,
				"Sky preview: 2027-01-01 00:00 UTC",
				"Real Sun and Earth directions shown while the site's hills load.",
			);
		}
		if let Some(p) = &mut self.panoramas[self.selected]
		{
			if let Some(time) = self.time
			{
				p.set_time(time);
			}
			p.show(ui);
			if let Some(time) = p.time()
			{
				self.time = Some(time);
			}
		}
		else
		{
			globe::help::annotated(
				ui,
				"Loading selected site...",
				"The selected site's hourly Sun and Earth data is downloading.",
			);
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	/// Validate the real prepared ephemerides through the same public ENU conversion used by lighting.
	#[test]
	fn moon_fixed_ephemeris_libration_ranges()
	{
		// Decode actual hourly data without adding JSON or I/O dependencies to the renderer.
		let data: serde_json::Value =
			serde_json::from_str(include_str!("../assets/data/connecting-ridge-c1.json")).unwrap();
		// Use the exact prepared observer position, not a coordinate synthesized to match a range.
		let lat = data["site"]["lat_deg"].as_f64().unwrap();
		// Preserve the same east-longitude convention as production markers.
		let lon = data["site"]["lon_deg"].as_f64().unwrap();
		for (body, latitude_limit, longitude_limit) in [("sun", 1.61, 180.0), ("earth", 7.4, 8.4)]
		{
			let mut latitudes = Vec::new();
			let mut longitudes = Vec::new();
			// Visit every aligned hourly pair, not merely one hand-picked date.
			for (az, el) in data[body]["az_deg"]
				.as_array()
				.unwrap()
				.iter()
				.zip(data[body]["el_deg"].as_array().unwrap())
			{
				let direction = globe::moon_fixed(
					lat,
					lon,
					globe::BodyDirection {
						az_deg: az.as_f64().unwrap() as f32,
						el_deg: el.as_f64().unwrap() as f32,
						visible: false,
					},
				)
				.unwrap();
				// Recover latitude from the Moon-fixed north component.
				latitudes.push(direction[1].clamp(-1.0, 1.0).asin().to_degrees());
				// Recover east longitude from the zero-longitude and east axes.
				longitudes.push(direction[0].atan2(direction[2]).to_degrees());
			}
			// Latitude and longitude bounds catch swapped basis axes or treating local azimuth as longitude.
			assert!(latitudes.iter().all(|x| x.abs() < latitude_limit));
			// Reject nonphysical lunar libration longitude or a malformed azimuth transform.
			assert!(longitudes.iter().all(|x| x.abs() <= longitude_limit));
			// Reject a constant placeholder with an otherwise plausible latitude.
			assert!(
				latitudes.iter().copied().fold(f32::NEG_INFINITY, f32::max)
					- latitudes.iter().copied().fold(f32::INFINITY, f32::min)
					> if body == "sun" { 2.5 } else { 10.0 }
			);
			// Solar longitude must sweep around the Moon; Earth longitude must librate substantially.
			assert!(
				longitudes.iter().copied().fold(f32::NEG_INFINITY, f32::max)
					- longitudes.iter().copied().fold(f32::INFINITY, f32::min)
					> if body == "sun" { 350.0 } else { 10.0 }
			);
		}
	}
	#[test]
	fn catalog_and_lazy_filenames()
	{
		let catalog: Vec<Entry> = serde_json::from_str(include_str!("../../sites.json")).unwrap();
		assert_eq!(catalog.len(), 7);
		assert_eq!(catalog[0].name, "Connecting Ridge C1");
		let files: std::collections::HashSet<_> =
			catalog.iter().map(|s| filename(&s.name)).collect();
		assert_eq!(files.len(), catalog.len());
		assert!(catalog.iter().all(|s| s.source.contains("https://")
			&& (-90.0..=-85.0).contains(&s.lat_deg)
			&& s.lon_deg.is_finite()));
		assert_eq!(filename("Shackleton  S1"), "shackleton-s1.json");
	}
}

impl eframe::App for MoonApp
{
	fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame)
	{
		self.update_custom(ui.ctx());
		self.update_sites(ui);
		self.synchronize_scene();
		let phone = ui.ctx().content_rect().width() < 760.0;
		theme::apply(ui.ctx(), phone);
		ui.set_style(ui.ctx().style_of(eframe::egui::Theme::Dark));
		self.settings(ui, phone);
		self.central_view(ui);
		self.welcome(ui.ctx());
		if std::mem::take(&mut self.guide_dismissal_pending)
			&& let Some(storage) = frame.storage_mut()
		{
			menu::dismiss_guide(storage);
		}
	}

	fn on_exit(&mut self, gl: Option<&eframe::glow::Context>)
	{
		if let Some(gl) = gl
		{
			// Explicit renderer teardown avoids invalid-context destruction.
			self.globe.destroy(gl);
		}
	}
}
