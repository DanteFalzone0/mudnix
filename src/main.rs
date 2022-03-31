#[macro_use] extern crate rocket;
use rocket::fs::FileServer;

#[get("/mudnix")]
fn mudnix() -> &'static str {
  "Hello from Mudnix!"
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .mount("/", routes![mudnix])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
