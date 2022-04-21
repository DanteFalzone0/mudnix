#[macro_use] extern crate rocket;
extern crate hex;
use std::sync::Mutex;
use rocket::fs::FileServer;
use sha2::{Sha256, Digest};
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::State;
use rocket::response::content;
use serde_json;

mod entities;
mod world_map;
mod mudnix_utils;

// https://stackoverflow.com/a/69342225/10942736
pub struct CORS;
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

type UsersFileMutex = mudnix_utils::FilePathMutex;

struct LoggedInUserPool {
  pub user_list_mutex: Mutex<entities::UserList>
}

#[get("/version")]
fn version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
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
  users_file_path_mutex: &State<UsersFileMutex>
) -> String {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = entities::UserList::from_file(users_file_path);

  if user_list.contains(username) {
    format!("User {} already exists.", username)
  } else {
    let password_hash = hash(password);
    let user = entities::User::new(
      username,
      &password_hash,
      "Quux_Plains::northern_region"
    );
    user_list.users.push(user);
    user_list.save_to_file(users_file_path);

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
  users_file_path_mutex: &State<UsersFileMutex>,
  logged_in_user_pool: &State<LoggedInUserPool>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = entities::UserList::from_file(users_file_path);

  let password_hash = hash(password);

  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    /* add the user to the pool of logged-in users if their credentials are valid
       and they aren't already in the pool */
    let mut pool = logged_in_user_pool.user_list_mutex.lock().unwrap();
    let already_logged_in = pool.contains(username);

    user_list.update_timestamp_of_index(i);
    if !already_logged_in {
      pool.users.push(
        user_list.users[i].clone()
      );
    }

    // save the user list with the updated timestamp
    user_list.save_to_file(users_file_path);

    // place the user in the appropriate location
    let world_loc_path = world_map::get_path_from_location_id(
      &user_list.users[i].world_location
    );
    let mut world_loc = match world_map::WorldLocation::from_file(&world_loc_path) {
      Ok(world_location) => world_location,
      Err(_) => return content::Json(serde_json::json!({
        "username": String::from(username),
        "logged_in": false,
        "err": format!(
          "nonexistent location {} found in user save file when attempting to log in",
          &user_list.users[i].world_location
        )
      }).to_string())
    };
    let response = world_loc.move_user_to_self(
      username,
      &user_list.users[i].world_location
    );
    world_loc.save_to_file(&world_loc_path);

    content::Json(serde_json::json!({
      "username": String::from(username),
      "logged_in": true,
      "info": response,
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
  users_file_path_mutex: &State<UsersFileMutex>,
  logged_in_user_pool: &State<LoggedInUserPool>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = entities::UserList::from_file(users_file_path);

  let password_hash = hash(password);

  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    /* remove the user from the pool of logged-in users if their credentials
       are valid and they are in the pool */
    let mut pool = logged_in_user_pool.user_list_mutex.lock().unwrap();

    // remove the user from the world
    let world_loc_path = world_map::get_path_from_location_id(
      &user_list.users[i].world_location
    );
    let mut world_loc = match world_map::WorldLocation::from_file(&world_loc_path) {
      Ok(world_location) => world_location,
      Err(_) => return content::Json(serde_json::json!({
        "username": String::from(username),
        "logged_out": false,
        "err": format!(
          "nonexistent location {} found in user save file when attempting to log out",
          &user_list.users[i].world_location
        )
      }).to_string())
    };
    world_loc.remove_user(username);
    world_loc.save_to_file(&world_loc_path);

    user_list.update_timestamp_of_index(i);
    pool.remove_user_if_exists(username);
    user_list.save_to_file(users_file_path);

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

fn move_user(
  username: &str,
  password_hash: &str,
  new_location_id: &str,
  users_file_path: &str,
  user_list: &mut entities::UserList
) -> content::Json<String> {
  if let Some(i) = user_list.get_index_if_valid_creds(username, password_hash) {
    let old_location_id: &str = &user_list.users[i].world_location;
    let mut old_location = match world_map::WorldLocation::from_location_id(old_location_id) {
      Ok(current_location) => current_location,
      Err(_) => return content::Json(serde_json::json!({
        "username": String::from(username),
        "succeeded": false,
        "err": format!("cannot move you from invalid location {}", old_location_id)
      }).to_string())
    };
    let response = match old_location.move_user_from(
      username, old_location_id
    ).to(new_location_id) {
      Ok(r) => r,
      Err(_) => return content::Json(serde_json::json!({
        "username": String::from(username),
        "succeeded": false,
        "err": format!("cannot move you to invalid location {}", new_location_id)
      }).to_string())
    };
    user_list.users[i].world_location = String::from(new_location_id);
    user_list.update_timestamp_of_index(i);
    user_list.save_to_file(users_file_path);
    content::Json(serde_json::json!({
      "username": String::from(username),
      "succeeded": true,
      "info": response
    }).to_string())
  } else {
    content::Json(serde_json::json!({
      "username": String::from(username),
      "succeeded": false,
      "err": "invalid credentials"
    }).to_string())
  }
}

#[get("/tp?<username>&<password>&<new_location>")]
fn teleport(
  username: &str,
  password: &str,
  new_location: &str,
  users_file_path_mutex: &State<UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = entities::UserList::from_file(users_file_path);
  let correct_hash = "5f910b87aa8c3a2b7eabf3c61f56fbae536af7949e15afd84cc0b67d50d5e909";
  if username == "dante_falzone" && hash(password) == correct_hash {
    move_user(username, correct_hash, new_location, users_file_path, &mut user_list)
  } else {
    content::Json(serde_json::json!({
      "username": String::from(username),
      "succeeded": false,
      "err": "You do not have permission to use this command."
    }).to_string())
  }
}

#[get("/goto?<username>&<password>&<new_location_id>")]
fn goto(
  username: &str,
  password: &str,
  new_location_id: &str,
  users_file_path_mutex: &State<UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = entities::UserList::from_file(users_file_path);
  let password_hash = hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    let old_location_id: &str = &user_list.users[i].world_location;
    let mut old_location = match world_map::WorldLocation::from_location_id(old_location_id) {
      Ok(current_location) => current_location,
      Err(_) => return content::Json(serde_json::json!({
        "username": String::from(username),
        "succeeded": false,
        "err": format!(
          "cannot move you from invalid location \"{}\"",
          old_location_id
        )
      }).to_string())
    };
    if old_location_id == new_location_id || old_location.is_neighbor(new_location_id) {
      let response = match old_location.move_user_from(
        username, old_location_id
      ).to(new_location_id) {
        Ok(r) => r,
        Err(_) => return content::Json(serde_json::json!({
          "username": String::from(username),
          "succeeded": false,
          "err": format!("cannot move you to invalid location {}", new_location_id)
        }).to_string())
      };
      user_list.users[i].world_location = String::from(new_location_id);
      user_list.update_timestamp_of_index(i);
      user_list.save_to_file(users_file_path);
      content::Json(serde_json::json!({
        "username": String::from(username),
        "succeeded": true,
        "info": response
      }).to_string())
    } else {
      content::Json(serde_json::json!({
        "username": String::from(username),
        "succeeded": false,
        "err": format!(
          "{} is not next to {}",
          world_map::location_id_to_human_readable(old_location_id),
          world_map::location_id_to_human_readable(new_location_id)
        )
      }).to_string())
    }
  } else {
    content::Json(serde_json::json!({
      "username": String::from(username),
      "succeeded": false,
      "err": "invalid credentials"
    }).to_string())
  }
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .manage(UsersFileMutex {
      mutex: Mutex::new(String::from("/home/runner/mudnix/users.json"))
    })
    .manage(LoggedInUserPool {
      user_list_mutex: Mutex::new(entities::UserList::new())
    })
    .attach(CORS)
    .mount("/", routes![mudnix])
    .mount("/", routes![version])
    .mount("/hash", routes![hash])
    .mount("/user", routes![new_user])
    .mount("/user", routes![login])
    .mount("/user", routes![logout])
    .mount("/game", routes![teleport])
    .mount("/game", routes![goto])
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
}
