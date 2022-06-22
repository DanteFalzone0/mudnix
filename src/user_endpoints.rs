/* API endpoints for user account management and info. */
use rocket::State;
use rocket::response::content;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::time::{self, Duration};
use serde_json;

use crate::user;
use crate::world_map;
use crate::mudnix_utils;

#[post("/new-user?<username>&<password>")]
pub fn new_user(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> String {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);

  if user_list.contains(username) {
    format!("User {} already exists.", username)
  } else {
    let password_hash = mudnix_utils::hash(password);
    let user = user::User::new(
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
pub fn login(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>,
  logged_in_user_pool: &State<mudnix_utils::LoggedInUserPool>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);

  let password_hash = mudnix_utils::hash(password);

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
        "username": username,
        "logged_in": false,
        "err": format!(
          "nonexistent location {} found in user save file when attempting to log in",
          &user_list.users[i].world_location
        )
      }).to_string())
    };
    let response = match world_loc.move_user_to_self(
      username,
      &user_list.users[i].world_location
    ) {
      Ok(r) => r,
      Err(_) => return content::Json(serde_json::json!({
        "username": username,
        "logged_in": false,
        "err": "invalid location in user file"
      }).to_string())
    };
    world_loc.save_to_file(&world_loc_path);

    content::Json(serde_json::json!({
      "username": username,
      "logged_in": true,
      "info": response,
      "was_previously_logged_in": already_logged_in
    }).to_string())
  } else {
    content::Json(serde_json::json!({
      "username": username,
      "logged_in": false,
      "err": "invalid credentials"
    }).to_string())
  }
}

#[get("/logout?<username>&<password>")]
pub fn logout(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>,
  logged_in_user_pool: &State<mudnix_utils::LoggedInUserPool>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);

  let password_hash = mudnix_utils::hash(password);

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
        "username": username,
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
      "username": username,
      "logged_out": true
    }).to_string())
  } else {
    content::Json(serde_json::json!({
      "username": username,
      "logged_out": false,
      "err": "invalid credentials"
    }).to_string())
  }
}

#[get("/autologout?<username>&<password>")]
pub fn autologout(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> EventStream![] {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();

  // we make this copy so we don't have to put the borrow inside the EventStream
  let username_copy = String::from(username);

  let password_hash = mudnix_utils::hash(password);

  // we copy the file path here, but it's fine because we are only reading the file, not writing it
  let users_file_path_copy: String = String::from(users_file_path);

  // run every 10 minutes
  let mut interval = time::interval(Duration::from_secs(600));

  EventStream! {
    loop {
      interval.tick().await;
      let fresh_user_list = user::UserList::from_file(&users_file_path_copy);
      let i = fresh_user_list.get_index_if_valid_creds(&username_copy, &password_hash).unwrap();
      if fresh_user_list.users[i].has_been_logged_in_30_mins() {
        yield Event::data(serde_json::json!({
          "username": username_copy,
          "succeeded": true,
          "info": "logout"
        }).to_string());
      }
    }
  }
}

#[get("/inventory?<username>&<password>")]
pub fn inventory(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);
  let password_hash = mudnix_utils::hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    user_list.update_timestamp_of_index(i);
    user_list.save_to_file(users_file_path);
    content::Json(serde_json::json!({
      "username": username,
      "succeeded": true,
      "inventory": user_list.users[i].inventory
    }).to_string())
  } else {
    mudnix_utils::error_response(username, "invalid credentials")
  }
}
