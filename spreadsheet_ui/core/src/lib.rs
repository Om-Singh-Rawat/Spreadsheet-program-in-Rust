
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
    #[derive(Clone)] 
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

    }
}