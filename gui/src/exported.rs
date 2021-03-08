//! The functions that will be exported later on
use crate::{
    adapters::{DisplayAdapter, KeyboardAdapter, SoundCallback},
    setup::{self, Data},
    timer::TimingWorker,
    utils,
};
use chip::Controller;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

/// The first function that has to be run or else no chip like functionality is available.
#[wasm_bindgen]
pub fn init() -> Result<JsBoundData, JsValue> {
    log::info!("Initializing");

    let bw = utils::BrowserWindow::new()?;

    setup::setup(&bw)?;

    let data = Data::new()?;
    let data = Rc::new(RefCell::new(data));

    let (kc, dc) = {

        let keyboard_closures = setup::setup_keyboard(&bw, data.clone())?;

        let dropdown_closures = setup::setup_dropdown(&bw, data.clone())?;

        (keyboard_closures, dropdown_closures)
    };

    let jd = JsBoundData::new(data, kc, dc);

    Ok(jd)
}

/// As the Controller has multiple long parameters, this
/// type is used to abriviate the given configuration.
pub(crate) type InternalController =
    Controller<DisplayAdapter, KeyboardAdapter, TimingWorker, SoundCallback>;

/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// a compromise had to be chosen, so here is `Rc<RefCell<>>` used.
#[wasm_bindgen]
pub struct JsBoundData {
    data: Rc<RefCell<Data>>,
    _keyboard_closures: setup::KeyboardClosures,
    _dropdown_closures: setup::DropDownClosure,
}

#[wasm_bindgen]
impl JsBoundData {
    /// Will initialize the data structure with the required default values.
    pub(crate) fn new(
        data: Rc<RefCell<Data>>,
        kc: setup::KeyboardClosures,
        dc: setup::DropDownClosure,
    ) -> Self {
        Self {
            data,
            _keyboard_closures: kc,
            _dropdown_closures: dc,
        }
    }

    /// Will start executing the
    pub fn start(&self, rom: &str) -> Result<(), JsValue> {
        self.data.borrow_mut().start(rom)
    }

    /// Will clear the interval that is running the application
    pub fn stop(&self) {
        self.data.borrow().stop()
    }
}
