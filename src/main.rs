#[macro_use] extern crate rocket;
extern crate hex;
use rocket::fs::FileServer;
use sha2::{Sha256, Digest};
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

pub struct CORS;

// https://stackoverflow.com/a/69342225/10942736
#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Attaching CORS headers to responses",
      kind: Kind::Response
    }
  }

  async fn on_response<'r>(
    &self,
    _request: &'r Request<'_>,
    response: &mut Response<'r>
  ) {
    response.set_header(
      Header::new("Access-Control-Allow-Origin", "*")
    );
    response.set_header(
      Header::new(
        "Access-Control-Allow-Methods",
        "POST, GET, PATCH, OPTIONS"
      )
    );
    response.set_header(
      Header::new("Access-Control-Allow-Headers", "*")
    );
    response.set_header(
      Header::new("Access-Control-Allow-Credentials", "true")
    );
  }
}

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

#[post("/new-user?<username>&<password>")]
fn new_user(username: &str, password: &str) -> String {
  let password_hash = hash(password);
  format!(
    "New user {} with password hash {} (not really)",
    username,
    password_hash
  )
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .attach(CORS)
    .mount("/", routes![mudnix])
    .mount("/hash", routes![hash])
    .mount("/user", routes![new_user])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
