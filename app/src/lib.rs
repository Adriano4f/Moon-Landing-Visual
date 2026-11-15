pub struct MoonApp
{
	globe: globe::Globe,
}

impl MoonApp
{
	pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, String>
	{
		let gl = cc
			.gl
			.as_ref()
			.ok_or("The Moon requires the glow renderer")?;
		Ok(Self {
			globe: globe::Globe::new(gl)?,
		})
	}
}

impl eframe::App for MoonApp
{
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		eframe::egui::CentralPanel::default()
			.frame(eframe::egui::Frame::NONE.fill(eframe::egui::Color32::BLACK))
			.show(ui, |ui| self.globe.show(ui));
	}

	fn on_exit(&mut self, gl: Option<&eframe::glow::Context>)
	{
		if let Some(gl) = gl
		{
			self.globe.destroy(gl);
		}
	}
}
