#[macro_use] extern crate rocket;
extern crate hex;
use rocket::fs::FileServer;
use sha2::{Sha256, Digest};


#[get("/mudnix")]
fn mudnix() -> &'static str {
  "Hello from Mudnix!"
}

#[get("/sha256?<s>")]
fn hash(s: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(s);
  let hash = hasher.finalize();
  hex::encode(hash)
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .mount("/", routes![mudnix])
    .mount("/hash", routes![hash])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
