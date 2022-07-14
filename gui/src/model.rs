use std::{
    cell::RefCell,
    rc::Rc,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use chip::{
    devices::KeyboardCommands,
    resources::RomArchives,
    timer::{TimedWorker, Timer},
};
use parking_lot::Mutex;
use yew::{
    classes, function_component, html, Callback, Component, Context, Html, Properties, TargetCast,
};

use crate::{
    adapter::{DisplayAdapter, DisplayState, KeyboardAdapter, SoundCallback},
    timer::TimingWorker,
};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <State />
    }
}

#[derive(Debug)]
enum Msg {
    Roms(usize),
    Keyboard(yew::KeyboardEvent, bool),
    Display,
}

#[derive(Debug)]
struct KeyboardCallbacks {
    key_up: Callback<yew::KeyboardEvent>,
    key_down: Callback<yew::KeyboardEvent>,
}

#[derive(custom_debug::Debug)]
struct State {
    props: Props,
    keyboard_callbacks: KeyboardCallbacks,
    #[debug(skip)]
    tick_timer: Rc<RefCell<Option<gloo::timers::callback::Interval>>>,
    #[debug(skip)]
    controller:
        Arc<Mutex<chip::Controller<DisplayAdapter, KeyboardAdapter, TimingWorker, SoundCallback>>>,
}

impl Component for State {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        use chip::definitions::display;

        let rom_props = {
            let callback = ctx.link().callback(Msg::Roms);

            RomProp {
                callback,
                roms: Default::default(),
            }
        };

        let (da, display_state) = {
            let display_callback = ctx.link().callback(|_| Msg::Display);
            // add default pattern
            let state = (0..display::WIDTH)
                .map(|y| (0..display::HEIGHT).map(|x| (y + x) % 2 == 0).collect())
                .collect();

            DisplayAdapter::new(state, display_callback)
        };

        let field_prop = FieldProp {
            display: display_state,
        };

        let ka = KeyboardAdapter::new();
        let keyboard_callbacks = {
            let callback = ctx
                .link()
                .callback(|(event, state)| Msg::Keyboard(event, state));

            let callback_up = callback.clone();

            let key_up = yew::Callback::from(move |event| callback_up.emit((event, true)));
            let key_down = yew::Callback::from(move |event| callback.emit((event, false)));

            KeyboardCallbacks { key_up, key_down }
        };

        let controller = Arc::new(Mutex::new(chip::Controller::new(da, ka)));

        let props = Props {
            field: field_prop,
            rom: rom_props,
        };

        Self {
            props,
            controller,
            keyboard_callbacks,
            tick_timer: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roms(new) => {
                /* update state */
                // TODO: update active chip
                self.props.rom.roms.chosen = new;
                let name = &self.props.rom.roms.files[new];
                log::debug!("name is <{}>", name);

                // setup correct rom
                let mut ra = RomArchives::new();
                let rom = ra.get_file_data(name);
                let rom = rom.expect("Able to correctly unwrap this rom file");

                self.controller.lock().set_rom(rom);

                // setup ticker
                if let Some(interval) = self.tick_timer.take() {
                    let interval = interval.cancel();
                    // explicit drop and cancel just to make sure
                    drop(interval);
                }

                let tt = self.tick_timer.clone();

                let controller = self.controller.clone();
                let callback = move || {
                    log::debug!("timer tick");

                    static STATE: AtomicUsize = AtomicUsize::new(1);

                    if STATE.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                        if let Err(err) = chip::run(&mut controller.lock()) {
                            log::error!("Unable to execute the tick <{}>", err);
                            STATE.store(0, std::sync::atomic::Ordering::SeqCst);
                            // clean up given the error state
                            tt.borrow_mut().take();
                        }
                    }
                };

                let mut tt = self.tick_timer.borrow_mut();
                *tt = Some(gloo::timers::callback::Interval::new(2, callback));

                true
            }
            Msg::Keyboard(event, pressed) => {
                // TODO: implement setting of keyboard
                handle_keypress(event, self.controller.lock().keyboard(), pressed);
                false
            }
            Msg::Display => {
                // TODO: update display state, with changes
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let props_rom = self.props.rom.clone();
        let props_field = self.props.field.clone();
        let onkeyup = self.keyboard_callbacks.key_up.clone();
        let onkeydown = self.keyboard_callbacks.key_down.clone();

        // tabindex='0' is need to make the div selectable
        // => so that the key event will fire
        html! {
            <div tabindex ="0" onkeyup = {onkeyup} onkeydown = {onkeydown}>
                <h1>{ "Chip8 Emulator" }</h1>
                <RomDropdown ..props_rom />
                <Field ..props_field />
            </ div>
        }
    }
}

