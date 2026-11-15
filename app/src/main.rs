#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result
{
	env_logger::init();
	eframe::run_native(
		"Karaya Explore",
		eframe::NativeOptions::default(),
		Box::new(|_| Ok(Box::new(app::MoonApp))),
	)
}

#[cfg(target_arch = "wasm32")]
fn main()
{
	use eframe::wasm_bindgen::JsCast as _;
	wasm_bindgen_futures::spawn_local(async {
		let document = web_sys::window()
			.expect("No window")
			.document()
			.expect("No document");
		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("Missing canvas")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("Not a canvas");
		let result = eframe::WebRunner::new()
			.start(
				canvas,
				eframe::WebOptions::default(),
				Box::new(|_| Ok(Box::new(app::MoonApp))),
			)
			.await;
		if let Some(loading) = document.get_element_by_id("loading_text")
		{
			match result
			{
				Ok(()) => loading.remove(),
				Err(err) =>
				{
					loading.set_text_content(Some(
						"The app could not start. See the developer console.",
					));
					panic!("Failed to start eframe: {err:?}");
				}
			}
		}
	});
}
