//! Contains functionality that initializes the console logging as well as the the panic hook.
use js_sys::Function;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, HtmlElement, Node, Text, Window};

pub(crate) fn print_info(message: &str, id: &str) -> Result<(), JsValue> {
    let bw = BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?;
    // check if the pre-tag with the given ID (id) exists
    let pre = match bw.get_element_by_id(id) {
        Some(elem) => elem,
        None => {
            let pre = bw.create_element("pre")?;
            pre.set_id(id);
            bw.append_child(&pre)?;
            pre
        }
    };

    pre.set_text_content(Some(message));
    Ok(())
}
/// An abstraction to the browser window, makes using the `wasm_bindgen` api simpler.
#[derive(Clone)]
pub(crate) struct BrowserWindow {
    window: Window,
    document: Document,
    body: HtmlElement,
}

#[allow(unused)]
pub enum Tag<'elem> {
    Body,
    Head,
    Element(&'elem str),
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
        self.append_child_to(element, Tag::Body)
    }

    pub fn append_child_to<'tag>(&self, element: &Element, to: Tag<'tag>) -> Result<(), JsValue> {
        match to {
            Tag::Body => self.body.append_with_node_1(element),
            Tag::Head => self
                .document
                .head()
                .ok_or_else(|| JsValue::from("unable to find the element called head"))?
                .append_with_node_1(element),
            Tag::Element(to) => {
                let elem = self.get_element_by_id(to).ok_or_else(|| {
                    JsValue::from(format!("unable to find the element called {}", to))
                })?;
                elem.append_with_node_1(element)
            }
        }
    }

    pub fn replace_child(&self, old: &Node, new: &Node) -> Result<(), JsValue> {
        self.body.replace_child(new, old).and_then(|_| Ok(()))
    }

    pub fn create_element(&self, element_type: &str) -> Result<Element, JsValue> {
        self.document.create_element(element_type)
    }

    pub fn create_text_node(&self, text: &str) -> Result<Text, JsValue> {
        Ok(self.document.create_text_node(text))
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

    /// Get a reference to the browser window's body.
    pub(crate) fn body(&self) -> &HtmlElement {
        &self.body
    }
}
