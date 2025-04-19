pub mod components;
use components::spreadsheet::SpreadsheetGrid;
use crate::components::toolbar::Toolbar;
use yew::prelude::*;
use web_sys::window;
use core::wasm::WasmSheet;
use wasm_bindgen::JsValue;
use core::spreadsheet::Spreadsheet;

#[function_component(App)]
fn app() -> Html {

    let spreadsheet = use_state(|| WasmSheet::new(20, 10));
    
    // Import handler
    let on_import = {
        let spreadsheet = spreadsheet.clone();
        Callback::from(move |csv_data: String| {
            let mut new_sheet = (*spreadsheet).clone();
            if let Err(e) = new_sheet.import_csv(&csv_data) {
                web_sys::console::error_1(&e);
            }
            spreadsheet.set(new_sheet);
        })
    };
    
    // Export handler
    let on_export = {
        let spreadsheet = spreadsheet.clone();
        Callback::from(move |_| {
            // Trigger download
            let _ = spreadsheet.download_csv("spreadsheet.csv");
        })
    };


    let on_cell_change = {
        let spreadsheet = spreadsheet.clone();
        Callback::from(move |(row, col, value): (usize, usize, String)| {
            let mut new_sheet = (*spreadsheet).clone();
            // Build A1-style reference
            let cell_ref = format!(
                "{}{}",
                core::spreadsheet::Spreadsheet::column_index_to_label(col),
                row + 1
            );
            if let Err(e) = new_sheet.assign(&cell_ref, &value.trim()) {
                web_sys::console::error_1(&JsValue::from_str(&format!("Error: assignment to {} failed", cell_ref)));
            }
            spreadsheet.set(new_sheet);
        })
    };

    html! {
        <div class="app-container">
        <Toolbar 
        on_import = {on_import} 
        on_export = {on_export} />
            <h1>{"Spreadsheet App"}</h1>
            <SpreadsheetGrid 
                spreadsheet={(*spreadsheet).clone()}
                on_change={on_cell_change}
                rows={20} 
                cols={10} 
            />
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