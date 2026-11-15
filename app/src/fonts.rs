use eframe::egui::{Context, FontData, FontDefinitions, FontFamily, FontTweak};

// Preserve interface glyph metrics without embedding unused scripts; C++ font registry setup.
pub(super) fn install(ctx: &Context)
{
	let mut fonts = FontDefinitions::empty();
	for (name, bytes, scale) in [
		(
			"body",
			include_bytes!("../assets/fonts/karaya-ui.ttf").as_slice(),
			1.0,
		),
		(
			"mono",
			include_bytes!("../assets/fonts/karaya-mono.ttf").as_slice(),
			1.0,
		),
		(
			"egui-icons",
			include_bytes!("../assets/fonts/egui-icons.ttf").as_slice(),
			0.9,
		),
	]
	{
		fonts.font_data.insert(
			name.into(),
			FontData::from_static(bytes)
				.tweak(FontTweak {
					scale,
					..Default::default()
				})
				.into(),
		);
	}
	fonts.families.insert(
		FontFamily::Proportional,
		vec!["body".into(), "egui-icons".into(), "mono".into()],
	);
	fonts.families.insert(
		FontFamily::Monospace,
		vec!["mono".into(), "body".into(), "egui-icons".into()],
	);
	ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests
{
	use super::*;
	use eframe::egui::{FontId, RawInput};

	#[test]
	fn interface_symbols_empty_and_unknown_glyphs()
	{
		let ctx = Context::default();
		install(&ctx);
		let mut output = ctx.run_ui(RawInput::default(), |ui| {
			ui.fonts_mut(|fonts| {
				for family in [FontFamily::Proportional, FontFamily::Monospace]
				{
					let id = FontId::new(15.0, family);
					assert!(fonts.has_glyphs(&id, "Karaya Gläser 0° 50% ● ■ × ← → … − ·"));
					assert!(fonts.has_glyphs(&id, ""));
					assert!(!fonts.has_glyph(&id, '\u{10ffff}'));
				}
			});
		});
		output.textures_delta.clear();
	}
}
