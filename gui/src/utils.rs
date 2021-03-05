//! Contains functionality that initializes the console logging as well as the the panic hook.
/// TODO: implement additional functionality over the internal values, so that running code is
/// simpler.
use std::sync::{Arc, Once, RwLock};

use chip::definitions::display;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, HtmlElement, Window};

use crate::definitions;

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

/// Will draw the empty initial board. For visual confirmation, that the process started
/// the board will be drawn in a chess like pattern.
/// TODO: refactor this function into untils.
pub(crate) fn create_board(window: &BrowserWindow) -> Result<Element, JsValue> {
    let table = window.document().create_element(definitions::field::TYPE)?;
    table.set_id(definitions::field::ID);

    for i in 0..display::WIDTH {
        let tr = window
            .document()
            .create_element(definitions::field::TYPE_ROW)?;
        for j in 0..display::HEIGHT {
            let td = window
                .document()
                .create_element(definitions::field::TYPE_COLUMN)?;
            if (i + j) % 2 == 0 {
                td.set_class_name(definitions::field::ACTIVE);
            }

            tr.append_child(&td)?;
        }
        table.append_child(&tr)?;
    }

    Ok(table)
}

/// Will initialize the drop down with the included rom names.
/// TODO: refactor this function into untils.
pub(crate) fn crate_dropdown(window: &BrowserWindow, files: &[&str]) -> Result<Element, JsValue> {
    let dropdown = window
        .document()
        .create_element(definitions::selector::TYPE)?;
    dropdown.set_id(definitions::selector::ID);
    for file in files.into_iter() {
        let option = window.document().create_element("option")?;
        option.set_attribute("value", *file)?;
        option.set_text_content(Some(*file));
        dropdown.append_child(&option)?;
    }
    Ok(dropdown)
}

pub(crate) fn print_info(message: &str) -> Result<(), JsValue> {
    let bw = BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?;
    let doc = bw.document();
    let pre = doc.create_element("pre")?;
    pre.set_text_content(Some(message));

    bw.body().append_child(&pre)?;
    Ok(())
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
