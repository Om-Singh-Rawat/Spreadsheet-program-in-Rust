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
    let selected_cell = use_state(|| (0, 0));
    let cell_values = use_state(|| HashMap::<(usize, usize), String>::new());

    // callback to update both the displayed text and the underlying Spreadsheet model
    let on_cell_change = {
        let spreadsheet = spreadsheet.clone();
        let cell_values = cell_values.clone();
        Callback::from(move |(row, col, value): (usize, usize, String)| {
            // update our local cell_values state
            let mut current = (*cell_values).clone();
            current.insert((row, col), value.clone());
            cell_values.set(current);

            // feed the input string (e.g. "A1=42") into the core model
            let mut sheet = (*spreadsheet).clone();
            let input = format!(
                "{}{}={}",
                Spreadsheet::column_index_to_label(col),
                row + 1,
                value
            );
            let _ = sheet.process_input(&input);
            spreadsheet.set(sheet);
        })
    };

    html! {
        <div class="spreadsheet-container">
            <div class="header-row">
                <div class="corner-cell"></div>
                {
                    (0..props.cols).map(|col| {
                        let label = Spreadsheet::column_index_to_label(col);
                        html! { <div class="header-cell">{ label }</div> }
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
                                    let value = (*cell_values)
                                        .get(&(row, col))
                                        .cloned()
                                        .unwrap_or_default();
                                    let selected = *selected_cell == (row, col);
                                    let on_change = on_cell_change.clone();
                                    let select_this = {
                                        let selected_cell = selected_cell.clone();
                                        Callback::from(move |_| {
                                            selected_cell.set((row, col));
                                        })
                                    };

                                    html! {
                                        <Cell
                                            row={row}
                                            col={col}
                                            value={value}
                                            selected={selected}
                                            on_change={on_change}
                                            onclick={select_this}
                                        />
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}
