use web_sys::{Document, HtmlElement, Window};

pub(crate) struct BrowserWindow {
    window: Window,
    document: Document,
    body: HtmlElement,
}

impl BrowserWindow {
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
    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn body(&self) -> &HtmlElement {
        &self.body
    }
}

/// This is the panic hook it will be called by the JS runtime itself
/// if something happends.
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
