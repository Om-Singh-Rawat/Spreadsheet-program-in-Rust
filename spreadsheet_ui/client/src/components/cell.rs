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
    pub on_edit_start: Callback<()>, 
    pub on_edit_end: Callback<()>,   
}

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


    // Create focus ref for selected cells
    let cell_ref = use_node_ref();
    
    // Effect to focus cell when selected
use_effect_with_deps(
    |(is_selected, cell_ref, is_editing)| {
        if *is_selected && !**is_editing {
            if let Some(element) = cell_ref.cast::<web_sys::HtmlElement>() {
                let _ = element.focus();
                
                // Return a boxed closure
                return Box::new(|| {
                    web_sys::console::log_1(&"Cell focus effect cleaned up".into());
                }) as Box<dyn FnOnce()>;
            }
        }
        
        // Return a different boxed closure with the same type
        Box::new(|| web_sys::console::log_1(&"Cell effect ran but no action taken".into())) as Box<dyn FnOnce()>
    },
    (props.is_selected, cell_ref.clone(), is_editing.clone())
);

// Effect to focus input when editing begins
use_effect_with_deps(
    |(is_editing, input_ref)| {
        if **is_editing {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
                input.select();
                
                // Return a boxed closure
                return Box::new(|| {
                    web_sys::console::log_1(&"Input focus effect cleaned up".into());
                }) as Box<dyn FnOnce()>;
            }
        }
        
        // Return a different boxed closure with the same type
        Box::new(|| web_sys::console::log_1(&"Input effect ran but no action taken".into())) as Box<dyn FnOnce()>
    },
    (is_editing.clone(), input_ref.clone())
);


    
    let ondblclick = {
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
        <div class={cell_classes}
        onclick={onclick}
        ondblclick={ondblclick}
        ref={cell_ref}
        tabindex={if props.is_selected { "0" } else { "-1" }}>
            {
                if *is_editing {
                    html! {
                        <input
                            ref={input_ref}
                            type="text"
                            class="cell-editor"
                            value={props.content.clone()}
                            autofocus=true
                            onblur={onblur}
                            onkeydown={onkeydown}
                        />
                    }
                } else {
                    html! {
                        <span class={if props.is_formula { "formula-value" } else { "" }}>
                            { display_value }
                        </span>
                    }
                }
            }
        </div>
    }
}
