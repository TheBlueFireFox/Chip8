use std::{cell::RefCell, rc::Rc};

use chip::resources::RomArchives;
use yew::{
    classes, function_component, html, Callback, Component, Context, Html, Properties, TargetCast,
};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
            <h1>{ "Chip8 Emulator" }</h1>
            <State/>
        </>
    }
}

enum Msg {
    Roms(usize),
}

struct State {
    display: Rc<RefCell<Vec<Vec<bool>>>>,
    rom_props: RomProp,
}

impl Component for State {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|index: usize| Msg::Roms(index));

        let props = RomProp {
            callback,
            roms: Default::default(),
        };

        use chip::definitions::display;

        let map = (0..display::WIDTH)
            .map(|y| (0..display::HEIGHT).map(|x| (y + x) % 2 == 0).collect())
            .collect();

        Self {
            display: Rc::new(RefCell::new(map)),
            rom_props: props,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roms(new) => {
                /* update state */
                self.rom_props.roms.chosen = new;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let props_rom = self.rom_props.clone();
        let props_field = FieldProp {
            display: self.display.clone(),
        };

        html! {
            <>
                <RomDropdown ..props_rom />
                <p>{format!("Value is <{}>", self.rom_props.roms.chosen)}</p>
                <Field ..props_field />
            </>
        }
    }
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

#[derive(Debug, PartialEq, Properties)]
struct FieldProp {
    display: Rc<RefCell<Vec<Vec<bool>>>>,
}

#[function_component(Field)]
fn draw_field(prop: &FieldProp) -> Html {
    let display = prop.display.borrow();

    let rows = display.iter().map(|row| {
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

/// The board in which the chip implementation runs.
pub mod field {
    /// The upper most id.
    pub const ID: &str = "board";

    /// The state of which the values exist on.
    /// Attention the implemtnation is in reverse, so a not `active` cell is per this definition
    /// `alive`.
    pub const ACTIVE: &str = "alive";
}
