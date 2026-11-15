// Compile the validated data loader; C++ includes a project module's declarations.
mod custom;
mod data;
// Compile the immediate-mode panorama view; C++ includes a project module's declarations.
mod panorama;
// Compile the angular sky-strip painter; C++ project drawing module.
mod sky;
// Keep grouped presentation settings outside scientific orchestration; C++ module declaration.
#[cfg(any(target_arch = "wasm32", test))]
mod fonts;
mod layout;
mod menu;
mod startup;
mod theme;
mod timeline;

// Deserialize the small metadata catalog only, never embed the per-site ephemerides; C++ aggregate schema.
#[derive(serde::Deserialize)]
struct Entry
{
	name: String,
	lat_deg: f64,
	lon_deg: f64,
	source: String,
}
// Derive the same ASCII filename convention used by prep; C++ string transform.
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

// Own persistent renderer and scientific-view state; C++ application class members.
pub struct MoonApp
{
	// Keep presentation independent of the scientific core; C++ renderer value member.
	globe: globe::Globe,
	// Embed only published metadata; C++ vector of catalog value records.
	catalog: Vec<Entry>,
	// Cache one optional asynchronous view model per stable catalog identifier; C++ vector<optional<View>>.
	panoramas: Vec<Option<panorama::Panorama>>,
	// Share the selected catalog identifier with globe picking; C++ array index.
	selected: usize,
	// Default to the selected site's pair rather than launching all requests; C++ bool option.
	all_rays: bool,
	// Preserve a UTC epoch independently of each view's sample index; C++ optional<int64_t>.
	time: Option<i64>,
	// Persist only whether the floating menu is open; C++ bool member.
	menu_open: bool,
	guide_open: bool,
	guide_dismissal_pending: bool,
	// Published catalog length; a custom point, if any, is the entry after them.
	published: usize,
	custom: Option<custom::Custom>,
	// Finished custom points other than the current one, for this session.
	cached: Vec<custom::Cached>,
	timeline: timeline::Shared,
}

