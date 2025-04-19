use crate::components::cell::Cell;
use crate::components::status_bar::StatusBar;
use core::spreadsheet::Spreadsheet;
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use core::wasm::WasmSheet;  

#[derive(Properties, PartialEq)]
pub struct SpreadsheetProps {
    pub rows: usize,
    pub cols: usize,
}

#[function_component(SpreadsheetGrid)]
pub fn spreadsheet_grid(props: &SpreadsheetProps) -> Html {
    let spreadsheet = use_state(|| WasmSheet::new(props.rows, props.cols));  // the actual spreadsheet engine (WasmSheet from Rust).
    let selected_cell = use_state(|| (0, 0));     // tracks which cell is clicked/active.
    let status = use_state(|| "Ready".to_string()); 

    let on_cell_change = {
        let spreadsheet = spreadsheet.clone();
        
        Callback::from(move |(row, col, value): (usize, usize, String)| {

            web_sys::console::log_1(&format!(
                "on_cell_change → cell {}{} = `{}`",
                Spreadsheet::column_index_to_label(col),
                row + 1,
                value
            ).into());

            // Build A1-style reference (e.g., "B1")
            let cell_ref = format!(
                "{}{}",
                Spreadsheet::column_index_to_label(col),
                row + 1,
            );
    
            let mut new_sheet = (*spreadsheet).clone();
    
            let trimmed_value = value.trim();

            if let Err(e) = new_sheet.assign(&cell_ref, &trimmed_value) {
                web_sys::console::error_1(
                    &JsValue::from_str(&format!("Error processing input '{}={}': {:?}", 
                        cell_ref, trimmed_value, e))
                );
            }
    
            spreadsheet.set(new_sheet);
        })
    };
    // Handle cell selection
    let select_cell = {
        let selected_cell = selected_cell.clone();
        Callback::from(move |(row, col): (usize, usize)| {
            selected_cell.set((row, col));
        })
    };

    let formula = {
        let (row, col) = *selected_cell;
        Some(spreadsheet.get_content(row, col))
    };

    html! {
        <div class="spreadsheet-container">
        <StatusBar
                selected_cell={Some(*selected_cell)}
                formula={formula}
                status={(*status).clone()}
            />
            <div class="grid-wrapper">
                <div class="header-row">
                    <div class="corner-cell"></div>
                    // Column headers (A, B, C, ...)
                    { (0..props.cols).map(|col| {
                        let label = Spreadsheet::column_index_to_label(col);
                        html! { <div class="header-cell">{ label }</div> }
                    }).collect::<Html>() }
                </div>
                { (0..props.rows).map(|row| {
                    html! {
                        <div class="grid-row" key={row}>
                            <div class="row-header">{ row + 1 }</div>
                            { (0..props.cols).map(|col| {
                                let has_error = spreadsheet.get_error(row, col).is_some();
                                html! {
                                    <Cell
                                        key={format!("{}-{}", row, col)}
                                        row={row}
                                        col={col}
                                        value={spreadsheet.get(row, col)}
                                        content={spreadsheet.get_content(row, col)}
                                        is_formula={spreadsheet.is_formula(row, col)}
                                        has_error={has_error}
                                        on_change={on_cell_change.clone()}
                                        is_selected={*selected_cell == (row, col)}
                                        on_select={select_cell.clone()}
                                    />
                                }
                            }).collect::<Html>() }
                        </div>
                    }
                }).collect::<Html>() }
            </div>
            
        </div>
    }    
}
