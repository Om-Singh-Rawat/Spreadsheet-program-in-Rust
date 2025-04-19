
use yew::prelude::*;
use web_sys::{File, FileReader, Event, HtmlInputElement};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::js_sys;

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    pub on_import: Callback<String>,
    pub on_export: Callback<()>,
}

#[function_component(Toolbar)]
pub fn toolbar(props: &ToolbarProps) -> Html {
    let file_input_ref = use_node_ref();

    let on_import_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                let _ = input.click();
            }
        })
    };

    let on_export_click = {
        let on_export = props.on_export.clone();
        Callback::from(move |_: MouseEvent| {
            on_export.emit(());
        })
    };

    let on_file_change = {
        let on_import = props.on_import.clone();
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: Event| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        let reader = FileReader::new().unwrap();
                        let on_import = on_import.clone();
                        let input_clone = input.clone();
                        
                        let onload = Closure::wrap(Box::new(move |e: Event| {
                            let target = e.target().unwrap();
                            let reader = target.dyn_into::<FileReader>().unwrap();
                            let result = reader.result().unwrap();
                            let text = result.as_string().unwrap();
                            
                            // Clear input to allow re-importing the same file
                            input_clone.set_value("");
                            on_import.emit(text);
                        }) as Box<dyn FnMut(_)>);
                        
                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                        onload.forget(); // Prevent closure from being dropped
                        reader.read_as_text(&file).unwrap();
                    }
                }
            }
        })
    };
    

    html! {
        <div class="toolbar">
            <input 
                type="file" 
                ref={file_input_ref.clone()} 
                style="display: none;" 
                accept=".csv" 
                onchange={on_file_change}
            />
            <button onclick={on_import_click}>{"Import CSV"}</button>
            <button onclick={on_export_click}>{"Export CSV"}</button>
        </div>
    }
}