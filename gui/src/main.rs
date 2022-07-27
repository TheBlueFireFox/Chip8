mod adapter;
mod definitions;
mod error;
mod model;
mod timer;
mod event_bus;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<model::App>();
}
