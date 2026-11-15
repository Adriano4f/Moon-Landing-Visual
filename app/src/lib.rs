// Compile the validated data loader; C++ includes a project module's declarations.
mod data;
// Compile the immediate-mode panorama view; C++ includes a project module's declarations.
mod panorama;

// Own persistent renderer and scientific-view state; C++ application class members.
pub struct MoonApp
{
	// Keep presentation independent of the scientific core; C++ renderer value member.
	globe: globe::Globe,
	// Own data, selected sample and plot window; C++ view-model value member.
	panorama: panorama::Panorama,
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
		// Move independently owned models into the app; C++ aggregate initialization.
		Ok(Self {
			// Propagate renderer initialization errors; C++ expected error forwarding.
			globe: globe::Globe::new(gl)?,
			// Start one asynchronous data request, not one per frame; C++ copy the shared UI context handle.
			panorama: panorama::Panorama::new(cc.egui_ctx.clone()),
		})
	}
}

// Implement the framework's application trait; C++ override a virtual application interface.
impl eframe::App for MoonApp
{
	// Rebuild responsive UI each frame; C++ virtual draw callback.
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		// Take completed data before deriving the globe marker; C++ poll an optional completion.
		self.panorama.update();
		// Share only geographic coordinates with the renderer; C++ optional<pair<double,double>> setter.
		self.globe.set_site(self.panorama.site());
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
						// Reserve a compact globe so phone controls and both axes remain visible; C++ clamp(height*0.24,140,220).
						let size = eframe::egui::vec2(
							available.x,
							(available.y * 0.24).clamp(140.0, 220.0),
						);
						// Constrain the globe callback to the top rectangle; C++ child container with explicit size.
						ui.allocate_ui(size, |ui| self.globe.show(ui));
						// Draw the scientific view below it; C++ member draw call.
						ui.allocate_ui(
							eframe::egui::vec2(
								available.x,
								(available.y - size.y - 8.0).max(460.0),
							),
							|ui| self.panorama.show(ui),
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
								self.globe.show(&mut columns[0]);
								// Disable columns' cross-axis justification so the plot can reserve tick-label space; C++ child layout override.
								columns[1].with_layout(
									eframe::egui::Layout::top_down(eframe::egui::Align::Min),
									|ui| self.panorama.show(ui),
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
