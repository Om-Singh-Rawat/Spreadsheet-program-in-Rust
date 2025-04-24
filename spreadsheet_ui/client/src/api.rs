#![cfg(not(tarpaulin))]
use std::borrow::BorrowMut;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response, console};
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use gloo_net::http::Request;
use gloo_utils::format::JsValueSerdeExt;


#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct SpreadsheetData {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
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
    auth_token: String,
}

#[wasm_bindgen]
impl ApiClient {
    #[wasm_bindgen(constructor)]
    pub fn new(auth_token: String) -> Self {
        Self {
            base_url: "/api".to_string(),
            auth_token,  // Properly initialize the field
        }
    }

    // This method isn't exposed to JS directly
    async fn fetch<T: for<'de> serde::Deserialize<'de>>(
        &self,
        url: &str,
        method: &str,
        body: Option<String>,
    ) -> Result<T, JsValue> {
        let full_url = format!("{}{}", self.base_url, url);
    
        // Choose method
        let request_builder = match method {
            "GET" => Request::get(&full_url),
            "POST" => Request::post(&full_url),
            "PUT" => Request::put(&full_url),
            "DELETE" => Request::delete(&full_url),
            _ => return Err(JsValue::from_str("Unsupported HTTP method")),
        };
    
        // Add headers if needed
        let request_builder = if body.is_some() {
            request_builder.header("Content-Type", "application/json")
        } else {
            request_builder
        };
    
        // Finalize the request correctly based on method type
        let request = match method {
            // For GET and HEAD, don't set a body
            "GET" | "HEAD" => request_builder.build(),
            // For other methods, include body if provided
            _ => match body {
                Some(body_str) => request_builder.body(body_str),
                None => request_builder.build(), // Don't set empty body
            }
        }.map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    
        // Send the request
        let resp = request.send().await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        
        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP Error: {}", resp.status())));
        }
        
        let json = resp.json::<T>().await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        
        Ok(json)
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
        let body = serde_json::to_string(&new_spreadsheet).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let response: ApiResponse<SpreadsheetData> = self.fetch("/spreadsheets", "POST", Some(body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }

    #[wasm_bindgen]
    pub async fn update_spreadsheet(&self, id: &str, name: Option<String>, content: Option<String>) -> Result<JsValue, JsValue> {
        let update = UpdateSpreadsheet { name, content };
        let body = serde_json::to_string(&update).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}", id), "PUT", Some(body)).await?;
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
        let body = serde_json::to_string(&updates).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}/cells", id), "POST", Some(body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }

    #[wasm_bindgen]
    pub async fn update_cells_js(&self, id: &str, cells_array: &JsValue) -> Result<JsValue, JsValue> {
        let cells: Vec<CellUpdate> = serde_wasm_bindgen::from_value(cells_array.clone())?;
        let updates = CellUpdates { cells };
        let body = serde_json::to_string(&updates).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}/cells", id), "POST", Some(body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }

    
    #[wasm_bindgen]
    pub async fn export_spreadsheet(&self, id: &str) -> Result<String, JsValue> {
        let response: String = self.fetch(&format!("/spreadsheets/{}/export", id), "GET", None).await?;
        Ok(response)
    }
    
    #[wasm_bindgen]
    pub async fn import_spreadsheet(&self, id: &str, csv_data: &str) -> Result<JsValue, JsValue> {
        // If your backend expects raw CSV as a string, send as plain text, else wrap in JSON
        let body = serde_json::to_string(&csv_data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let response: ApiResponse<SpreadsheetData> = self.fetch(&format!("/spreadsheets/{}/import", id), "POST", Some(body)).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }


    #[wasm_bindgen]
    pub async fn get_spreadsheet_by_name(&self, name: &str) -> Result<JsValue, JsValue> {
        let encoded_name = js_sys::encode_uri_component(name);
        let url = format!("{}/spreadsheets/by_name?name={}", self.base_url, encoded_name);
        
        let response = Request::get(&url)
            .header("Authorization", &self.auth_token)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {}", e)))?;

        if response.status() != 200 {
            return Err(JsValue::from_str(&format!("HTTP error: {}", response.status())));
        }

        let text = response.text().await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        // Parse the JSON string into a Rust struct using serde_json
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| JsValue::from_str(&format!("serde_json error: {:?}", e)))?;

        // Convert the serde_json::Value into JsValue for use with serde_wasm_bindgen::from_value in Yew
        let js_value = wasm_bindgen::JsValue::from_serde(&value)
            .map_err(|e| JsValue::from_str(&format!("JsValue::from_serde error: {:?}", e)))?;

        Ok(js_value)
    }

}
