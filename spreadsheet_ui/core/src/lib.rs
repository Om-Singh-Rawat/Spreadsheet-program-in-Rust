
pub mod spreadsheet;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "cli")]
pub fn run_cli() {
    println!("CLI mode disabled.");
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::spreadsheet::Spreadsheet;

    #[wasm_bindgen]
    #[derive(Clone, PartialEq)] 
    pub struct WasmSheet {
        inner: Spreadsheet,
    }

    #[wasm_bindgen]
    impl WasmSheet {
        #[wasm_bindgen(constructor)]
        pub fn new(rows: usize, cols: usize) -> Self {
            console_error_panic_hook::set_once();
            WasmSheet { inner: Spreadsheet::new(rows, cols) }
        }

        #[wasm_bindgen]
        pub fn assign(&mut self, cell: &str, expr: &str) -> Result<(), JsValue> {
            self.inner
                .handle_assignment(&format!("{}={}", cell, expr))
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn get(&self, row: usize, col: usize) -> i32 {
            self.inner.get_cell_value(row, col).unwrap_or(0)
        }

        #[wasm_bindgen]
        pub fn get_content(&self, row: usize, col: usize) -> String {
            self.inner.get_cell_content(row, col)
        }

        #[wasm_bindgen]
        pub fn is_formula(&self, row: usize, col: usize) -> bool {
            self.inner.is_formula(row, col)
        }

        #[wasm_bindgen]
        pub fn get_error(&self, row: usize, col: usize) -> Option<String> {
            self.inner.get_error(row, col)
        }

        #[wasm_bindgen]
        pub fn get_cell_value(&self, row: usize, col: usize) -> i32 {
            self.inner.get_cell_value(row, col).unwrap_or(0)
        }

        #[wasm_bindgen]
        pub fn process_input(&mut self, input: &str) -> Result<(), JsValue> {
            self.inner
                .handle_assignment(input)
                .map_err(|e| JsValue::from_str(&e))
        }

        #[wasm_bindgen]
        pub fn import_csv(&mut self, csv_data: &str) -> Result<(), JsValue> {
            self.inner.import_csv(csv_data)
                .map_err(|e| {
                    let error_msg = format!("Import failed: {}", e);
                    web_sys::console::error_1(&JsValue::from_str(&error_msg));
                    JsValue::from_str(&error_msg)
                })
        }

        #[wasm_bindgen]
        pub fn export_csv(&self) -> String {
            self.inner.export_csv()
        }

        #[wasm_bindgen]
        pub fn download_csv(&self, filename: &str) -> Result<(), JsValue> {
            let csv_data = self.inner.export_csv();
            // Create a Uint8Array from the CSV string
            let array = js_sys::Uint8Array::new_with_length(csv_data.len() as u32);
            array.copy_from(csv_data.as_bytes());
            // Create a Blob and download URL
            let blob = web_sys::Blob::new_with_u8_array_sequence(&js_sys::Array::of1(&array))?;
            let url = web_sys::Url::create_object_url_with_blob(&blob)?;
            
            // Create and trigger a download link
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let a = document.create_element("a")?;
            a.set_attribute("href", &url)?;
            a.set_attribute("download", filename)?;
            a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
            
            web_sys::Url::revoke_object_url(&url)?;
            Ok(())
        }

        #[wasm_bindgen]
        pub fn column_index_to_label(index: usize) -> String {
            Spreadsheet::column_index_to_label(index)
        }

        #[wasm_bindgen]
        pub fn column_label_to_index(label: &str) -> Option<usize> {
            Spreadsheet::column_label_to_index(label)
        }


    }
}