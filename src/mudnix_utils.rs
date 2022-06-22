use std::sync::Mutex;
use sha2::{Sha256, Digest};
use rocket::response::content;
use serde_json;

use crate::user;
use crate::world_map;

pub struct FilePathMutex {
  pub mutex: Mutex<String>
}

pub type UsersFileMutex = FilePathMutex;

pub struct LoggedInUserPool {
  pub user_list_mutex: Mutex<user::UserList>
}

#[get("/sha256?<s>")]
pub fn hash(s: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(s);
  let hash = hasher.finalize();
  hex::encode(hash)
}

pub fn error_response(username: &str, error_response: &str) -> content::Json<String> {
  content::Json(serde_json::json!({
    "username": username,
    "succeeded": false,
    "err": error_response
  }).to_string())
}

pub fn move_user(
  username: &str,
  password_hash: &str,
  new_location_id: &str,
  users_file_path: &str,
  user_list: &mut user::UserList
) -> content::Json<String> {
  if let Some(i) = user_list.get_index_if_valid_creds(username, password_hash) {
    let old_location_id: &str = &user_list.users[i].world_location;
    let mut old_location = match world_map::WorldLocation::from_location_id(old_location_id) {
      Ok(current_location) => current_location,
      Err(_) => return error_response(
        username,
        &format!("cannot move you from invalid location {}", old_location_id)
      )
    };
    let response = match old_location.move_user_from(
      username, old_location_id
    ).to(new_location_id) {
      Ok(r) => r,
      Err(_) => return error_response(
        username,
        &format!("cannot move you to invalid location {}", new_location_id)
      )
    };

    user_list.users[i].world_location = String::from(new_location_id);
    user_list.update_timestamp_of_index(i);
    user_list.save_to_file(users_file_path);
    content::Json(serde_json::json!({
      "username": username,
      "succeeded": true,
      "info": response,
      "active_treasure_chest": user_list.users[i].active_treasure_chest
    }).to_string())
  } else {
    error_response(username, "invalid credentials")
  }
}
