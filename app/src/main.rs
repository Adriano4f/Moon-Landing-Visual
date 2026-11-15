// Hide the console window only in optimized Windows builds; C++ a platform-specific linker subsystem setting.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Compile this entry point only for native targets; C++ #if around main.
#[cfg(not(target_arch = "wasm32"))]
// Return framework startup errors to the runtime; C++ an expected-returning launcher.
fn main() -> eframe::Result
{
	// Initialize environment-controlled native logging; C++ logging-library initialization.
	env_logger::init();
	// Start the native event loop with the unchanged window title; C++ GUI application launcher.
	eframe::run_native(
		"Moon Landing Visual",
		// Configure depth and multisample storage before context creation; C++ window/context descriptor.
		eframe::NativeOptions {
			// Allocate 24-bit depth for sphere visibility; C++ depth buffer request.
			depth_buffer: 24,
			// Retain four coverage samples per pixel; C++ MSAA sample-count request.
			multisampling: 4,
			// Keep all other options at framework defaults; C++ default construction followed by overrides.
			..Default::default()
		},
		// Own a startup closure returning an owned trait object; C++ unique_ptr<App> from a lambda factory.
		Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
	)
}

// Compile a different entry point for the browser; C++ #if for a web build.
#[cfg(target_arch = "wasm32")]
// Start asynchronous browser initialization without blocking JavaScript; C++ browser bridge entry point.
fn main()
{
	// Bring checked JavaScript casts into method resolution without binding a name; C++ a cast-helper interface.
	use eframe::wasm_bindgen::JsCast as _;
	// Schedule a local asynchronous task; C++ a browser coroutine on the main event loop.
	wasm_bindgen_futures::spawn_local(async {
		// Require the browser DOM, reporting missing startup prerequisites explicitly; C++ checked optional extraction.
		let document = web_sys::window()
			.expect("No window")
			.document()
			.expect("No document");
		// Locate and type-check the canvas created by index.html; C++ checked downcast of a DOM object.
		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("Missing canvas")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("Not a canvas");
		// Start eframe on the existing canvas and suspend until initialization completes; C++ co_await startup.
		let result = eframe::WebRunner::new()
			.start(
				canvas,
				// Request a depth buffer while retaining other web defaults; C++ context descriptor.
				eframe::WebOptions {
					// Keep sphere depth precision consistent with native; C++ context depth request.
					depth_buffer: 24,
					// Inherit unspecified framework defaults; C++ default initialization.
					..Default::default()
				},
				// Move the app into a boxed framework trait object; C++ unique_ptr<App> from a factory lambda.
				Box::new(|cc| Ok(Box::new(app::MoonApp::new(cc)?))),
			)
			.await;
		// Update the initial HTML loading label only if it exists; C++ if(optional).
		if let Some(loading) = document.get_element_by_id("loading_text")
		{
			// Exhaustively handle web startup success/failure; C++ switch over expected's alternatives.
			match result
			{
				// Remove the HTML overlay once egui owns the screen; C++ DOM removal call.
				Ok(()) => loading.remove(),
				// Keep an actionable startup error visible; C++ error alternative.
				Err(err) =>
				{
					// Explain failure in the DOM even if the canvas could not start; C++ textContent assignment.
					loading.set_text_content(Some(
						"The app could not start. See the developer console.",
					));
					// Include detailed startup diagnostics in the browser console; C++ terminate after logging.
					panic!("Failed to start eframe: {err:?}");
				}
			}
		}
	});
}
