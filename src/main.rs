#[macro_use] extern crate rocket;
extern crate hex;
use rocket::fs::FileServer;
use sha2::{Sha256, Digest};


#[get("/mudnix")]
fn mudnix() -> &'static str {
  "Hello from Mudnix!"
}

#[get("/hash/<string_to_hash>")]
fn hash(string_to_hash: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(string_to_hash);
  let hash = hasher.finalize();
  hex::encode(hash)
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .mount("/", routes![mudnix])
    .mount("/sha256", routes![hash])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