fn handle_keypress(event: yew::KeyboardEvent, ka: &mut KeyboardAdapter, pressed: bool) {
    if event.repeat() {
        return;
    }

    let key = event.code();
    log::debug!("keypress registered <{}>", key);
    if let Some(key) = KeyboardAdapter::map_key(&key) {
        log::debug!(
            "valid keypress registered <{}> - is pressed <{}>",
            key,
            pressed
        );
        ka.set_key(key, pressed);
    }
}

#[derive(Debug)]
struct Props {
    field: FieldProp,
    rom: RomProp,
}

#[derive(Debug, PartialEq, Clone)]
struct Roms {
    files: Rc<[String]>,
    chosen: usize,
}

impl Default for Roms {
    fn default() -> Self {
        // get rom names
        let ra = RomArchives::new();
        let mut files: Vec<_> = ra.file_names().iter().map(|a| a.to_string()).collect();
        files.sort_unstable();

        let files = files.into();
        let chosen = 0;

        Self { files, chosen }
    }
}

#[derive(Debug, PartialEq, Properties, Clone)]
struct RomProp {
    callback: Callback<usize>,
    roms: Roms,
}

#[function_component(RomDropdown)]
fn draw_dropdown(props: &RomProp) -> Html {
    const BASE_CASE: &str = "--";

    let base_case = std::iter::once(BASE_CASE);
    let files = props.roms.files.iter().map(|a| &a[..]);

    let items = Iterator::chain(base_case, files);

    let items = items.enumerate().map(|(i, v)| {
        let selected = i == props.roms.chosen;

        html! {
            <option selected = {selected} > { v } </option>
        }
    });

    let callback = props.callback.clone();

    let callback = move |event: yew::Event| {
        if let Some(input) = event.target_dyn_into::<web_sys::HtmlSelectElement>() {
            let val = input.selected_index();

            log::debug!("the selected input value is <{val}>");

            // ignore no value -1 and base case '--'
            if val <= 0 {
                return;
            }

            // remove base case
            callback.emit((val - 1) as _);
        } else {
            log::warn!("Unable to cast");
        }
    };

    let callback = Callback::from(callback);

    html! {
        <select name = { "files" } onchange = { callback }>
            { for items }
        </select>
    }
}

#[derive(Debug, Properties, Clone)]
struct FieldProp {
    display: Arc<Mutex<DisplayState>>,
}

impl PartialEq for FieldProp {
    fn eq(&self, other: &Self) -> bool {
        // dirty hack to compare the pointer location
        let me = &*self.display as *const _ as *const usize;
        let other = &*other.display as *const _ as *const usize;
        me == other
    }
}

#[function_component(Field)]
fn draw_field(prop: &FieldProp) -> Html {
    use crate::definitions::field;

    let display = prop.display.lock();

    let rows = display.state().iter().map(|row| {
        let columns = row.iter().map(|&state| {
            let state = state.then_some(field::ACTIVE);

            html! {
                <th class={classes!(state)}></th>
            }
        });

        html! {
            <tr>
                { for columns }
            </tr>
        }
    });

    html! {
        <table id = {field::ID}>
            { for rows }
        </table>
    }
}
