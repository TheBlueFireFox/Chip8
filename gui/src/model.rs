use std::rc::Rc;

use chip::resources::RomArchives;
use yew::prelude::*;

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
    display: Vec<Vec<bool>>,
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
        Self {
            display: Default::default(),
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
        let props = self.rom_props.clone();
        html! {
            <>
                <RomDropdown ..props />
                <p>{format!("Value is <{}>", self.rom_props.roms.chosen)}</p>
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

#[derive(Debug)]
struct RomDropdown;

impl Component for RomDropdown {
    type Message = ();

    type Properties = RomProp;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        const BASE_CASE: &str = "--";

        let props = ctx.props();

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

        let callback = move |event: Event| {
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
}
