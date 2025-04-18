use yew::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Properties, PartialEq)]
pub struct CellProps {
    pub row: usize,
    pub col: usize,
    pub value: String,
    pub selected: bool,
    pub on_change: Callback<(usize, usize, String)>,
    pub onclick: Callback<MouseEvent>,
}

#[function_component(Cell)]
pub fn cell(props: &CellProps) -> Html {
    let is_editing = use_state(|| false);
    let input_ref = use_node_ref();

    // when the cell is clicked: fire parent’s onclick, then go into edit mode
    let onclick_cell = {
        let is_editing = is_editing.clone();
        let onclick = props.onclick.clone();
        Callback::from(move |e: MouseEvent| {
            onclick.emit(e);
            is_editing.set(true);
        })
    };

    // when input loses focus: read value and bubble up on_change
    let onblur = {
        let is_editing = is_editing.clone();
        let on_change = props.on_change.clone();
        let input_ref = input_ref.clone();
        let row = props.row;
        let col = props.col;

        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let val = input.value();
                on_change.emit((row, col, val));
            }
            is_editing.set(false);
        })
    };

    html! {
        <div
            class={ classes!("cell", props.selected.then(|| "selected")) }
            onclick={onclick_cell}
        >
            {
                if *is_editing {
                    html! {
                        <input
                            ref={input_ref}
                            type="text"
                            class="cell-input"
                            value={props.value.clone()}
                            autofocus=true
                            onblur={onblur}
                        />
                    }
                } else {
                    html! { <div class="cell-value">{ &props.value }</div> }
                }
            }
        </div>
    }
}
