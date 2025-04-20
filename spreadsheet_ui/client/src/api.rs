use std::borrow::BorrowMut;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen;


#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    status: String,
    data: Option<T>,
    message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SpreadsheetMetadata {
    id: String,
    name: String,
    created_at: u64,
    updated_at: u64,
}

#[derive(Serialize, Deserialize)]
struct SpreadsheetData {
    id: String,
    name: String,
    content: String,
    created_at: u64,
    updated_at: u64,
}

#[derive(Serialize, Deserialize)]
struct NewSpreadsheet {
    name: String,
    rows: usize,
    cols: usize,
}

#[derive(Serialize, Deserialize)]
struct UpdateSpreadsheet {
    name: Option<String>,
    content: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CellUpdate {
    row: usize,
    col: usize,
    value: String,
}

#[derive(Serialize, Deserialize)]
struct CellUpdates {
    cells: Vec<CellUpdate>,
}

#[wasm_bindgen]
pub struct ApiClient {
    base_url: String,
}

#[wasm_bindgen]
impl ApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            base_url: "/api".to_string(),
        }
    }

    // This method isn't exposed to JS directly
    async fn fetch<T: for<'de> Deserialize<'de>>(&self, url: &str, method: &str, body: Option<&JsValue>) -> Result<T, JsValue> {
        let mut opts = RequestInit::new();
        opts.method(method);
        opts.mode(RequestMode::Cors);
        
        if let Some(b) = body {
            opts.body(Some(b));
        }
        
        let request = Request::new_with_str_and_init(&format!("{}{}", self.base_url, url), &opts)?;
        
        if method != "GET" {
            request.headers().set("Content-Type", "application/json")?;
        }
        
        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into().unwrap();
        
        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP Error: {}", resp.status())));
        }
        
        let json = JsFuture::from(resp.json()?).await?;
        let result: T = serde_wasm_bindgen::from_value(json)?;
        
        Ok(result)
    }

    // Exposed methods using JS-friendly types
    #[wasm_bindgen]
    pub async fn list_spreadsheets(&self) -> Result<JsValue, JsValue> {
        let response: ApiResponse<Vec<SpreadsheetMetadata>> = self.fetch("/spreadsheets", "GET", None).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    #[wasm_bindgen]
    pub async fn get_spreadsheet(&self, id: &str) -> Result<JsValue, JsValue> {
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}", id), "GET", None).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    #[wasm_bindgen]
    pub async fn create_spreadsheet(&self, name: &str, rows: usize, cols: usize) -> Result<JsValue, JsValue> {
        let new_spreadsheet = NewSpreadsheet {
            name: name.to_string(),
            rows,
            cols,
        };
        
        let body = serde_wasm_bindgen::to_value(&new_spreadsheet)?;
        let response: ApiResponse<SpreadsheetData> = self.fetch("/spreadsheets", "POST", Some(&body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    #[wasm_bindgen]
    pub async fn update_spreadsheet(&self, id: &str, name: Option<String>, content: Option<String>) -> Result<JsValue, JsValue> {
        let update = UpdateSpreadsheet {
            name,
            content,
        };
        
        let body = serde_wasm_bindgen::to_value(&update)?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}", id), "PUT", Some(&body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    #[wasm_bindgen]
    pub async fn delete_spreadsheet(&self, id: &str) -> Result<JsValue, JsValue> {
        let response: ApiResponse<()> = self.fetch(&format!("/spreadsheets/{}", id), "DELETE", None).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    // Individual cell update (JS-friendly)
    #[wasm_bindgen]
    pub async fn update_single_cell(&self, id: &str, row: usize, col: usize, value: String) -> Result<JsValue, JsValue> {
        let updates = CellUpdates {
            cells: vec![CellUpdate { row, col, value }],
        };
        
        let body = serde_wasm_bindgen::to_value(&updates)?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}/cells", id), "POST", Some(&body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    // Update multiple cells (gets JS array)
    #[wasm_bindgen]
    pub async fn update_cells_js(&self, id: &str, cells_array: &JsValue) -> Result<JsValue, JsValue> {
        let cells: Vec<CellUpdate> = serde_wasm_bindgen::from_value(cells_array.clone())?;
        let updates = CellUpdates { cells };
        
        let body = serde_wasm_bindgen::to_value(&updates)?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}/cells", id), "POST", Some(&body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    #[wasm_bindgen]
    pub async fn export_spreadsheet(&self, id: &str) -> Result<String, JsValue> {
        let response: String = self.fetch(&format!("/spreadsheets/{}/export", id), "GET", None).await?;
        Ok(response)
    }
    
    #[wasm_bindgen]
    pub async fn import_spreadsheet(&self, id: &str, csv_data: &str) -> Result<JsValue, JsValue> {
        let body = JsValue::from_str(csv_data);
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}/import", id), "POST", Some(&body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
}
