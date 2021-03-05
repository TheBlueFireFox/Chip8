//! Contains functionality that initializes the console logging as well as the the panic hook.
use std::sync::{Arc, Once, RwLock};

use chip::definitions::display;
use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, HtmlElement, Node, Window};

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

    pub fn append_child(&self, element: &Element) -> Result<(), JsValue> {
        self.body.append_child(element).and_then(|_| Ok(()))
    }

    pub fn replace_child(&self, old: &Node, new: &Node) -> Result<(), JsValue> {
        self.body.replace_child(new, old).and_then(|_| Ok(()))
    }

    pub fn create_element(&self, element_type: &str) -> Result<Element, JsValue> {
        self.document.create_element(element_type)
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<Element> {
        self.document.get_element_by_id(id)
    }

    pub fn set_timeout(&self, callback: &Function, timeout: i32) -> Result<i32, JsValue> {
        self.window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback, timeout)
    }

    pub fn clear_timeout(&self, handle: i32) {
        self.window.clear_timeout_with_handle(handle);
    }

    pub fn set_interval(&self, callback: &Function, timeout: i32) -> Result<i32, JsValue> {
        self.window
            .set_interval_with_callback_and_timeout_and_arguments_0(callback, timeout)
    }

    pub fn clear_interval(&self, handle: i32) {
        self.window.clear_interval_with_handle(handle);
    }
}

/// Will draw the empty initial board. For visual confirmation, that the process started
/// the board will be drawn in a chess like pattern.
pub(crate) fn create_board(window: &BrowserWindow) -> Result<Element, JsValue> {
    let table = window.create_element(definitions::field::TYPE)?;
    table.set_id(definitions::field::ID);

    for i in 0..display::WIDTH {
        let tr = window.create_element(definitions::field::TYPE_ROW)?;
        for j in 0..display::HEIGHT {
            let td = window.create_element(definitions::field::TYPE_COLUMN)?;
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
pub(crate) fn crate_dropdown(window: &BrowserWindow, files: &[&str]) -> Result<Element, JsValue> {
    let dropdown = window.create_element(definitions::selector::TYPE)?;
    dropdown.set_id(definitions::selector::ID);
    for file in files.into_iter() {
        let option = window.create_element("option")?;
        option.set_attribute("value", *file)?;
        option.set_text_content(Some(*file));
        dropdown.append_child(&option)?;
    }
    Ok(dropdown)
}

pub(crate) fn print_info(message: &str) -> Result<(), JsValue> {
    let bw = BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?;
    let pre = bw.create_element("pre")?;
    pre.set_text_content(Some(message));

    bw.append_child(&pre)?;
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
