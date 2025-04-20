use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;
use crate::models::{SpreadsheetMetadata, SpreadsheetData};
use core::spreadsheet::Spreadsheet;

pub struct Storage {
    spreadsheets: RwLock<HashMap<String, SpreadsheetData>>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            spreadsheets: RwLock::new(HashMap::new()),
        }
    }

    pub fn list_spreadsheets(&self) -> Vec<SpreadsheetMetadata> {
        let spreadsheets = self.spreadsheets.read().unwrap();
        spreadsheets.values()
            .map(|s| SpreadsheetMetadata {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
            })
            .collect()
    }

    pub fn get_spreadsheet(&self, id: &str) -> Option<SpreadsheetData> {
        let spreadsheets = self.spreadsheets.read().unwrap();
        spreadsheets.get(id).cloned()
    }

    pub fn create_spreadsheet(&self, name: String, rows: usize, cols: usize) -> SpreadsheetData {
        let id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Create a new empty spreadsheet
        let sheet = Spreadsheet::new(rows, cols);
        let csv_content = sheet.export_csv();
        
        let spreadsheet = SpreadsheetData {
            id: id.clone(),
            name,
            content: csv_content,
            created_at: now,
            updated_at: now,
        };
        
        let mut spreadsheets = self.spreadsheets.write().unwrap();
        spreadsheets.insert(id, spreadsheet.clone());
        
        spreadsheet
    }

    pub fn update_spreadsheet(&self, id: &str, name: Option<String>, content: Option<String>) -> Option<SpreadsheetData> {
        let mut spreadsheets = self.spreadsheets.write().unwrap();
        
        if let Some(spreadsheet) = spreadsheets.get_mut(id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if let Some(new_name) = name {
                spreadsheet.name = new_name;
            }
            
            if let Some(new_content) = content {
                spreadsheet.content = new_content;
            }
            
            spreadsheet.updated_at = now;
            return Some(spreadsheet.clone());
        }
        
        None
    }

    pub fn delete_spreadsheet(&self, id: &str) -> bool {
        let mut spreadsheets = self.spreadsheets.write().unwrap();
        spreadsheets.remove(id).is_some()
    }

    pub fn update_cells(&self, id: &str, updates: &[(usize, usize, String)]) -> Option<SpreadsheetData> {
        let mut spreadsheets = self.spreadsheets.write().unwrap();
        
        if let Some(spreadsheet) = spreadsheets.get_mut(id) {
            let mut sheet = Spreadsheet::new(20, 10); // Default size
            
            // Import current content
            let _ = sheet.import_csv(&spreadsheet.content);
            
            // Apply updates
            for (row, col, value) in updates {
                let cell_ref = format!("{}{}", Spreadsheet::column_index_to_label(*col), row + 1);
                let assignment = format!("{}={}", cell_ref, value);
                if let Err(e) = sheet.handle_assignment(&assignment) {
                    println!("Error assigning value to cell {}: {}", cell_ref, e);
                    // Optionally handle the error
                }
            }
            
            // Export updated content
            spreadsheet.content = sheet.export_csv();
            spreadsheet.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            return Some(spreadsheet.clone());
        }
        
        None
    }
}
