//! Native and WebAssembly entry points for the Moon Landing Visual application.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result
{
	env_logger::init();
	eframe::run_native(
		"Moon Landing Visual",
		// Depth testing is required for sphere visibility.
		eframe::NativeOptions {
			depth_buffer: 24,
			multisampling: 4,
			..Default::default()
		},
		Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
	)
}

#[cfg(target_arch = "wasm32")]
/// Start asynchronous browser initialization without blocking JavaScript.
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
				eframe::WebOptions {
					// Keep sphere depth precision consistent with native.
					depth_buffer: 24,
					..Default::default()
				},
				Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
			)
			.await;
		if let Some(loading) = document.get_element_by_id("loading_text")
		{
			match result
			{
				Ok(()) => loading.remove(),
				Err(err) =>
				{
					// Explain failure in the DOM even if the canvas could not start.
					loading.set_text_content(Some(
						"The app could not start. See the developer console.",
					));
					panic!("Failed to start eframe: {err:?}");
				}
			}
		}
	});
}
