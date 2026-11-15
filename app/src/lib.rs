// Compile the validated data loader; C++ includes a project module's declarations.
mod data;
// Compile the immediate-mode panorama view; C++ includes a project module's declarations.
mod panorama;
// Compile the angular sky-strip painter; C++ project drawing module.
mod sky;
// Keep grouped presentation settings outside scientific orchestration; C++ module declaration.
mod menu;

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
	// Persist only whether the phone drawer is open; C++ bool member.
	menu_open: bool,
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
		// Parse exact published metadata before initializing any lazy track loads; C++ checked JSON decode.
		let catalog: Vec<Entry> =
			serde_json::from_str(include_str!("../../sites.json")).map_err(|e| e.to_string())?;
		// Check this state or input before acting; C++ conditional branch.
		if catalog.is_empty()
		{
			return Err("Empty site catalog".into());
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let mut panoramas: Vec<_> = (0..catalog.len()).map(|_| None).collect();
		panoramas[0] = Some(panorama::Panorama::new(
			cc.egui_ctx.clone(),
			data::FILE.into(),
		));
		// Move independently owned models into the app; C++ aggregate initialization.
		Ok(Self {
			// Propagate renderer initialization errors; C++ expected error forwarding.
			globe: globe::Globe::new(gl)?,
			// Start one asynchronous data request, not one per frame; C++ copy the shared UI context handle.
			catalog,
			panoramas,
			selected: 0,
			all_rays: false,
			time: None,
			menu_open: false,
		})
	}
	// Render the selected scientific view while preserving the shared epoch; C++ member function.
	fn show_panorama(&mut self, ui: &mut eframe::egui::Ui)
	{
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
				"Wait for the selected site's validated hourly data. No results are invented while it loads.",
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
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		// Load only the selected site, or the remaining sites when all-rays is explicitly requested; C++ lazy cache initialization.
		for i in 0..self.catalog.len()
		{
			if (i == self.selected || self.all_rays) && self.panoramas[i].is_none()
			{
				self.panoramas[i] = Some(panorama::Panorama::new(
					ui.ctx().clone(),
					filename(&self.catalog[i].name),
				));
			}
			// Check this state or input before acting; C++ conditional branch.
			if let Some(p) = &mut self.panoramas[i]
			{
				p.update();
			}
		}
		// Keep one UTC epoch across switches and all-site rays; C++ optional epoch restoration.
		if let (Some(t), Some(p)) = (self.time, &mut self.panoramas[self.selected])
		{
			p.set_time(t);
		}
		// Bind this intermediate value for the current operation; C++ local initialization.
		let margin = self.panoramas[self.selected]
			.as_ref()
			.map_or(0.5, |p| p.margin());
		// Bind this intermediate value for the current operation; C++ local initialization.
		let epoch = self.time.or_else(|| {
			self.panoramas[self.selected]
				.as_ref()
				.and_then(|p| p.time())
		});
		// Bind this intermediate value for the current operation; C++ local initialization.
		let scene = self
			.catalog
			.iter()
			.enumerate()
			.map(|(i, s)| {
				// Bind this intermediate value for the current operation; C++ local initialization.
				let directions = if i == self.selected || self.all_rays
				{
					self.panoramas[i]
						.as_mut()
						.and_then(|p| epoch.and_then(|t| p.directions_at(t, margin)))
				}
				else
				{
					None
				};
				globe::Marker {
					id: i,
					name: s.name.clone(),
					lat_deg: s.lat_deg,
					lon_deg: s.lon_deg,
					directions,
				}
			})
			.collect();
		self.globe.set_scene(scene, self.selected);
		// Draw settings before the central view so desktop width and phone drawer are independent; C++ responsive panel call.
		let phone = ui.ctx().content_rect().width() < 760.0;
		self.settings(ui, phone);
		// Preserve black behind the Moon in both layouts; C++ immediate-mode central container.
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK))
			.show(ui, |ui| {
				// Capture finite viewport dimensions before entering the unbounded scroll axis; C++ copy a size vector.
				let available = ui.available_size();
				// Scroll only if a short screen cannot fit both views; C++ immediate-mode scroll container.
				eframe::egui::ScrollArea::vertical().show(ui, |ui| {
					// Stack on phones rather than crushing a 360-degree plot; C++ viewport-width branch.
					if available.x < 700.0
					{
						// Reserve room for phone camera and lighting controls; C++ clamp(height*0.32,220,290).
						let size = eframe::egui::vec2(
							available.x,
							(available.y * 0.32).clamp(220.0, 290.0),
						);
						// Constrain the globe callback to the top rectangle; C++ child container with explicit size.
						ui.allocate_ui(size, |ui| {
							// Check this state or input before acting; C++ conditional branch.
							if let Some(id) = self.globe.show(ui)
							{
								// Update this persistent member explicitly; C++ member assignment.
								self.selected = id;
							}
						});
						// Draw the scientific view below it; C++ member draw call.
						ui.allocate_ui(
							eframe::egui::vec2(
								available.x,
								(available.y - size.y - 8.0).max(460.0),
							),
							|ui| self.show_panorama(ui),
						);
					}
					// Use equal-width columns on larger windows; C++ else with two child containers.
					else
					{
						// Rebuild columns without retaining widget handles; C++ immediate-mode callback.
						ui.allocate_ui(available, |ui| {
							// Divide only the bounded native/desktop viewport, not the unbounded scroll axis; C++ column container.
							ui.columns(2, |columns| {
								// Keep the rotating Moon on black at left; C++ first child draw.
								if let Some(id) = self.globe.show(&mut columns[0])
								{
									// Update this persistent member explicitly; C++ member assignment.
									self.selected = id;
								}
								// Disable columns' cross-axis justification so the plot can reserve tick-label space; C++ child layout override.
								columns[1].with_layout(
									eframe::egui::Layout::top_down(eframe::egui::Align::Min),
									|ui| self.show_panorama(ui),
								);
							})
						});
					}
				});
			});
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
