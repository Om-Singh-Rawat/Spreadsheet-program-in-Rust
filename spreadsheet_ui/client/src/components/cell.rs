// client/src/components/cell.rs
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CellProps {
    pub row: usize,
    pub col: usize,
    pub value: String,
    pub selected: bool,
    pub on_change: Callback<(usize, usize, String)>,
}

#[function_component(Cell)]
pub fn cell(props: &CellProps) -> Html {
    let is_editing = use_state(|| false);
    let input_ref = use_node_ref();

    // Handle cell click to start editing
    let onclick = {
        let is_editing = is_editing.clone();
        Callback::from(move |_| {
            is_editing.set(true);
        })
    };

    // Handle input blur to save changes
    let onblur = {
        let is_editing = is_editing.clone();
        let on_change = props.on_change.clone();
        let input_ref = input_ref.clone();
        let row = props.row;
        let col = props.col;

        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                let value = input.value();
                on_change.emit((row, col, value));
            }
            is_editing.set(false);
        })
    };

    html! {
        <div 
            class={classes!("cell", props.selected.then(|| "selected"))} 
            {onclick}
        >
            if *is_editing {
                <input 
                    ref={input_ref}
                    type="text"
                    class="cell-input"
                    value={props.value.clone()}
                    autofocus=true
                    onblur={onblur}
                />
            } else {
                <div class="cell-value">{&props.value}</div>
            }
        </div>
    }
}
