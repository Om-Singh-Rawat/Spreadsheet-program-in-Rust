use rocket::launch;
#[macro_use]
extern crate rocket;

mod handlers;
mod models;
mod storage;

use handlers::*;
use rocket::fs::{relative, FileServer};
use storage::Storage;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Storage::new())
        .mount("/", FileServer::from(relative!("static")))
        .mount(
            "/api",
            routes![
                list_spreadsheets,
                get_spreadsheet_by_name,
                get_spreadsheet,
                create_spreadsheet,
                update_spreadsheet,
                delete_spreadsheet,
                update_cells,
                export_spreadsheet,
                import_spreadsheet,
            ],
        )
}
