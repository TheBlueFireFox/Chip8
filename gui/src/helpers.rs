use web_sys::{Document, HtmlElement, Window};

pub(crate) struct BrowserWindow {
    window: Window,
    document: Document,
    body: HtmlElement,
}

impl BrowserWindow {
    pub fn new() -> Self {
        let window = window();
        let document = document(&window);
        let body = body(&document);
        Self {
            window,
            document,
            body,
        }
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

fn window() -> Window {
    web_sys::window().expect("no global `window` exists.")
}

fn document(window: &Window) -> Document {
    window.document().expect("no document available")
}

fn body(document: &Document) -> HtmlElement {
    document.body().expect("document should have a valid body")
}
