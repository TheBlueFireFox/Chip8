use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Once, RwLock},
};

use chip::{definitions::display, devices::KeyboardCommands, resources::RomArchives};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::Element;

use crate::{definitions, utils::BrowserWindow, InternalController};

lazy_static::lazy_static! {
    /// Will make sure that given unic call setup function can only be calles a single time.
    static ref START: Once = Once::new();
    /// Will store the result of the of the setup function
    static ref START_RESULT: Arc<RwLock<Result<(), log::SetLoggerError>>> =
        Arc::new(RwLock::new(Ok(())));
}

pub(crate) struct Elements {
    table: Element,
    dropdown: Element,
}

impl Elements {
    /// Get a reference to the elements's table.
    pub(crate) fn table(&self) -> &Element {
        &self.table
    }

    /// Get a reference to the elements's dropdown.
    pub(crate) fn dropdown(&self) -> &Element {
        &self.dropdown
    }
}

pub(crate) fn setup(browser_window: &BrowserWindow) -> Result<Elements, JsValue> {
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

    let board = create_board(&browser_window)?;

    // setup_keyboard(&browser_window, &board)?;

    browser_window.append_child(&board)?;

    Ok(Elements {
        table: board,
        dropdown: select,
    })
}

/// Will setup the system
fn setup_systems() -> Result<(), JsValue> {
    // make sure that there will never be a setup call more then once
    START.call_once(|| {
        // will set the panic hook to be the console logs
        set_panic_hook();

        let mut res = START_RESULT.write().unwrap();
        *res = console_log::init_with_level(log::STATIC_MAX_LEVEL.to_level().unwrap());
    });

    if let Err(err) = START_RESULT.read() {
        Err(JsValue::from(format!("{}", err)))
    } else {
        Ok(())
    }
}

type KeyboardClosure = Closure<dyn FnMut(web_sys::KeyboardEvent)>;

pub(crate) struct KeyboardClosures {
    _keydown: KeyboardClosure,
    _keyup: KeyboardClosure,
}

pub(crate) fn setup_keyboard(
    controller: Rc<RefCell<InternalController>>,
    table: &Element,
) -> Result<KeyboardClosures, JsValue> {
    // The actuall callback that is executed every time a key event is called
    fn callback(event: web_sys::KeyboardEvent, controller: &mut InternalController, to: bool) {
        // let mut controller = keydown_controller.borrow_mut();
        let keyboard = controller.keyboard();
        let event = &event.code();
        log::debug!("was pressed {}", event);
        for (i, row) in definitions::keyboard::BROWSER_LAYOUT.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if *cell == event {
                    let key = i * 4 + j;
                    keyboard.set_key(key, to);
                    return;
                }
            }
        }
    }
    struct KeyEvent<'a> {
        name: &'a str,
        state: bool,
    }

    let keydown = KeyEvent {
        name: "keydown",
        state: true,
    };
    let keyup = KeyEvent {
        name: "keydown",
        state: false,
    };

    let register = move |KeyEvent { name, state }| -> Result<KeyboardClosure, JsValue> {
        let event_controller = controller.clone();
        let istate = state;
        let callback = move |event: web_sys::KeyboardEvent| {
            let mut controller = event_controller.borrow_mut();
            callback(event, &mut controller, istate);
        };

        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        table.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())?;

        Ok(closure)
    };

    Ok(KeyboardClosures {
        _keydown: register(keydown)?,
        _keyup: register(keyup)?,
    })
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
