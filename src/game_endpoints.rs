/**
 * API endpoints for things that happen in the game, or game actions
 * that a user may take.
 */
use crate::rand::Rng;
use rocket::State;
use rocket::response::content;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::time::{self, Duration};
use serde_json;

use crate::entities::ItemContainer;
use crate::entities;
use crate::user;
use crate::world_map;
use crate::mudnix_utils;
use crate::message;

#[get("/tp?<username>&<password>&<new_location>")]
pub fn teleport(
  username: &str,
  password: &str,
  new_location: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);
  let correct_hash = "e6fd95a315bb7129a50fd85b20af443d9a4d42c22aaff632c81808b4aee53335";
  if username == "dante_falzone" && mudnix_utils::hash(password) == correct_hash {
    mudnix_utils::move_user(username, correct_hash, new_location, users_file_path, &mut user_list)
  } else {
    mudnix_utils::error_response(username, "you do not have permission to use this command")
  }
}

#[get("/goto?<username>&<password>&<new_location_id>")]
pub fn goto(
  username: &str,
  password: &str,
  new_location_id: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);
  let password_hash = mudnix_utils::hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    let old_location_id: &str = &user_list.users[i].world_location;
    let mut old_location = match world_map::WorldLocation::from_location_id(old_location_id) {
      Ok(current_location) => current_location,
      Err(_) => return mudnix_utils::error_response(
        username, &format!(
          "cannot move you from invalid location \"{}\"",
          old_location_id
        )
      )
    };

    let old_sublocation_id = match world_map::get_sublocation_from_id(old_location_id) {
      Ok(id) => id,
      Err(_) => return mudnix_utils::error_response(
        username, &format!(
          "no sublocation specified for {}",
          old_location_id
        )
      )
    };

    let legal_to_move: bool =
      old_location_id == new_location_id
      || old_location.name == world_map::get_parent_location_from_id(new_location_id)
      || old_location.attrs.sublocations.iter().any(
        |sl| sl.name == old_sublocation_id
        && sl.is_neighbor(new_location_id)
      );

    if legal_to_move {
      let response = match old_location.move_user_from(
        username, old_location_id
      ).to(new_location_id) {
        Ok(r) => r,
        Err(_) => return mudnix_utils::error_response(
          username, &format!("cannot move you to invalid location {}", new_location_id)
        )
      };
      let new_location = world_map::WorldLocation::from_location_id(new_location_id)
        .unwrap();

      // generate a TreasureChest
      let spawn_val = rand::thread_rng().gen_range(0.0..1.0);
      if spawn_val < new_location.attrs.treasure_chest_spawn_rate {
        user_list.users[i].active_treasure_chest = Some(entities::TreasureChest::new());
      } else {
        user_list.users[i].active_treasure_chest = None;
      }
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
      mudnix_utils::error_response(
        username,
        &format!(
          "{} is not next to {}",
          world_map::location_id_to_human_readable(old_location_id),
          world_map::location_id_to_human_readable(new_location_id)
        )
      )
    }
  } else {
    mudnix_utils::error_response(username, "invalid credentials")
  }
}

#[get("/map?<username>&<password>")]
pub fn map(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let user_list = user::UserList::from_file(users_file_path);
  let password_hash = mudnix_utils::hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    let old_location_id: &str = &user_list.users[i].world_location;
    let old_location = match world_map::WorldLocation::from_location_id(old_location_id) {
      Ok(current_location) => current_location,
      Err(_) => return mudnix_utils::error_response(
        username,
        &format!(
          "you are currently located at invalid location \"{}\"",
          old_location_id
        )
      )
    };
    let old_sublocation_id = match world_map::get_sublocation_from_id(old_location_id) {
      Ok(id) => id,
      Err(_) => return mudnix_utils::error_response(
        username,
        &format!(
          "location id {} does not contain a sublocation",
          old_location_id
        )
      )
    };
    let mut neighbors: Vec<String> = vec![];
    let old_sublocation_index = match old_location.sublocation_index(&old_sublocation_id) {
      Ok(i) => i,
      Err(_) => return mudnix_utils::error_response(
        username,
        &format!(
          "unable to find the requested sublocation {}",
          old_sublocation_id
        )
      )
    };
    for neighbor in old_location.attrs.sublocations[old_sublocation_index].neighbors.iter() {
      neighbors.push(String::from(neighbor));
    }
    for sublocation in old_location.attrs.sublocations {
      neighbors.push(format!(
        "{}::{}",
        old_location.name,
        sublocation.name
      ));
    }
    content::Json(serde_json::json!({
      "username": username,
      "succeeded": true,
      "locations": neighbors
    }).to_string())
  } else {
    mudnix_utils::error_response(username, "invalid credentials")
  }
}

