//! The functions that will be exported later on
use crate::{setup, utils};

use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// The first function that has to be run or else no chip like functionality is available.
#[wasm_bindgen]
pub fn init() -> Result<JsBoundData, JsValue> {
    log::info!("Initializing");

    let bw = utils::BrowserWindow::new()?;

    setup::setup(&bw)?;

    let data = Rc::new(setup::Data::new()?);

    let kc = setup::setup_keyboard(&bw, data.clone())?;

    let dc = setup::setup_dropdown(&bw, data.clone())?;

    Ok(JsBoundData {
        data,
        _kc: kc,
        _dc: dc,
    })
}

/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// only internal mutability is uesd.
#[wasm_bindgen]
pub struct JsBoundData {
    data: Rc<setup::Data>,
    _kc: setup::KeyboardClosures,
    _dc: setup::DropDownClosure,
}

#[wasm_bindgen]
impl JsBoundData {
    /// Will start executing the
    pub fn start(&self, rom: &str) -> Result<(), JsValue> {
        self.data.start(rom)
    }

    /// Will clear the interval that is running the application
    pub fn stop(&self) {
        self.data.stop()
    }
}