// Define application member functions; C++ out-of-class member definitions.
impl MoonApp
{
	// Construct both views with the active GL context and event-loop handle; C++ expected-returning factory.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, String>
	{
		// Borrow the glow context or propagate a readable error; C++ checked optional reference.
		let gl = cc
			.gl
			.as_ref()
			.ok_or("The Moon requires the glow renderer")?;
		// Use readable light text on dark panels consistently across platforms; C++ UI theme setter.
		cc.egui_ctx.set_theme(eframe::egui::ThemePreference::Dark);
		#[cfg(target_arch = "wasm32")]
		fonts::install(&cc.egui_ctx);
		// Parse exact published metadata before initializing any lazy track loads; C++ checked JSON decode.
		let catalog: Vec<Entry> =
			serde_json::from_str(include_str!("../../sites.json")).map_err(|e| e.to_string())?;
		// Check this state or input before acting; C++ conditional branch.
		if catalog.is_empty()
		{
			return Err("Empty site catalog".into());
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let panoramas: Vec<_> = (0..catalog.len()).map(|_| None).collect();
		let published = catalog.len();
		let mut globe = globe::Globe::new(gl)?;
		let [sun, earth] =
			startup::directions(include_bytes!("../assets/data/moon-ephemeris.initial.bin"))?;
		globe.set_startup_light(sun, earth)?;
		// Move independently owned models into the app; C++ aggregate initialization.
		Ok(Self {
			// Propagate renderer initialization errors; C++ expected error forwarding.
			globe,
			// Start one asynchronous data request, not one per frame; C++ copy the shared UI context handle.
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
	// Start a job for a new globe click, advance the running job, and highlight its tile.
	fn update_custom(&mut self, ctx: &eframe::egui::Context)
	{
		self.globe.set_grid(|lat, lon| {
			let t = lunar_core::anypoint::tile_of(lat, lon);
			[t.lat_deg, t.lon_deg, t.half_lat_deg, t.half_lon_deg]
		});
		if let Some((lat, lon)) = self.globe.take_pick()
		{
			let tile = lunar_core::anypoint::tile_of(lat, lon);
			if self.custom.as_ref().is_none_or(|c| c.tile.id != tile.id || c.failed)
			{
				// Keep a finished current point; unfinished or failed ones are dropped.
				let old = self.panoramas.get_mut(self.published).and_then(Option::take);
				if let (Some(c), Some(mut view)) = (self.custom.take(), old)
					&& view.sun_pct().is_some()
				{
					self.cached.push(custom::Cached { tile: c.tile, view });
				}
				let timeline = std::sync::Arc::clone(&self.timeline);
				let (job, view) = match self.cached.iter().position(|c| c.tile.id == tile.id)
				{
					Some(i) =>
					{
						let c = self.cached.swap_remove(i);
						(custom::Custom::restored(c.tile, timeline), c.view)
					}
					None =>
					{
						let job = custom::Custom::start(ctx, lat, lon, std::sync::Arc::clone(&timeline));
						let view = panorama::Panorama::with_pending(job.pending.clone(), timeline);
						(job, view)
					}
				};
				let entry = Entry {
					name: job.name.clone(),
					lat_deg: job.tile.lat_deg,
					lon_deg: job.tile.lon_deg,
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
		if let Some(job) = &mut self.custom
		{
			job.step(ctx);
		}
		let tile = self
			.custom
			.as_ref()
			.filter(|_| self.selected >= self.published)
			.map(|c| {
				[
					c.tile.lat_deg,
					c.tile.lon_deg,
					c.tile.half_lat_deg,
					c.tile.half_lon_deg,
				]
			});
		self.globe.set_tile(tile);
		let corners = |t: &lunar_core::anypoint::Tile| [t.lat_deg, t.lon_deg, t.half_lat_deg, t.half_lon_deg];
		let current = self
			.custom
			.as_ref()
			.zip(self.panoramas.get_mut(self.published).and_then(Option::as_mut));
		let done = self
			.cached
			.iter_mut()
			.map(|c| (&c.tile, &mut c.view))
			.chain(current.map(|(c, v)| (&c.tile, v)))
			.filter_map(|(t, v)| Some((corners(t), v.sun_pct()?)))
			.collect();
		self.globe.set_tiles(done);
	}

	// Render the selected scientific view while preserving the shared epoch; C++ member function.
	fn show_panorama(&mut self, ui: &mut eframe::egui::Ui)
	{
		ui.horizontal_wrapped(|ui| {
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

// Check catalog/filename invariants without GPU initialization or ephemeris I/O; C++ test namespace.
#[cfg(test)]
mod tests
{
	use super::*;
	// Validate the real prepared ephemerides through the same public ENU conversion used by lighting; C++ integration test.
	#[test]
	fn moon_fixed_ephemeris_libration_ranges()
	{
		// Decode actual hourly data without adding JSON or I/O dependencies to the renderer; C++ test fixture.
		let data: serde_json::Value =
			serde_json::from_str(include_str!("../assets/data/connecting-ridge-c1.json")).unwrap();
		// Use the exact prepared observer position, not a coordinate synthesized to match a range; C++ fixture reads.
		let lat = data["site"]["lat_deg"].as_f64().unwrap();
		// Preserve the same east-longitude convention as production markers; C++ checked fixture access.
		let lon = data["site"]["lon_deg"].as_f64().unwrap();
		// Check both physical ranges and nonconstant evolution over the three-year series; C++ counted regression loop.
		for (body, latitude_limit, longitude_limit) in [("sun", 1.61, 180.0), ("earth", 7.4, 8.4)]
		{
			// Accumulate transformed latitudes for bounds and temporal-span checks; C++ vector<float>.
			let mut latitudes = Vec::new();
			// Accumulate longitudes independently so axis swaps cannot hide; C++ vector<float>.
			let mut longitudes = Vec::new();
			// Visit every aligned hourly pair, not merely one hand-picked date; C++ zipped range iteration.
			for (az, el) in data[body]["az_deg"]
				.as_array()
				.unwrap()
				.iter()
				.zip(data[body]["el_deg"].as_array().unwrap())
			{
				// The same public conversion feeds both real lights and the Earth camera; C++ optional unwrap in a test.
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
				// Recover latitude from the Moon-fixed north component; C++ asin after a roundoff clamp.
				latitudes.push(direction[1].clamp(-1.0, 1.0).asin().to_degrees());
				// Recover east longitude from the zero-longitude and east axes; C++ atan2.
				longitudes.push(direction[0].atan2(direction[2]).to_degrees());
			}
			// Latitude and longitude bounds catch swapped basis axes or treating local azimuth as longitude; C++ assertions.
			assert!(latitudes.iter().all(|x| x.abs() < latitude_limit));
			// Reject nonphysical lunar libration longitude or a malformed azimuth transform; C++ all_of assertion.
			assert!(longitudes.iter().all(|x| x.abs() <= longitude_limit));
			// Reject a constant placeholder with an otherwise plausible latitude; C++ max/min range assertion.
			assert!(
				latitudes.iter().copied().fold(f32::NEG_INFINITY, f32::max)
					- latitudes.iter().copied().fold(f32::INFINITY, f32::min)
					> if body == "sun" { 2.5 } else { 10.0 }
			);
			// Solar longitude must sweep around the Moon; Earth longitude must librate substantially; C++ range assertion.
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
		// Bind this intermediate value for the current operation; C++ local initialization.
		let catalog: Vec<Entry> = serde_json::from_str(include_str!("../../sites.json")).unwrap();
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(catalog.len(), 7);
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(catalog[0].name, "Connecting Ridge C1");
		// Bind this intermediate value for the current operation; C++ local initialization.
		let files: std::collections::HashSet<_> =
			catalog.iter().map(|s| filename(&s.name)).collect();
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(files.len(), catalog.len());
		// Enforce the independent regression expectation; C++ test assertion.
		assert!(catalog.iter().all(|s| s.source.contains("https://")
			&& (-90.0..=-85.0).contains(&s.lat_deg)
			&& s.lon_deg.is_finite()));
		// Enforce the independent regression expectation; C++ test assertion.
		assert_eq!(filename("Shackleton  S1"), "shackleton-s1.json");
	}
}

// Implement the framework's application trait; C++ override a virtual application interface.
impl eframe::App for MoonApp
{
	// Rebuild responsive UI each frame; C++ virtual draw callback.
	fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame)
	{
		self.update_custom(ui.ctx());
		self.update_sites(ui);
		self.synchronize_scene();
		// Draw the brand bar and floating menu before the central view; C++ responsive panel call.
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

	// Destroy GPU resources while the framework still has a current context; C++ virtual shutdown callback.
	fn on_exit(&mut self, gl: Option<&eframe::glow::Context>)
	{
		// Only issue GL commands when a context was supplied; C++ if(optional).
		if let Some(gl) = gl
		{
			// Explicit renderer teardown avoids invalid-context destruction; C++ member cleanup call.
			self.globe.destroy(gl);
		}
	}
}
