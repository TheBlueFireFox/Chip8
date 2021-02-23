use {
    crate::{
        adapters::{DisplayAdapter, KeyboardAdapter},
        definitions,
        observer::{EventSystem, Observer},
        timer::{TimingWorker, WasmWorker},
        utils::{set_panic_hook, BrowserWindow},
    },
    chip::{definitions::display, devices::Key, resources::RomArchives, Controller},
    std::{
        cell::{Ref, RefCell, RefMut},
        rc::Rc,
        time::Duration,
    },
    wasm_bindgen::prelude::*,
    web_sys::Element,
};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

pub(crate) fn console_log(s: &str) {
    unsafe {log(s);};
}

fn create_board(window: &BrowserWindow) -> Result<Element, JsValue> {
    let table = window.document().create_element(definitions::field::TYPE)?;

    for i in 0..display::HEIGHT {
        let tr = window
            .document()
            .create_element(definitions::field::TYPE_ROW)?;
        for j in 0..display::WIDTH {
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
    // will set the panic hook to be the console logs
    set_panic_hook();

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

struct ObservedKeypress {
    controller: Rc<RefCell<InternalController>>,
}

impl ObservedKeypress {
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
type InternalController = Controller<DisplayAdapter, KeyboardAdapter, TimingWorker>;

/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// a compromise had to be chosen, so here is `Rc<RefCell<>>` used.
#[wasm_bindgen]
pub struct JsBoundData {
    controller: Rc<RefCell<InternalController>>,
    worker: WasmWorker,
    keypress_event: EventSystem<Key>,
}

#[wasm_bindgen]
impl JsBoundData {
    pub(crate) fn new() -> Self {
        let controller = Controller::new(DisplayAdapter::new(), KeyboardAdapter::new());
        let rc_controller = Rc::new(RefCell::new(controller));
        let mut eh = EventSystem::new();

        let keypress = ObservedKeypress::new(rc_controller.clone());
        let keypress = Rc::new(RefCell::new(keypress));
        eh.register_observer(keypress);

        let res = Self {
            controller: rc_controller,
            worker: WasmWorker::new(),
            keypress_event: eh,
        };

        res
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
    pub fn start(&mut self, rom_name: &str) -> Result<(), JsValue> {
        let mut ra = RomArchives::new();

        let rom = ra
            .get_file_data(&rom_name)
            .map_err(|err| JsValue::from(format!("{}", err)))?;

        self.controller_mut().set_rom(rom);

        // Will setup the worker
        let ccontroller = self.controller.clone();

        // Will convert the Data type into a mutable controller, so that
        // it can be used by the chip, this will run a single opcode of the
        // chip.
        let callback = move || {
            // moving the ccontroller into this closure
            let mut controller = ccontroller.borrow_mut();
            console_log(&format!("{:?}", controller.operation()));

            // running the chip step
            chip::run(&mut controller)
                .expect("Something went wrong while stepping to the next step.");
        };
        self.worker.start(
            callback,
            Duration::from_micros(chip::definitions::cpu::INTERVAL),
        )?;

        Ok(())
    }

    /// Will clear the interval that is running the application
    pub fn stop(&mut self) {
        // stop executing chip
        self.worker.stop();
        self.controller_mut().remove_rom();
    }
}
