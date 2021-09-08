use chip::{devices::KeyboardCommands, resources::RomArchives, Controller};
use parking_lot::Once;
use std::{cell::RefCell, rc::Rc, time::Duration};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Element;

use crate::{
    adapters::{DisplayAdapter, KeyboardAdapter, SoundCallback},
    definitions,
    timer::{ProcessWorker, TimingWorker},
    utils::{self, BrowserWindow},
};

/// Will make sure that given unique call setup function can only be calles a single time.
static START: Once = Once::new();

/// As the Controller has multiple long parameters, this
/// type is used to abriviate the given configuration.
type InternalController = Controller<DisplayAdapter, KeyboardAdapter, TimingWorker, SoundCallback>;

pub(crate) struct Data {
    controller: Rc<RefCell<InternalController>>,
    worker: Rc<RefCell<ProcessWorker>>,
}

impl Data {
    pub fn new(da: DisplayAdapter, ka: KeyboardAdapter) -> Result<Self, JsValue> {
        let controller = InternalController::new(da, ka);
        let rc_controller = Rc::new(RefCell::new(controller));

        Ok(Self {
            controller: rc_controller,
            worker: Rc::new(RefCell::new(ProcessWorker::new()?)),
        })
    }

    pub fn start(&self, rom_name: &str) -> Result<(), JsValue> {
        self.stop();

        let mut ra = RomArchives::new();

        let rom = ra
            .get_file_data(&rom_name)
            .map_err(|err| JsValue::from(format!("{}", err)))?;

        log::debug!("Loading {}", rom_name);

        self.controller.borrow_mut().set_rom(rom);

        utils::print_info(
            &format!(
                "{}",
                self.controller
                    .borrow()
                    .chipset()
                    .as_ref()
                    .ok_or_else(|| JsValue::from("printing went terribly wrong"))?,
            ),
            definitions::info::ID,
        )?;

        // Will setup the worker
        let shutdown_callback = {
            let scontroller = self.controller.clone();
            let cworker = self.worker.clone();

            move || {
                stop(cworker, scontroller);
            }
        };

        // Will convert the Data type into a mutable controller, so that
        // it can be used by the chip, this will run a single opcode of the
        // chip.
        let callback = {
            let ccontroller = self.controller.clone();

            move || {
                // moving the ccontroller into this closure
                let mut controller = ccontroller.borrow_mut();

                // running the chip step
                // using anhow::result magic to convert this here
                chip::run(&mut controller)?;
                Ok(())
            }
        };

        self.worker.borrow_mut().start_with_shutdown(
            callback,
            shutdown_callback,
            Duration::from_micros(chip::definitions::cpu::INTERVAL),
        )
    }

    pub fn stop(&self) {
        stop(self.worker.clone(), self.controller.clone());
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        self.stop()
    }
}

/// Will stop execution of any and all processes.
fn stop(worker: Rc<RefCell<ProcessWorker>>, controller: Rc<RefCell<InternalController>>) {
    // stop executing chip
    worker.borrow_mut().stop();
    controller.borrow_mut().remove_rom();
}

pub(crate) fn setup(browser_window: &BrowserWindow) -> Result<Data, JsValue> {
    log::debug!("Setting up the system");

    setup_systems()?;

    // let browser_window = BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?;

    // create elements
    let val = browser_window.create_element("p")?;
    val.set_inner_html("Hello from Rust");
    browser_window.append_child(&val)?;

    // get rom names
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let select = crate_dropdown(&browser_window, &files)?;

    browser_window.append_child(&select)?;

    let mut da = DisplayAdapter::new(&browser_window);
    da.create_board()?;

    let ka = KeyboardAdapter::new();

    Data::new(da, ka)
}

/// Will setup the system
fn setup_systems() -> Result<(), JsValue> {
    // make sure that there will never be a setup call more then once
    START.call_once(|| {
        // will set the panic hook to be the console logs
        set_panic_hook();
    });

    if START.state().done() {
        Ok(())
    } else {
        Err("START controller was poisoned".into())
    }
}

