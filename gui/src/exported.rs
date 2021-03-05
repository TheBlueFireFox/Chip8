//! The functions that will be exported later on
use crate::{
    adapters::{DisplayAdapter, KeyboardAdapter, SoundCallback},
    observer::{EventSystem, Observer},
    timer::{ProcessWorker, TimingWorker},
    utils 
};
use chip::{devices::Key, resources::RomArchives, Controller};
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    time::Duration,
};
use wasm_bindgen::prelude::*;

/// The first function that has to be run or else no chip like functionality is available.
#[wasm_bindgen]
pub fn setup() -> Result<JsBoundData, JsValue> {
    utils::setup_systems()?;

    let browser_window = utils::BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?;
    // create elements
    let val = browser_window.create_element("p")?;
    val.set_inner_html("Hello from Rust");
    browser_window.append_child(&val)?;

    // get rom names
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let select = utils::crate_dropdown(&browser_window, &files)?;
    browser_window.append_child(&select)?;

    let board = utils::create_board(&browser_window)?;

    browser_window.append_child(&board)?;

    log::info!("Online");

    JsBoundData::new()
}

/// Represents the observed keybpresses
/// TODO: Write documentation, what it is for.
struct ObservedKeypress {
    controller: Rc<RefCell<InternalController>>,
}

impl ObservedKeypress {
    /// The standard new instance.
    fn new(controller: Rc<RefCell<InternalController>>) -> Self {
        Self { controller }
    }
}

impl Observer<Key> for ObservedKeypress {
    fn on_notify(&mut self, event: &Key) {
        self.controller
            .borrow_mut()
            .chipset_mut()
            .expect("Extracting the chipset from the controller, went terribly wrong!")
            .set_key(event.get_index(), event.get_current());
    }
}

/// As the Controller has multiple long parameters, this
/// type is used to abriviate the given configuration.
type InternalController = Controller<DisplayAdapter, KeyboardAdapter, TimingWorker, SoundCallback>;

/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// a compromise had to be chosen, so here is `Rc<RefCell<>>` used.
#[wasm_bindgen]
pub struct JsBoundData {
    controller: Rc<RefCell<InternalController>>,
    worker: Rc<RefCell<ProcessWorker>>,
    keypress_event: EventSystem<Key>,
}

#[wasm_bindgen]
impl JsBoundData {
    /// Will initialize the data structure with the required default values.
    pub(crate) fn new() -> Result<Self, JsValue> {
        let controller = Controller::new(DisplayAdapter::new(), KeyboardAdapter::new());
        let rc_controller = Rc::new(RefCell::new(controller));
        let mut eh = EventSystem::new();

        let keypress = ObservedKeypress::new(rc_controller.clone());
        let keypress = Rc::new(RefCell::new(keypress));
        eh.register_observer(keypress);

        let res = Self {
            controller: rc_controller,
            worker: Rc::new(RefCell::new(ProcessWorker::new()?)),
            keypress_event: eh,
        };

        Ok(res)
    }

    /// Get a mutable reference to the data's controller.
    pub(crate) fn controller_mut(&self) -> RefMut<'_, InternalController> {
        self.controller.borrow_mut()
    }

    /// Get a reference to the data's controller.
    pub(crate) fn controller(&self) -> Ref<'_, InternalController> {
        self.controller.borrow()
    }

    /// Will start executing the
    pub fn start(&self, rom_name: &str) -> Result<(), JsValue> {
        let mut ra = RomArchives::new();

        let rom = ra
            .get_file_data(&rom_name)
            .map_err(|err| JsValue::from(format!("{}", err)))?;

        log::debug!("Loading {}", rom_name);

        self.controller_mut().set_rom(rom);

        utils::print_info(&format!(
            "{}",
            self.controller()
                .chipset()
                .as_ref()
                .ok_or_else(|| JsValue::from("printing went terribly wrong"))?
        ))?;

        // Will setup the worker
        let ccontroller = self.controller.clone();
        let scontroller = self.controller.clone();
        let cworker = self.worker.clone();

        let shutdown_callback = move || {
            stop(cworker, scontroller);
        };

        // Will convert the Data type into a mutable controller, so that
        // it can be used by the chip, this will run a single opcode of the
        // chip.
        let callback = move || {
            // moving the ccontroller into this closure
            let mut controller = ccontroller.borrow_mut();

            // running the chip step
            chip::run(&mut controller)
        };

        self.worker.borrow_mut().start_with_shutdown(
            callback,
            shutdown_callback,
            Duration::from_micros(chip::definitions::cpu::INTERVAL),
        )
    }

    /// Will clear the interval that is running the application
    pub fn stop(&self) {
        stop(self.worker.clone(), self.controller.clone());
    }
}

/// Will stop execution of any and all processes.
fn stop(worker: Rc<RefCell<ProcessWorker>>, controller: Rc<RefCell<InternalController>>) {
    // stop executing chip
    worker.borrow_mut().stop();
    controller.borrow_mut().remove_rom();
}
