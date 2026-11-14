#[derive(Default)]
pub struct MoonApp;

impl eframe::App for MoonApp
{
	fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame)
	{
		eframe::egui::CentralPanel::default().show(ui, |ui| {
			ui.heading("Moon Landing Visual");
			ui.label("The panorama is coming.");
		});
	}
}
