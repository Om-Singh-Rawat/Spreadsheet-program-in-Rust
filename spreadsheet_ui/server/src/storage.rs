use crate::models::{SpreadsheetData, SpreadsheetMetadata};
use core::spreadsheet::Spreadsheet;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub struct Storage {
    spreadsheets: RwLock<HashMap<String, SpreadsheetData>>,
    name_to_id: RwLock<HashMap<String, String>>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            spreadsheets: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
        }
    }

    pub fn list_spreadsheets(&self) -> Vec<SpreadsheetMetadata> {
        let spreadsheets = self.spreadsheets.read().unwrap();
        spreadsheets
            .values()
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

    pub fn get_by_name(&self, name: &str) -> Option<SpreadsheetData> {
        let name_index = self.name_to_id.read().unwrap();
        name_index
            .get(name)
            .and_then(|id| self.spreadsheets.read().unwrap().get(id).cloned())
    }

    pub fn create_spreadsheet(&self, name: String, rows: usize, cols: usize) -> SpreadsheetData {
        let id = Uuid::new_v4().to_string();
        let name_clone = name.clone(); // Clone name before move

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let sheet = Spreadsheet::new(rows, cols);
        let csv_content = sheet.export_csv();

        // Create spreadsheet with original name
        let spreadsheet = SpreadsheetData {
            id: id.clone(),
            name, // name moved here
            content: csv_content,
            created_at: now,
            updated_at: now,
        };

        // Insert into main storage with cloned ID
        let mut spreadsheets = self.spreadsheets.write().unwrap();
        spreadsheets.insert(id.clone(), spreadsheet.clone());

        // Insert into name index with cloned values
        let mut name_index = self.name_to_id.write().unwrap();
        name_index.insert(name_clone, id.clone());

        spreadsheet
    }

    pub fn update_spreadsheet(
        &self,
        id: &str,
        name: Option<String>,
        content: Option<String>,
    ) -> Option<SpreadsheetData> {
        let mut spreadsheets = self.spreadsheets.write().unwrap();

        if let Some(spreadsheet) = spreadsheets.get_mut(id) {
            let mut name_index = self.name_to_id.write().unwrap();
            let mut old_name = None;

            // Handle name changes first
            if let Some(new_name) = &name {
                // Track old name for index update
                old_name = Some(spreadsheet.name.clone());
                // Update the spreadsheet name
                spreadsheet.name = new_name.clone();
            }

            // Update content if provided
            if let Some(new_content) = &content {
                spreadsheet.content = new_content.clone();
            }

            // Update timestamps
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            spreadsheet.updated_at = now;

            // Update name-to-ID mapping
            if let Some(old_name) = old_name {
                // Remove old name entry
                name_index.remove(&old_name);
            }
            if name.is_some() {
                // Insert new name entry
                name_index.insert(spreadsheet.name.clone(), id.to_string());
            }

            Some(spreadsheet.clone())
        } else {
            None
        }
    }

    pub fn delete_spreadsheet(&self, id: &str) -> bool {
        let mut spreadsheets = self.spreadsheets.write().unwrap();
        spreadsheets.remove(id).is_some()
    }

    pub fn update_cells(
        &self,
        id: &str,
        updates: &[(usize, usize, String)],
    ) -> Option<SpreadsheetData> {
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
