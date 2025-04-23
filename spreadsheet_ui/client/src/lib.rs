pub mod components;
pub mod api;

use components::spreadsheet::SpreadsheetGrid;
use crate::components::toolbar::Toolbar;
use yew::prelude::*;
use web_sys::window;
use spreadsheet_core::wasm::WasmSheet;
use wasm_bindgen::JsValue;
use spreadsheet_core::spreadsheet::Spreadsheet;
use api::ApiClient;
use wasm_bindgen_futures::spawn_local;
use serde::Deserialize;
use gloo_net::http::Request;
use serde_wasm_bindgen::from_value;

#[derive(Deserialize)]
struct SpreadsheetData {
    id: String,
    name: String,
    content: String,
    created_at: u64,
    updated_at: u64,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    status: String,
    data: T,
    message: Option<String>,
}


#[function_component(App)]
fn app() -> Html {
    println!("Into app");
    let spreadsheet = use_state(|| WasmSheet::new(20, 10));
    let current_id = use_state(|| None::<String>);
    let api_client = use_state(|| ApiClient::new(
        "your_authentication_token_here".to_string()  // Provide actual token
    ));
    
    // Import handler
    let on_import = {
        let spreadsheet = spreadsheet.clone();
        let current_id = current_id.clone();
        let api_client = api_client.clone();
        
        Callback::from(move |csv_data: String| {
            let mut new_sheet = (*spreadsheet).clone();
            if let Err(e) = new_sheet.import_csv(&csv_data) {
                web_sys::console::error_1(&e);
                return;
            }
            
            spreadsheet.set(new_sheet.clone());
            
            // If we have a current spreadsheet, update it on the server
            if let Some(id) = (*current_id).clone() {
                let id_clone = id.clone();
                let api_client = api_client.clone();
                spawn_local(async move {
                    match api_client.import_spreadsheet(&id_clone, &csv_data).await {
                        Ok(_) => web_sys::console::log_1(&"Successfully updated spreadsheet on server".into()),
                        Err(e) => web_sys::console::error_1(&e),
                    }
                });
            }
        })
    };
    
    // Export handler
    let on_export = {
        let spreadsheet = spreadsheet.clone();
        Callback::from(move |_| {
            // Trigger download
            let _ = spreadsheet.download_csv("spreadsheet.csv");
        })
    };
    
    // Save handler
    let on_save = {
        let spreadsheet = spreadsheet.clone();
        let current_id = current_id.clone();
        let api_client = api_client.clone();
    
        Callback::from(move |name: String| {
            let csv_data = (*spreadsheet).export_csv();
    
            match &*current_id {
                Some(id) => {
                    // Update existing spreadsheet
                    let id_clone = id.clone();
                    let name_clone = name.clone();
                    let api_client_clone = api_client.clone();
                    let spreadsheet_clone = spreadsheet.clone();
    
                    spawn_local(async move {
                        let csv_content = spreadsheet_clone.export_csv();
    
                        match api_client_clone
                            .update_spreadsheet(&id_clone, Some(name_clone), Some(csv_content))
                            .await
                        {
                            Ok(_) => web_sys::console::log_1(&"Successfully updated spreadsheet".into()),
                            Err(e) => web_sys::console::error_1(&e),
                        }
                    });
                }
                None => {
                    // Create new spreadsheet
                    let name_clone = name.clone();
                    let api_client_clone = api_client.clone();
                    let current_id_clone = current_id.clone();
                    let spreadsheet_clone = spreadsheet.clone();

                    spawn_local(async move {
                        match api_client_clone.create_spreadsheet(&name_clone, 20, 10).await {
                            Ok(response) => {
                                // If response is JsValue (from serde_wasm_bindgen), convert to JSON string
                                let response_obj: serde_json::Value = if let Some(json_str) = response.as_string() {
                                    serde_json::from_str(&json_str).unwrap()
                                } else {
                                    serde_wasm_bindgen::from_value(response).unwrap()
                                };

                                let id = response_obj["data"]["id"].as_str().unwrap().to_string();
                                current_id_clone.set(Some(id.clone()));
                                web_sys::console::log_1(&"Successfully created spreadsheet".into());

                                // --- Immediately update with current content ---
                                let spreadsheet_clone2 = spreadsheet_clone.clone();
                                let api_client_clone2 = api_client_clone.clone();
                                spawn_local(async move {
                                    let csv_content = spreadsheet_clone2.export_csv();
                                    match api_client_clone2.update_spreadsheet(&id, None, Some(csv_content)).await {
                                        Ok(_) => web_sys::console::log_1(&"Spreadsheet content updated after creation".into()),
                                        Err(e) => web_sys::console::error_1(&e),
                                    }
                                });
                            }
                            Err(e) => web_sys::console::error_1(&e),
                        }
                    });
                }
            }
        })
    };
    
    
    // Load handler
    let on_load_by_id = {
        let spreadsheet = spreadsheet.clone();
        Callback::from(move |id: String| {
            let spreadsheet = spreadsheet.clone();
            spawn_local(async move {
                let url = format!("/api/spreadsheets/{}", id);
                let resp = match Request::get(&url).send().await {
                    Ok(r) => r,
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to fetch spreadsheet: {:?}", e).into());
                        return;
                    }
                };
                let json: Result<ApiResponse<SpreadsheetData>, _> = resp.json().await;
                match json {
                    Ok(api_resp) => {
                        let csv_content = api_resp.data.content;
                        let mut new_sheet = (*spreadsheet).clone();
                        if let Err(e) = new_sheet.import_csv(&csv_content) {
                            web_sys::console::error_1(&format!("Import error: {:?}", e).into());
                        } else {
                            spreadsheet.set(new_sheet);
                            web_sys::console::log_1(&"Spreadsheet loaded successfully!".into());
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("JSON parse error: {:?}", e).into());
                    }
                }
            });
        })
    };

    let on_load_by_name = {
        let spreadsheet = spreadsheet.clone();
        let api_client = api_client.clone();
        Callback::from(move |name: String| {
            let spreadsheet = spreadsheet.clone();
            let api_client = api_client.clone();
            spawn_local(async move {
                match api_client.get_spreadsheet_by_name(&name).await {
                    Ok(js_value) => {
                        // Deserialize the API response wrapper, not just SpreadsheetData
                        let api_response: ApiResponse<Option<SpreadsheetData>> = from_value(js_value).unwrap();
                        if let Some(data) = api_response.data {
                            let mut new_sheet = (*spreadsheet).clone();
                            if let Err(e) = new_sheet.import_csv(&data.content) {
                                web_sys::console::error_1(&format!("Import error: {:?}", e).into());
                            } else {
                                spreadsheet.set(new_sheet);
                                web_sys::console::log_1(&"Spreadsheet loaded by name!".into());
                            }
                        } else {
                            web_sys::console::error_1(&"No spreadsheet data returned from API".into());
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Load by name error: {:?}", e).into());
                    }
                }
            });
        })
    };
    
    
    
    let on_cell_change = {
        let spreadsheet = spreadsheet.clone();
        let current_id = current_id.clone();
        let api_client = api_client.clone();
        
        Callback::from(move |(row, col, value): (usize, usize, String)| {
            let mut new_sheet = (*spreadsheet).clone();
            
            // Build A1-style reference
            let cell_ref = format!(
                "{}{}",
                Spreadsheet::column_index_to_label(col),
                row + 1
            );
            
            if let Err(e) = new_sheet.assign(&cell_ref, &value.trim()) {
                web_sys::console::error_1(&JsValue::from_str(&format!("Error: assignment to {} failed", cell_ref)));
            }
            
            spreadsheet.set(new_sheet);
            
            // If we have a current spreadsheet ID, update the cell on the server
            if let Some(id) = (*current_id).clone() {
                let id_clone = id.clone();
                let value_clone = value.clone();
                let api_client = api_client.clone();
                
                spawn_local(async move {
                    match api_client.update_single_cell(&id_clone, row, col, value_clone).await {
                        Ok(_) => {},
                        Err(e) => web_sys::console::error_1(&e),
                    }
                });
            }
        })
    };
    
    // Add a button to list all spreadsheets
    let on_list_spreadsheets = {
        let api_client = api_client.clone();
        
        Callback::from(move |_| {
            let api_client_clone = api_client.clone();
            
            spawn_local(async move {
                match api_client_clone.list_spreadsheets().await {
                    Ok(response) => {
                        web_sys::console::log_1(&response);
                        // You could update a state here to display the list in the UI
                    },
                    Err(e) => web_sys::console::error_1(&e),
                }
            });
        })
    };

    html! {
        <div class="app-container">
            <Toolbar 
                on_import={on_import}
                on_export={on_export}
                on_save={on_save}
                on_load_by_name={on_load_by_name.clone()}
                on_list_spreadsheets={on_list_spreadsheets}
            />
            <SpreadsheetGrid 
                rows={20}
                cols={10}
                spreadsheet={(*spreadsheet).clone()}
                on_change={on_cell_change}
                on_load_by_name={on_load_by_name.clone()}
            />
        </div>
    }
}

use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen(start)]
pub fn run() {
    web_sys::console::log_1(&"Starting Yew app...".into());

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document.get_element_by_id("root").unwrap();
    yew::Renderer::<App>::with_root(root).render();
}

