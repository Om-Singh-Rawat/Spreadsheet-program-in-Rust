use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;

use crate::models::{
    ApiResponse, CellUpdates, NewSpreadsheet, SpreadsheetData, SpreadsheetMetadata,
    UpdateSpreadsheet,
};
use crate::storage::Storage;

// List all spreadsheets
#[get("/spreadsheets")]
pub fn list_spreadsheets(storage: &State<Storage>) -> Json<ApiResponse<Vec<SpreadsheetMetadata>>> {
    let spreadsheets = storage.list_spreadsheets();

    Json(ApiResponse {
        status: "success".to_string(),
        data: Some(spreadsheets),
        message: None,
    })
}

// Get spreadsheet by ID
#[get("/spreadsheets/<id>", rank = 2)]
pub fn get_spreadsheet(
    id: &str,
    storage: &State<Storage>,
) -> Result<Json<ApiResponse<SpreadsheetData>>, status::Custom<Json<ApiResponse<()>>>> {
    match storage.get_spreadsheet(id) {
        Some(spreadsheet) => Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(spreadsheet),
            message: None,
        })),
        None => Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with ID {} not found", id)),
            }),
        )),
    }
}

// Create new spreadsheet
#[post("/spreadsheets", data = "<new_spreadsheet>")]
pub fn create_spreadsheet(
    new_spreadsheet: Json<NewSpreadsheet>,
    storage: &State<Storage>,
) -> Json<ApiResponse<SpreadsheetData>> {
    let spreadsheet = storage.create_spreadsheet(
        new_spreadsheet.name.clone(),
        new_spreadsheet.rows,
        new_spreadsheet.cols,
    );

    Json(ApiResponse {
        status: "success".to_string(),
        data: Some(spreadsheet),
        message: None,
    })
}

// Update spreadsheet
#[put("/spreadsheets/<id>", data = "<update>")]
pub fn update_spreadsheet(
    id: &str,
    update: Json<UpdateSpreadsheet>,
    storage: &State<Storage>,
) -> Result<Json<ApiResponse<SpreadsheetData>>, status::Custom<Json<ApiResponse<()>>>> {
    match storage.update_spreadsheet(id, update.name.clone(), update.content.clone()) {
        Some(spreadsheet) => Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(spreadsheet),
            message: None,
        })),
        None => Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with ID {} not found", id)),
            }),
        )),
    }
}

// Delete spreadsheet
#[delete("/spreadsheets/<id>")]
pub fn delete_spreadsheet(
    id: &str,
    storage: &State<Storage>,
) -> Result<Json<ApiResponse<()>>, status::Custom<Json<ApiResponse<()>>>> {
    if storage.delete_spreadsheet(id) {
        Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: None,
            message: Some(format!("Spreadsheet with ID {} deleted", id)),
        }))
    } else {
        Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with ID {} not found", id)),
            }),
        ))
    }
}

// Update cells
#[post("/spreadsheets/<id>/cells", data = "<updates>")]
pub fn update_cells(
    id: &str,
    updates: Json<CellUpdates>,
    storage: &State<Storage>,
) -> Result<Json<ApiResponse<SpreadsheetData>>, status::Custom<Json<ApiResponse<()>>>> {
    let cell_updates: Vec<(usize, usize, String)> = updates
        .cells
        .iter()
        .map(|cell| (cell.row, cell.col, cell.value.clone()))
        .collect();

    match storage.update_cells(id, &cell_updates) {
        Some(spreadsheet) => Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(spreadsheet),
            message: None,
        })),
        None => Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with ID {} not found", id)),
            }),
        )),
    }
}

// Export spreadsheet as CSV
#[get("/spreadsheets/<id>/export")]
pub fn export_spreadsheet(
    id: &str,
    storage: &State<Storage>,
) -> Result<String, status::Custom<Json<ApiResponse<()>>>> {
    match storage.get_spreadsheet(id) {
        Some(spreadsheet) => Ok(spreadsheet.content),
        None => Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with ID {} not found", id)),
            }),
        )),
    }
}

// Import CSV data
#[post("/spreadsheets/<id>/import", data = "<csv_data>")]
pub fn import_spreadsheet(
    id: &str,
    csv_data: String,
    storage: &State<Storage>,
) -> Result<Json<ApiResponse<SpreadsheetData>>, status::Custom<Json<ApiResponse<()>>>> {
    match storage.update_spreadsheet(id, None, Some(csv_data)) {
        Some(spreadsheet) => Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(spreadsheet),
            message: None,
        })),
        None => Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with ID {} not found", id)),
            }),
        )),
    }
}

#[get("/spreadsheets/by_name?<name>", rank = 1)]
pub fn get_spreadsheet_by_name(
    name: &str,
    storage: &State<Storage>,
) -> Result<Json<ApiResponse<SpreadsheetData>>, status::Custom<Json<ApiResponse<()>>>> {
    match storage.get_by_name(name) {
        Some(data) => Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(data),
            message: None,
        })),
        None => Err(status::Custom(
            Status::NotFound,
            Json(ApiResponse {
                status: "error".to_string(),
                data: None,
                message: Some(format!("Spreadsheet with name '{}' not found", name)),
            }),
        )),
    }
}
