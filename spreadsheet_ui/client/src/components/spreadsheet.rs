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
    pub spreadsheet: WasmSheet,  // Add this to receive the spreadsheet from parent
    pub on_change: Callback<(usize, usize, String)>,  // Add callback to update parent's state
}

#[function_component(SpreadsheetGrid)]
pub fn spreadsheet_grid(props: &SpreadsheetProps) -> Html {
    
    let selected_cell = use_state(|| (0, 0));     // tracks which cell is clicked/active.
    let status = use_state(|| "Ready".to_string()); 
    let is_editing = use_state(|| false); // Track if a cell is being edited
    

    let on_cell_change = props.on_change.clone();
    // Handle cell selection
    let select_cell = {
        let selected_cell = selected_cell.clone();
        Callback::from(move |(row, col): (usize, usize)| {
            selected_cell.set((row, col));
        })
    };

    let formula = {
        let (row, col) = *selected_cell;
        Some(props.spreadsheet.get_content(row, col))
    };

    let onkeydown = {
        let selected_cell = selected_cell.clone();
        let is_editing = is_editing.clone();
        let rows = props.rows;
        let cols = props.cols;
        let status = status.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            // Only handle navigation when not editing a cell
            if !*is_editing {
                let (row, col) = *selected_cell;
                
                match e.key().as_str() {
                    "ArrowUp" => {
                        if row > 0 {
                            selected_cell.set((row - 1, col));
                            status.set(format!("Moved to cell {}{}", 
                                Spreadsheet::column_index_to_label(col), row));
                            e.prevent_default();
                        }
                    },
                    "ArrowDown" => {
                        if row < rows - 1 {
                            selected_cell.set((row + 1, col));
                            status.set(format!("Moved to cell {}{}", 
                                Spreadsheet::column_index_to_label(col), row + 2));
                            e.prevent_default();
                        }
                    },
                    "ArrowLeft" => {
                        if col > 0 {
                            selected_cell.set((row, col - 1));
                            status.set(format!("Moved to cell {}{}", 
                                Spreadsheet::column_index_to_label(col - 1), row + 1));
                            e.prevent_default();
                        }
                    },
                    "ArrowRight" => {
                        if col < cols - 1 {
                            selected_cell.set((row, col + 1));
                            status.set(format!("Moved to cell {}{}", 
                                Spreadsheet::column_index_to_label(col + 1), row + 1));
                            e.prevent_default();
                        }
                    },
                    "Enter" => {
                        // Enter edit mode for the currently selected cell
                        is_editing.set(true);
                        status.set(format!("Editing cell {}{}", 
                            Spreadsheet::column_index_to_label(col), row + 1));
                        e.prevent_default();
                    },
                    _ => {}
                }
            }
        })
    };
    
    // Update Cell props to notify when editing begins
    let on_edit_start = {
        let is_editing = is_editing.clone();
        Callback::from(move |_| {
            is_editing.set(true);
        })
    };
    
    // Update Cell props to notify when editing ends
    let on_edit_end = {
        let is_editing = is_editing.clone();
        Callback::from(move |_| {
            is_editing.set(false);
        })
    };


    html! {
        <div class="spreadsheet-container">
            <StatusBar
                selected_cell={Some(*selected_cell)}
                formula={formula}
                status={(*status).clone()}
            />
            <div 
                class="grid-container" 
                tabindex="0" 
                onkeydown={onkeydown}
            >
                <table class="spreadsheet">
                    <thead>
                        <tr>
                            <th></th>
                            {
                                (0..props.cols).map(|col| {
                                    let label = Spreadsheet::column_index_to_label(col);
                                    html! { <th>{ label }</th> }
                                }).collect::<Html>()
                            }
                        </tr>
                    </thead>
                    <tbody>
                        {
                            (0..props.rows).map(|row| {
                                html! {
                                    <tr>
                                        <th>{ row + 1 }</th>
                                        {
                                            (0..props.cols).map(|col| {
                                                let value = props.spreadsheet.get(row, col);
                                                let content = props.spreadsheet.get_content(row, col);
                                                let is_formula = props.spreadsheet.is_formula(row, col);
                                                let has_error = props.spreadsheet.get_error(row, col).is_some();
                                                html! {
                                                    <td>
                                                        <Cell
                                                            row={row}
                                                            col={col}
                                                            value={value}
                                                            content={content}
                                                            is_formula={is_formula}
                                                            has_error={has_error}
                                                            on_change={on_cell_change.clone()}
                                                            is_selected={*selected_cell == (row, col)}
                                                            on_select={select_cell.clone()}
                                                            on_edit_start={on_edit_start.clone()}
                                                            on_edit_end={on_edit_end.clone()}
                                                        />
                                                    </td>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </tr>
                                }
                            }).collect::<Html>()
                        }
                    </tbody>
                </table>
            </div>
        </div>
    }
} 