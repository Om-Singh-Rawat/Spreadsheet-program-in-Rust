// client/src/components/status_bar.rs
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusBarProps {
    pub selected_cell: Option<(usize, usize)>,
    pub formula: Option<String>,
    pub status: String,
}

#[function_component(StatusBar)]
pub fn status_bar(props: &StatusBarProps) -> Html {
    let cell_ref = match &props.selected_cell {
        Some((row, col)) => format!(
            "{}{}",
            core::spreadsheet::Spreadsheet::column_index_to_label(*col),
            row + 1
        ),
        None => "".to_string(),
    };

    html! {
        <div class="status-bar">
            <div class="cell-ref">{"Cell: "}{cell_ref}</div>
            <div class="formula">
                {"Formula: "}{ props.formula.clone().unwrap_or_default() }
            </div>
            <div class="status">{ &props.status }</div>
        </div>
    }
}
