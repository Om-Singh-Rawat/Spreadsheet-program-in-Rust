// client/src/components/spreadsheet.rs
use crate::components::cell::Cell;
use yew::prelude::*;
use core::spreadsheet::Spreadsheet;
use std::collections::HashMap;

#[derive(Properties, PartialEq)]
pub struct SpreadsheetProps {
    pub rows: usize,
    pub cols: usize,
}

#[function_component(SpreadsheetGrid)]
pub fn spreadsheet_grid(props: &SpreadsheetProps) -> Html {
    let spreadsheet = use_state(|| Spreadsheet::new(props.rows, props.cols));

    // Create a state to track the selected cell
    // and a state to hold the cell values
    let selected_cell = use_state(|| (0, 0));
    let cell_values = use_state(|| HashMap::<(usize, usize), String>::new());
    
    let on_cell_change = {
        let spreadsheet = spreadsheet.clone();
        let cell_values = cell_values.clone();
        
        Callback::from(move |(row, col, value): (usize, usize, String)| {
            let mut current_values = (*cell_values).clone();
            current_values.insert((row, col), value.clone());
            cell_values.set(current_values);
            
            let mut sheet = (*spreadsheet).clone();
            let input = format!("{}{}={}", 
                Spreadsheet::column_index_to_label(col), 
                row + 1, 
                value);
            
            let _ = sheet.process_input(&input);
            spreadsheet.set(sheet);
        })
    };


    // Render the spreadsheet grid
    html! {
        <div class="spreadsheet-container">
            <div class="header-row">
                <div class="corner-cell"></div>
                {
                    (0..props.cols).map(|col| {
                        let col_label = Spreadsheet::column_index_to_label(col);
                        html! { <div class="header-cell">{ col_label }</div> }
                    }).collect::<Html>()
                }
            </div>
            {
                (0..props.rows).map(|row| {
                    html! {
                        <div class="grid-row">
                            <div class="row-header">{ row + 1 }</div>
                            {
                                (0..props.cols).map(|col| {
                                    let value = spreadsheet.get_cell_value(row, col).unwrap_or(0);
                                    html! { <div class="grid-cell">{ value }</div> }
                                }).collect::<Html>()
                            }
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}
