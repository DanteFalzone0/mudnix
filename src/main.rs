#[macro_use] extern crate rocket;
extern crate hex;
use std::fs;
use std::sync::Mutex;
use rocket::fs::FileServer;
use sha2::{Sha256, Digest};
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::State;
use serde_json;

mod entities;

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

struct UsersJsonFilePath {
  pub file_path_mutex: Mutex<String>
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
fn new_user(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<UsersJsonFilePath>
) -> String {
  let users_file_path: &str = &users_file_path_mutex.file_path_mutex
    .lock().unwrap().to_string();
  let original_json = fs::read_to_string(users_file_path).unwrap();
  let mut user_list: entities::UserList = serde_json::from_str(&original_json)
    .expect("unable to parse json from users.json");
  if user_list.contains(username) {
    format!("User {} already exists.", username)
  } else {
    let password_hash = hash(password);
    let user = entities::User::new(
      username,
      &password_hash,
      "nowhere"
    );
    user_list.users.push(user);
    let output_json = serde_json::to_string_pretty(&user_list).unwrap();
    fs::write(users_file_path, output_json)
      .expect("unable to save user to users.json");
    format!(
      "New user {} created. Save your password - it can't be recovered!",
      username
    )
  }
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .manage(UsersJsonFilePath {
      file_path_mutex: Mutex::new(String::from("/home/runner/mudnix/users.json"))
    })
    .attach(CORS)
    .mount("/", routes![mudnix])
    .mount("/hash", routes![hash])
    .mount("/user", routes![new_user])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
