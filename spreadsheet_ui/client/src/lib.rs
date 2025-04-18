pub mod components;
use components::spreadsheet::SpreadsheetGrid;

use yew::prelude::*;
use web_sys::window;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="app-container">
            <h1>{"Spreadsheet App"}</h1>
            <SpreadsheetGrid rows={20} cols={10} />
        </div>
    }
}

use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen(start)]
pub fn run() {
    web_sys::console::log_1(&"Starting Yew app...".into());

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document.get_element_by_id("root").unwrap();
    yew::Renderer::<App>::with_root(root).render();
}