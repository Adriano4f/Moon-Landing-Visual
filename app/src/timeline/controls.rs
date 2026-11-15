use super::*;

impl Timeline
{
	pub fn controls(&mut self, ui: &mut egui::Ui, cursor: &mut i64)
	{
		ui.strong(crate::panorama::utc_label(*cursor));
		let mut seconds = *cursor as f64;
		ui.spacing_mut().slider_width = (ui.available_width() - 110.0).max(80.0);
		ui.horizontal(|ui| {
			ui.add(egui::Slider::new(&mut seconds,self.options.period.start as f64..=self.options.period.end as f64)
				.show_value(false).step_by(1.0).text("Time"));
			globe::help::question(ui,"Slide through the selected period. Pictures interpolate between hourly directions; short events can still be missed.");
		});
		*cursor = seconds.round() as i64;
		ui.horizontal(|ui| {
			ui.label("Step");
			egui::ComboBox::from_id_salt("timeline-step")
				.width(145.0)
				.selected_text(self.options.step.label())
				.show_ui(ui, |ui| {
					for step in [Step::Hour, Step::Day, Step::Month]
					{
						ui.selectable_value(&mut self.options.step, step, step.label());
					}
				});
		});
		self.step_buttons(ui, cursor);
		ui.collapsing("Period", |ui| self.period_controls(ui));
		*cursor = (*cursor).clamp(self.options.period.start, self.options.period.end);
	}

	fn step_buttons(&mut self, ui: &mut egui::Ui, cursor: &mut i64)
	{
		let now = ui.input(|i| i.time);
		ui.horizontal(|ui| {
			for (i,direction) in [-1,1].into_iter().enumerate() {
				let step=if self.options.step==Step::Month {"1 month"} else {self.options.step.label()};
				let label=if direction<0 {format!("< {step}")} else {format!("{step} >")};
				let response=ui.button(label);
				let down=self.repeat[i].held(response.is_pointer_button_down_on(),ui.input(|i|i.pointer.primary_down()));
				if self.repeat[i].tick(down,response.clicked(),now)
					&& let Some(t)=self.options.step.shift(*cursor,direction) {
					*cursor=t.clamp(self.options.period.start,self.options.period.end);
				}
				if down { ui.ctx().request_repaint(); }
			}
			globe::help::question(ui,"Tap to move one step, or hold to repeat. Step also sets statistics sampling; coarse steps can miss light and dark periods.");
		});
	}

	fn period_controls(&mut self, ui: &mut egui::Ui)
	{
		for (i, label) in ["Start", "End"].into_iter().enumerate()
		{
			ui.horizontal(|ui| {
				ui.label(label);
				ui.add(
					egui::TextEdit::singleline(&mut self.dates[i])
						.desired_width(128.0)
						.id_salt(("period-date", i)),
				);
			});
		}
		ui.horizontal(|ui| {
			if ui.button("Apply period").clicked() {
				match Period::parse(&self.dates[0],&self.dates[1]) {
					Ok(period)=> {self.options.period=period;self.validation=None;},
					Err(error)=>self.validation=Some(error),
				}
			}
			globe::help::question(ui,"Enter UTC dates from 2027-01-01 through 2035-01-01. Start must precede end. Later dates download additional sky data only when needed.");
		});
		if let Some(error) = &self.validation
		{
			ui.colored_label(egui::Color32::LIGHT_RED, error);
		}
	}
}
