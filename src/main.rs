#[macro_use] extern crate rocket;
extern crate hex;
use std::fs;
use std::sync::Mutex;
use std::time::SystemTime;
use rocket::fs::FileServer;
use sha2::{Sha256, Digest};
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::State;
use rocket::response::content;
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

struct LoggedInUserPool {
  pub users_mutex: Mutex<Vec<entities::User>>
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

  // load the old user data
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

    // save the new user data
    let output_json = serde_json::to_string_pretty(&user_list).unwrap();
    fs::write(users_file_path, output_json)
      .expect("unable to save user to users.json");

    format!(
      "New user {} created. Save your password - it can't be recovered!",
      username
    )
  }
}

#[get("/login?<username>&<password>")]
fn login(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<UsersJsonFilePath>,
  logged_in_user_pool: &State<LoggedInUserPool>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.file_path_mutex
    .lock().unwrap().to_string();

  // load old user data
  let original_json = fs::read_to_string(users_file_path).unwrap();
  let mut user_list: entities::UserList = serde_json::from_str(&original_json)
    .expect("unable to parse json from users.json");

  let password_hash = hash(password);

  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    /* add the user to the pool of logged-in users if their credentials are valid
       and they aren't already in the pool */
    let mut pool = logged_in_user_pool.users_mutex.lock().unwrap();
    let already_logged_in = pool.iter().any(|user|
      user.username == username
    );
    // update timestamp to reflect the last time they did something
    user_list.users[i].last_activity_timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    if !already_logged_in {
      pool.push(
        user_list.users[i].clone()
      );
    }

    // save the user list with the updated timestamp
    let output_json = serde_json::to_string_pretty(&user_list).unwrap();
    fs::write(users_file_path, output_json)
      .expect("unable to save user to users.json");

    content::Json(serde_json::json!({
      "username": String::from(username),
      "logged_in": true,
      "was_previously_logged_in": already_logged_in
    }).to_string())
  } else {
    content::Json(serde_json::json!({
      "username": String::from(username),
      "logged_in": false,
      "err": "invalid credentials"
    }).to_string())
  }
}

#[get("/logout?<username>&<password>")]
fn logout(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<UsersJsonFilePath>,
  logged_in_user_pool: &State<LoggedInUserPool>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.file_path_mutex
    .lock().unwrap().to_string();

  // load old user data
  let original_json = fs::read_to_string(users_file_path).unwrap();
  let mut user_list: entities::UserList = serde_json::from_str(&original_json)
    .expect("unable to parse json from users.json");

  let password_hash = hash(password);

  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    /* remove the user from the pool of logged-in users if their credentials
      are valid and they are in the pool */
    let mut pool = logged_in_user_pool.users_mutex.lock().unwrap();

    // update timestamp to reflect the last time they did something
    user_list.users[i].last_activity_timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    if let Some(j) = pool.iter().position(|user| user.username == username) {
      pool.swap_remove(j);
    }

    // save the user list with the updated timestamp
    let output_json = serde_json::to_string_pretty(&user_list).unwrap();
    fs::write(users_file_path, output_json)
      .expect("unable to save user to users.json");

    content::Json(serde_json::json!({
      "username": String::from(username),
      "logged_out": true
    }).to_string())
  } else {
    content::Json(serde_json::json!({
      "username": String::from(username),
      "logged_out": false,
      "err": "invalid credentials"
    }).to_string())
  }
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .manage(UsersJsonFilePath {
      file_path_mutex: Mutex::new(String::from("/home/runner/mudnix/users.json"))
    })
    .manage(LoggedInUserPool {
      users_mutex: Mutex::new(vec![])
    })
    .attach(CORS)
    .mount("/", routes![mudnix])
    .mount("/hash", routes![hash])
    .mount("/user", routes![new_user])
    .mount("/user", routes![login])
    .mount("/user", routes![logout])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
