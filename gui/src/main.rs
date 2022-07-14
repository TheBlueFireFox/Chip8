mod model;
mod adapter;
mod error;
mod timer;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<model::App>();
}
