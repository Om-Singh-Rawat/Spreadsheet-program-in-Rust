use core::spreadsheet::Spreadsheet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadsheetMetadata {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadsheetData {
    pub id: String,
    pub name: String,
    pub content: String, // CSV content
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSpreadsheet {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSpreadsheet {
    pub name: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CellUpdate {
    pub row: usize,
    pub col: usize,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CellUpdates {
    pub cells: Vec<CellUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
}
