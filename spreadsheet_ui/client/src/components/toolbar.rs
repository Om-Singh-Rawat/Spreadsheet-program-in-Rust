use yew::prelude::*;
use web_sys::{Event, File, FileReader, HtmlInputElement, HtmlElement};
use wasm_bindgen::{JsCast, closure::Closure, JsValue};

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    pub on_import: Callback<String>,
    pub on_export: Callback<()>,
    pub on_save: Callback<String>,
    pub on_load_by_name: Callback<String>,
    pub on_list_spreadsheets: Callback<()>,
}

#[function_component(Toolbar)]
pub fn toolbar(props: &ToolbarProps) -> Html {
    let import_input_ref = use_node_ref();
    let export_button_ref = use_node_ref();
    let save_button_ref = use_node_ref();
    let load_button_ref = use_node_ref();

    // Import button handler
    let on_import_click = {
        let import_input_ref = import_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = import_input_ref.cast::<HtmlInputElement>() {
                let _ = input.click();
            }
        })
    };

    // File input change handler
    let on_import_change = {
        let on_import = props.on_import.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let file_reader = FileReader::new().unwrap();
                let on_import_clone = on_import.clone();

                let onload = Closure::wrap(Box::new(move |e: Event| {
                    let target: web_sys::EventTarget = e.target().unwrap();
                    let file_reader: FileReader = target.dyn_into().unwrap();
                    if let Ok(result) = file_reader.result() {
                        if let Some(text) = result.as_string() {
                            on_import_clone.emit(text);
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                let _ = file_reader.read_as_text(&file);
                onload.forget();
            }
        })
    };

    // Export button effect
    use_effect_with_deps(
        |(export_ref, on_export)| {
            let button = match export_ref.cast::<HtmlElement>() {
                Some(b) => b,
                None => return Box::new(|| {}) as Box<dyn FnOnce()>,
            };

            let on_export = on_export.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                on_export.emit(());
            });

            button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();

            Box::new(move || {
                button.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }) as Box<dyn FnOnce()>
        },
        (export_button_ref.clone(), props.on_export.clone()),
    );

    // Save button effect
    use_effect_with_deps(
        |(save_ref, on_save)| {
            let button = match save_ref.cast::<HtmlElement>() {
                Some(b) => b,
                None => return Box::new(|| {}) as Box<dyn FnOnce()>,
            };

            let on_save = on_save.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                let name = web_sys::window()
                    .unwrap()
                    .prompt_with_message("Enter spreadsheet name:")
                    .unwrap();

                if let Some(name) = name.filter(|n| !n.trim().is_empty()) {
                    on_save.emit(name);
                }
            });

            button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();

            Box::new(move || {
                button.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }) as Box<dyn FnOnce()>
        },
        (save_button_ref.clone(), props.on_save.clone()),
    );

    // Load button effect
    use_effect_with_deps(
        |(load_ref, on_load, on_list)| {
            let button = match load_ref.cast::<HtmlElement>() {
                Some(b) => b,
                None => return Box::new(|| {}) as Box<dyn FnOnce()>,
            };

            let on_load = on_load.clone();
            let on_list = on_list.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                on_list.emit(());
                let name = web_sys::window()
                    .unwrap()
                    .prompt_with_message("Enter spreadsheet name to load:")
                    .unwrap();

                if let Some(name) = name.filter(|n| !n.trim().is_empty()) {
                    on_load.emit(name);
                }
            });

            button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();

            Box::new(move || {
                button.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();
            }) as Box<dyn FnOnce()>
        },
        (load_button_ref.clone(), props.on_load_by_name.clone(), props.on_list_spreadsheets.clone()),
    );

    html! {
        <div class="toolbar">
            <button onclick={on_import_click}>{"Import CSV"}</button>
            <button ref={export_button_ref}>{"Export CSV"}</button>
            <button ref={save_button_ref}>{"Save"}</button>
            <button ref={load_button_ref}>{"Load"}</button>
            <input 
                type="file" 
                accept=".csv" 
                ref={import_input_ref}
                onchange={on_import_change}
                class="hidden-file-input" 
            />
        </div>
    }
}
