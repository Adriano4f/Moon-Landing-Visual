use super::*;

// Compute geometric lunar phase from real Moon-fixed directions; C++ checked vector geometry.
fn lunar_phase(sun: [f32; 3], earth: [f32; 3]) -> Option<(f32, f32, bool)>
{
	let norm =
		(sun.iter().map(|x| x * x).sum::<f32>() * earth.iter().map(|x| x * x).sum::<f32>()).sqrt();
	if !norm.is_finite() || norm < 1e-6
	{
		return None;
	}
	let cosine = (sun.iter().zip(earth).map(|(s, e)| s * e).sum::<f32>() / norm).clamp(-1.0, 1.0);
	let waxing = sun[2] * earth[0] - sun[0] * earth[2] < 0.0;
	Some((cosine.acos().to_degrees(), (1.0 + cosine) / 2.0, waxing))
}

impl Globe
{
	// Keep a naturally dark phase distinct from missing data or eclipses; C++ presentation helper.
	pub(super) fn phase_status(&mut self, ui: &mut egui::Ui, light: Lighting)
	{
		let Some((angle, fraction, waxing)) = lunar_phase(light.sun, light.earth)
		else
		{
			return;
		};
		let phase = phase_label(angle, fraction, waxing);
		help::annotated(
			ui,
			format!("{phase} · {:.1}% lit", fraction * 100.0),
			"Share of the Moon's Earth-facing side lit by the Sun on the selected date.",
		);
		if fraction < 0.1
		{
			ui.horizontal(|ui| {
				let label = if self.boosted { "Earthshine boosted" } else { "Boost earthshine" };
				if ui.add_enabled(!self.boosted, egui::Button::new(label)).clicked()
				{
					self.boosted = true;
				}
				help::question(ui, "Brighten sunlight reflected by Earth so the dark face becomes visible.");
			});
		}
	}
}

// Name approximate phases without equating their illuminated area with brightness; C++ classification.
fn phase_label(angle: f32, fraction: f32, waxing: bool) -> &'static str
{
	if angle > 168.0
	{
		"Near new Moon"
	}
	else if angle < 12.0
	{
		"Near full Moon"
	}
	else if (angle - 90.0).abs() < 2.0
	{
		"Half Moon"
	}
	else if fraction < 0.5
	{
		if waxing
		{
			"Waxing crescent"
		}
		else
		{
			"Waning crescent"
		}
	}
	else if waxing
	{
		"Waxing gibbous"
	}
	else
	{
		"Waning gibbous"
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn phases_normalized_and_invalid()
	{
		for (sun, expected) in [
			([0.0, 0.0, 2.0], 1.0),
			([0.0, 0.0, -2.0], 0.0),
			([2.0, 0.0, 0.0], 0.5),
		]
		{
			let (angle, fraction, _) = lunar_phase(sun, [0.0, 0.0, 3.0]).unwrap();
			assert!((fraction - expected).abs() < 1e-6);
			assert!((angle.to_radians().cos() - (2.0 * fraction - 1.0)).abs() < 1e-6);
		}
		assert!(lunar_phase([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap().2);
		assert!(!lunar_phase([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]).unwrap().2);
		assert!(lunar_phase([0.0; 3], [1.0; 3]).is_none());
		assert!(lunar_phase([f32::NAN; 3], [1.0; 3]).is_none());
		assert!(lunar_phase([1.0; 3], [f32::INFINITY; 3]).is_none());
		assert_eq!(phase_label(180.0, 0.0, true), "Near new Moon");
		assert_eq!(phase_label(0.0, 1.0, false), "Near full Moon");
		assert_eq!(phase_label(90.0, 0.5, false), "Half Moon");
		assert_eq!(phase_label(120.0, 0.25, true), "Waxing crescent");
		assert_eq!(phase_label(120.0, 0.25, false), "Waning crescent");
		assert_eq!(phase_label(60.0, 0.75, true), "Waxing gibbous");
		assert_eq!(phase_label(60.0, 0.75, false), "Waning gibbous");
	}
}
