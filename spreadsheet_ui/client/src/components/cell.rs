use yew::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Properties, PartialEq)]
pub struct CellProps {
    pub row: usize,
    pub col: usize,
    pub value: i32,
    pub content: String,
    pub is_formula: bool,
    pub has_error: bool,
    pub on_change: Callback<(usize, usize, String)>,
    pub is_selected: bool,
    pub on_select: Callback<(usize, usize)>,
}

// ... (imports and props)

#[function_component(Cell)]
pub fn cell(props: &CellProps) -> Html {
    let is_editing = use_state(|| false);
    let input_ref = use_node_ref();

    let onclick = {
        let on_select = props.on_select.clone();
        let row = props.row;
        let col = props.col;
        Callback::from(move |_| {
            on_select.emit((row, col));
        })
    };

    let ondoubleclick = {
        let is_editing = is_editing.clone();
        Callback::from(move |_| {
            is_editing.set(true);
        })
    };

    let onblur = {
        let is_editing = is_editing.clone();
        let on_change = props.on_change.clone();
        let input_ref = input_ref.clone();
        let row = props.row;
        let col = props.col;

        Callback::from(move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let value = input.value();
                on_change.emit((row, col, value));
            }
            is_editing.set(false);
        })
    };

    let onkeydown = {
        let is_editing = is_editing.clone();
        let on_change = props.on_change.clone();
        let input_ref = input_ref.clone();
        let row = props.row;
        let col = props.col;

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                    let value = input.value();
                    on_change.emit((row, col, value));
                }
                is_editing.set(false);
            }
        })
    };

    let display_value = if props.has_error {
        "ERR".to_string()
    } else if *is_editing {
        props.content.clone()
    } else {
        props.value.to_string()
    };

    let cell_classes = classes!(
        "cell",
        if props.is_selected { Some("selected") } else { None },
        if props.is_formula { Some("formula") } else { None },
        if props.has_error { Some("error") } else { None },
    );

    html! {
        <div class={cell_classes} onclick={onclick} ondblclick={ondoubleclick}>
            {
                if *is_editing {
                    html! {
                        <input
                            ref={input_ref}
                            type="text"
                            class="cell-input"
                            value={display_value}
                            autofocus=true
                            onblur={onblur}
                            onkeydown={onkeydown}
                        />
                    }
                } else {
                    html! {
                        <div class="cell-value">
                            { display_value }
                        </div>
                    }
                }
            }
        </div>
    }
}
