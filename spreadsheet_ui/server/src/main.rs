use rocket::launch;
#[macro_use] extern crate rocket;

mod models;
mod storage;
mod handlers;

use rocket::fs::{FileServer, relative};
use storage::Storage;
use handlers::*;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Storage::new())
        .mount("/", FileServer::from(relative!("static")))
        .mount("/api", routes![
            list_spreadsheets,
            get_spreadsheet_by_name,
            get_spreadsheet,
            create_spreadsheet,
            update_spreadsheet,
            delete_spreadsheet,
            update_cells,
            export_spreadsheet,
            import_spreadsheet,
        ])
}