#[get("/close-chest?<username>&<password>")]
pub fn close_chest(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);
  let password_hash = mudnix_utils::hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    if let Some(treasure_chest) = user_list.users[i].active_treasure_chest.clone() {
      for item in treasure_chest.contents.iter() {
        user_list.users[i].inventory.add_item(item);
      }
    }
    user_list.users[i].active_treasure_chest = None;
    user_list.update_timestamp_of_index(i);
    user_list.save_to_file(users_file_path);
    content::Json(serde_json::json!({
      "username": username,
      "succeeded": true,
      "info": "The chest closes and disappears into ethereal green flames\
              \nwhich radiate no heat."
    }).to_string())
  } else {
    mudnix_utils::error_response(username, "request failed")
  }
}

#[post("/say?<username>&<password>&<message>")]
pub fn say(
  username: &str,
  password: &str,
  message: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> &'static str {
  /* We use the pool here instead of just loading from the users file
     because we don't want to let users send messages if they aren't logged in.
     Otherwise it would be trivial to write a script to spam people with messages. */
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);
  let password_hash = mudnix_utils::hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    let mut message_queue = message::MessageQueue::new("/home/runner/mudnix/message_queue");
    message_queue.send_message(message::Message::new(
      message, username, &user_list.users[i].world_location
    ));
    user_list.update_timestamp_of_index(i);
    user_list.save_to_file(users_file_path);
    "Ok"
  } else {
    "Denied"
  }
}

#[get("/message-queue?<username>&<password>")]
pub fn get_messages(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> EventStream![] {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let starting_user_list = user::UserList::from_file(users_file_path);

  // we make this copy so we don't have to put the borrow inside the EventStream
  let username_copy = String::from(username);

  let password_hash = mudnix_utils::hash(password);

  let starting_world_location: String = if let Some(i) = starting_user_list.get_index_if_valid_creds(
    &username_copy, &password_hash
  ) {
    starting_user_list.users[i].world_location.clone()
  } else {
    String::from("invalid")
  };

  // we copy the file path here, but it's fine because we are only reading the file, not writing it
  let users_file_path_copy: String = String::from(users_file_path);

  let mut message_queue = message::MessageQueue::new("/home/runner/mudnix/message_queue");
  let mut interval = time::interval(Duration::from_millis(100));
  EventStream! {
    if starting_world_location != "invalid" {
      loop {
        let fresh_user_list = user::UserList::from_file(&users_file_path_copy);
        let i = fresh_user_list.get_index_if_valid_creds(&username_copy, &password_hash).unwrap();
        let current_location = fresh_user_list.users[i].world_location.clone();
        interval.tick().await;
        message_queue.flush_queue();
        let messages = message_queue.get_messages(&current_location);
        if messages.len() > 0 {
          yield Event::data(serde_json::json!({
            "username": username_copy,
            "succeeded": true,
            "queue": messages
          }).to_string());
        }
      }
    } else {
      yield Event::data(serde_json::json!({
        "username": username_copy,
        "succeeded": false,
        "err": "You are not logged in."
      }).to_string());
    }
  }
}

#[get("/whos-here?<username>&<password>")]
pub fn whos_here(
  username: &str,
  password: &str,
  users_file_path_mutex: &State<mudnix_utils::UsersFileMutex>
) -> content::Json<String> {
  let users_file_path: &str = &users_file_path_mutex.mutex
    .lock().unwrap().to_string();
  let mut user_list = user::UserList::from_file(users_file_path);
  let password_hash = mudnix_utils::hash(password);
  if let Some(i) = user_list.get_index_if_valid_creds(username, &password_hash) {
    let location_id = user_list.users[i].world_location.clone();
    let location = match world_map::WorldLocation::from_location_id(&location_id) {
      Ok(location) => location,
      Err(_) => return mudnix_utils::error_response(
        username,
        &format!(
          "you are currently located at invalid location \"{}\"",
          location_id
        )
      )
    };

    let subloc = world_map::get_sublocation_from_id(&location_id).unwrap();
    let nearby_users: Vec<String> = match location.get_users_from_sublocation(&subloc) {
      Ok(users) => users,
      Err(_) => return mudnix_utils::error_response(
        username,
        &format!(
          "unable to get users at invalid location \"{}\"",
          location_id
        )
      )
    };
    user_list.update_timestamp_of_index(i);
    user_list.save_to_file(users_file_path);
    content::Json(serde_json::json!({
      "username": username,
      "succeeded": true,
      "active_location": location_id,
      "nearby_users": nearby_users
    }).to_string())
  } else {
    mudnix_utils::error_response(username, "invalid credentials")
  }
}