type EventClosure = Closure<dyn FnMut(web_sys::Event)>;

struct EventListener {
    name: &'static str,
    closure: EventClosure,
    element: Element,
}

impl EventListener {
    fn new<F>(name: &'static str, callback: F, element: &Element) -> Result<Self, JsValue>
    where
        F: FnMut(web_sys::Event) + 'static,
    {
        let element = element.clone();
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(web_sys::Event)>);
        element.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())?;

        Ok(Self {
            name,
            closure,
            element,
        })
    }
}

impl Drop for EventListener {
    fn drop(&mut self) {
        self.element
            .remove_event_listener_with_callback(self.name, self.closure.as_ref().unchecked_ref())
            .expect("Something went wrong with removing the event listener.");
    }
}

pub(crate) struct KeyboardClosures {
    _keydown: EventListener,
    _keyup: EventListener,
}

pub(crate) fn setup_keyboard(
    browser_window: &BrowserWindow,
    data: Rc<Data>,
) -> Result<KeyboardClosures, JsValue> {
    // The actuall callback that is executed every time a key event is called
    fn check_keypress(event: &str, controller: &mut InternalController, to: bool) {
        let keyboard = controller.keyboard();

        for (i, row) in definitions::keyboard::BROWSER_LAYOUT.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if *cell == event {
                    // translate from the 2d matrix to the 1d
                    let key = i * row.len() + j;
                    log::debug!(
                        "{} key was registered and mapped to {}",
                        event,
                        definitions::keyboard::LAYOUT[i][j]
                    );
                    keyboard.set_key(key, to);
                    return;
                }
            }
        }
    }

    let controller = data.controller.clone();

    let register = move |name, state| -> Result<EventListener, JsValue> {
        let event_controller = controller.clone();

        let callback = move |event: web_sys::Event| {
            let event: web_sys::KeyboardEvent = event.dyn_into().unwrap();

            log::trace!("was registered {} for {}", event.code(), name);

            let mut controller = event_controller.borrow_mut();
            check_keypress(&event.code(), &mut controller, state);
        };

        log::trace!("registering event {}", name);

        EventListener::new(name, callback, browser_window.body())
    };

    Ok(KeyboardClosures {
        _keydown: register("keydown", true)?,
        _keyup: register("keyup", false)?,
    })
}

pub(crate) struct DropDownClosure {
    _selector: EventListener,
}

pub(crate) fn setup_dropdown(
    browser_window: &BrowserWindow,
    data: Rc<Data>,
) -> Result<DropDownClosure, JsValue> {
    let dropdown = browser_window
        .get_element_by_id(definitions::selector::ID)
        .ok_or_else(|| JsValue::from("No such element found."))?;

    let callback = move |event: web_sys::Event| {
        // SAFETY: These expect unwraps are safe given the context in which
        // the function will be called from.
        let target: web_sys::HtmlSelectElement = event
            .target()
            .expect("Unable to extract the event target from the event handler.")
            .dyn_into()
            .expect("Unable to convert the event target to the selected HTML element.");

        let rom = target.value();
        log::trace!("loaded {}", rom);

        if let Err(err) = data.start(&rom) {
            data.stop();
            panic!("{:?}", err)
        }
    };

    let event = EventListener::new("change", callback, &dropdown)?;

    Ok(DropDownClosure { _selector: event })
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

/// Will initialize the drop down with the included rom names.
pub(crate) fn crate_dropdown(window: &BrowserWindow, files: &[&str]) -> Result<Element, JsValue> {
    let dropdown = window.create_element(definitions::selector::TYPE)?;
    dropdown.set_id(definitions::selector::ID);
    for file in files {
        let option = window.create_element("option")?;
        option.set_attribute("value", *file)?;
        option.set_text_content(Some(*file));
        dropdown.append_child(&option)?;
    }
    Ok(dropdown)
}
