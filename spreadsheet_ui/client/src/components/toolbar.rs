use yew::prelude::*;
use web_sys::{Event, File, FileReader, HtmlInputElement,js_sys};
use wasm_bindgen::{JsCast, closure::Closure, JsValue};


#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    pub on_import: Callback<String>,
    pub on_export: Callback<()>,
    pub on_save: Callback<String>,
    pub on_load: Callback<String>,
    pub on_list_spreadsheets: Callback<()>,
}

#[function_component(Toolbar)]
pub fn toolbar(props: &ToolbarProps) -> Html {
    let import_input_ref = use_node_ref();
    let save_input_ref = use_node_ref();
    let load_input_ref = use_node_ref();
    
    let on_import_click = {
        let import_input_ref = import_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = import_input_ref.cast::<HtmlInputElement>() {
                let _ = input.click();
            }
        })
    };
    
    let on_import_change = {
        let on_import = props.on_import.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let file_reader = FileReader::new().unwrap();
                let on_import_clone = on_import.clone();
                
                // Create the callback and store it in a variable
                let onload = Closure::wrap(Box::new(move |e: Event| {
                    // Get the FileReader from the event target instead of capturing it
                    let target: web_sys::EventTarget = e.target().unwrap();
                    let file_reader: FileReader = target.dyn_into().unwrap();
                    let result = file_reader.result().unwrap();
                    let text = result.as_string().unwrap();
                    on_import_clone.emit(text);
                }) as Box<dyn FnMut(_)>);
                
                // Now set the onload callback
                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                let _ = file_reader.read_as_text(&file);
                
                // Keep the closure alive until the file is read
                onload.forget();
            }
        })
    };

    
    
    let on_export_click = {
        let on_export = props.on_export.clone();
        Callback::from(move |_| {
            on_export.emit(());
        })
    };

    let on_save = props.on_save.clone();
    
    let on_save_click = {
        Callback::from(move |_| {
            // Now using on_save which is cloned and owned by this closure
            let name = web_sys::window()
                .unwrap()
                .prompt_with_message("Enter spreadsheet name:")
                .unwrap();
            
            if let Some(name) = name {
                if !name.trim().is_empty() {
                    on_save.emit(name);
                }
            }
        })
    };
    
    let on_load_click = {
        let on_list_spreadsheets = props.on_list_spreadsheets.clone();
        let on_load = props.on_load.clone();
        
        Callback::from(move |_| {
            // First list all spreadsheets
            on_list_spreadsheets.emit(());
            
            // Then show a dialog to select (this is a simplification - ideally you'd show a proper UI)
            let id = web_sys::window()
                .unwrap()
                .prompt_with_message("Enter spreadsheet ID to load:")
                .unwrap();
            
            if let Some(id) = id {
                if !id.trim().is_empty() {
                    on_load.emit(id);
                }
            }
        })
    };

    html! {
        <div class="toolbar">
            <button onclick={on_import_click}>{"Import CSV"}</button>
            <button onclick={on_export_click}>{"Export CSV"}</button>
            <button onclick={on_save_click}>{"Save"}</button>
            <button onclick={on_load_click}>{"Load"}</button>
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
