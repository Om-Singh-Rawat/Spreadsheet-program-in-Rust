use rocket::launch;
use rocket::fs::{FileServer, Options, relative};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/",FileServer::from(relative!("static")))
}
