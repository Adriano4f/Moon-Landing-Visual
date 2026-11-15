use super::*;

impl Timeline
{
	pub fn controls(&mut self, ui: &mut egui::Ui, cursor: &mut i64)
	{
		ui.horizontal_wrapped(|ui| {
			ui.strong(self.options.period.label());
			globe::help::question(ui, &format!(
				"Percentages cover the chosen period using {} samples; Sun means sunlight and Earth means a clear direct link.",
				self.options.step.label(),
			));
		});
		let mut seconds = *cursor as f64;
		let inline_time = ui.available_width() >= 560.0;
		ui.horizontal_wrapped(|ui| {
			ui.spacing_mut().button_padding.x=6.0;
			ui.spacing_mut().item_spacing.x=6.0;
			for (label,preset) in [("1 week",Preset::Week),("1 month",Preset::Month),("1 year",Preset::Year),("All",Preset::All)] {
				if ui.button(label).clicked() {
					self.options.period=preset.period(*cursor);
					self.dates=[self.options.period.start,self.options.period.end].map(|t|DateTime::<Utc>::from_timestamp(t,0).unwrap().format("%Y-%m-%d").to_string());
					self.validation=None;
				}
			}
			let step = if self.options.step == Step::Month { "1 month" } else { self.options.step.label() };
			egui::ComboBox::from_id_salt("timeline-step")
				.width(110.0)
				.selected_text(format!("Step: {step}"))
				.show_ui(ui, |ui| {
					for step in [Step::Hour, Step::Day, Step::Month]
					{
						ui.selectable_value(&mut self.options.step, step, step.label());
					}
				});
			if inline_time
			{
				ui.spacing_mut().slider_width = 90.0;
				ui.add(egui::Slider::new(&mut seconds, self.options.period.start as f64..=self.options.period.end as f64)
					.show_value(false).step_by(1.0).text("Time"));
			}
		});
		if !inline_time
		{
			ui.spacing_mut().slider_width = (ui.available_width() - 60.0).max(80.0);
			ui.add(egui::Slider::new(&mut seconds, self.options.period.start as f64..=self.options.period.end as f64)
				.show_value(false).step_by(1.0).text("Time"));
		}
		*cursor = seconds.round() as i64;
		self.period_controls(ui, cursor);
		*cursor = (*cursor).clamp(self.options.period.start, self.options.period.end);
	}

	fn step_buttons(&mut self, ui: &mut egui::Ui, cursor: &mut i64)
	{
		let now = ui.input(|i| i.time);
		ui.horizontal(|ui| {
			for (i,direction) in [-1,1].into_iter().enumerate() {
				let label=if direction<0 {"<"} else {">"};
				let response=ui.button(label);
				let down=self.repeat[i].held(response.is_pointer_button_down_on(),ui.input(|i|i.pointer.primary_down()));
				if self.repeat[i].tick(down,response.clicked(),now)
					&& let Some(t)=self.options.step.shift(*cursor,direction) {
					*cursor=t.clamp(self.options.period.start,self.options.period.end);
				}
				if down { ui.ctx().request_repaint(); }
			}
			globe::help::question(ui,"Choose a period through 2035-01-01; arrows move by Step and can be held.");
		});
	}

	fn period_controls(&mut self, ui: &mut egui::Ui, cursor: &mut i64)
	{
		ui.horizontal_wrapped(|ui| {
			for (i, label) in ["Start", "End"].into_iter().enumerate()
			{
				ui.label(label);
				ui.add(
					egui::TextEdit::singleline(&mut self.dates[i])
						.desired_width(108.0)
						.hint_text("YYYY-MM-DD")
						.id_salt(("period-date", i)),
				);
			}
			if ui.button("Apply").clicked() {
				match Period::parse(&self.dates[0],&self.dates[1]) {
					Ok(period)=> {self.options.period=period;self.validation=None;},
					Err(error)=>self.validation=Some(error),
				}
			}
			let size = egui::vec2(3.0 * ui.spacing().interact_size.x.max(36.0) + 12.0,
				ui.spacing().interact_size.y.max(36.0));
			ui.allocate_ui(size, |ui| self.step_buttons(ui, cursor));
		});
		if let Some(error) = &self.validation
		{
			ui.colored_label(egui::Color32::LIGHT_RED, error);
		}
	}
}
