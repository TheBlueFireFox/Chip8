use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    time::Duration,
};

use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::{
    definitions,
    timer::{TimingWorker, WasmWorker},
    utils::BrowserWindow,
    DisplayAdapter, KeyboardAdapter,
};
use chip::{
    definitions::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
    resources::RomArchives,
    Controller,
};

fn create_board(window: &BrowserWindow) -> Result<Element, JsValue> {
    let table = window.document().create_element(definitions::field::TYPE)?;

    for i in 0..DISPLAY_HEIGHT {
        let tr = window
            .document()
            .create_element(definitions::field::TYPE_ROW)?;
        for j in 0..DISPLAY_WIDTH {
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

fn crate_dropdown(window: &BrowserWindow, files: &[&str]) -> Result<Element, JsValue> {
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

#[wasm_bindgen]
pub fn setup() -> Result<JsBoundData, JsValue> {
    let browser_window = BrowserWindow::new();
    // create elements
    let val = browser_window.document().create_element("p")?;
    val.set_inner_html("Hello from Rust");
    browser_window.body().append_child(&val)?;

    // get rom names
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let select = crate_dropdown(&browser_window, &files)?;
    browser_window.body().append_child(&select)?;

    let board = create_board(&browser_window)?;

    browser_window.body().append_child(&board)?;

    let data = JsBoundData::new();

    Ok(data)
}

/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// a compromise had to be chosen, so here is `Rc<RefCell<>>` used.
#[wasm_bindgen]
pub struct JsBoundData {
    controller: Rc<RefCell<Controller<DisplayAdapter, KeyboardAdapter, TimingWorker>>>,
    interval: u32,
    worker: WasmWorker,
}

#[wasm_bindgen]
impl JsBoundData {
    pub(crate) fn new() -> Self {
        let controller = Controller::new(DisplayAdapter::new(), KeyboardAdapter::new());

        Self {
            controller: Rc::new(RefCell::new(controller)),
            interval: chip::definitions::CPU_INTERVAL as u32,
            worker: WasmWorker::new(),
        }
    }

    /// Get a mutable reference to the data's controller.
    pub(crate) fn controller_mut(
        &self,
    ) -> RefMut<'_, Controller<DisplayAdapter, KeyboardAdapter, TimingWorker>> {
        self.controller.borrow_mut()
    }

    /// Get a reference to the data's controller.
    pub(crate) fn controller(
        &self,
    ) -> Ref<'_, Controller<DisplayAdapter, KeyboardAdapter, TimingWorker>> {
        self.controller.borrow()
    }

    /// Get a reference to the data's interval.
    pub fn interval(&self) -> u32 {
        self.interval
    }

    /// Get a reference to the data's callback id.
    pub fn callback_id(&self) -> Option<i32> {
        self.worker.interval_id()
    }

    /// Will start executing the
    pub fn start(&mut self, rom_name: &str) -> Result<(), JsValue> {
        let mut ra = RomArchives::new();

        let rom = ra
            .get_file_data(&rom_name)
            .map_err(|err| JsValue::from(format!("{}", err)))?;

        self.controller_mut().set_rom(rom);

        // Will setup the worker
        let controller = self.controller.clone();

        // Will convert the Data type into a mutable controller, so that
        // it can be used by the chip, this will run a single opcode of the
        // chip.
        let callback = move || {
            chip::run(&mut *controller.borrow_mut())
                .expect("Something went wrong while stepping to the next step.");
        };
        self.worker.start(
            callback,
            Duration::from_micros(chip::definitions::CPU_INTERVAL),
        )?;

        Ok(())
    }

    /// Will clear the interval that is running the application
    pub fn stop(&mut self) {
        // stop executing chip
        self.worker.stop();
    }
}
