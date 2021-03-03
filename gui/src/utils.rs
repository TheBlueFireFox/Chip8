//! Contains functionality that initializes the console logging as well as the the panic hook.
/// TODO: implement additional functionality over the internal values, so that running code is
/// simpler.

use std::sync::{Arc, Once, RwLock};

use wasm_bindgen::JsValue;
use web_sys::{Document, HtmlElement, Window};

/// An abstraction to the browser window, makes using the `wasm_bindgen` api simpler.
pub(crate) struct BrowserWindow {
    window: Window,
    document: Document,
    body: HtmlElement,
}

impl BrowserWindow {
    /// Create a new browser window
    pub fn new() -> Result<Self, &'static str> {
        let window = web_sys::window().ok_or("no global `window` exists.")?;
        let document = window.document().ok_or("no document available")?;
        let body = document.body().ok_or("document should have a valid body")?;
        Ok(Self {
            window,
            document,
            body,
        })
    }

    /// Will return the window struct
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Will return the document 
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Will return the body
    pub fn body(&self) -> &HtmlElement {
        &self.body
    }
}

lazy_static::lazy_static! {
    /// Will make sure that given unic call setup function can only be calles a single time.
    static ref START: Once = Once::new();
    /// Will store the result of the of the setup function
    static ref START_RESULT: Arc<RwLock<Result<(), log::SetLoggerError>>> =
        Arc::new(RwLock::new(Ok(())));
}

/// Will setup the system 
pub fn setup_systems() -> Result<(), JsValue> {
    // make sure that there will never be a setup call more then once
    START.call_once(|| {
        // will set the panic hook to be the console logs
        crate::utils::set_panic_hook();

        let mut res = START_RESULT.write().unwrap();
        *res = console_log::init_with_level(log::STATIC_MAX_LEVEL.to_level().unwrap());
    });

    if let Err(err) = START_RESULT.read() {
        Err(JsValue::from(format!("{}", err)))
    } else {
        Ok(())
    }
}

/// This is the panic hook it will be called by the JS runtime itself
/// if something happends.
fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
