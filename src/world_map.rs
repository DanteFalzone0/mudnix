use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use serde_json;
use crate::entities;

#[derive(Serialize, Deserialize)]
pub struct Biome {
  pub eco: String,
  pub urban: bool
}

#[derive(Serialize, Deserialize)]
pub struct SubLocation {
  pub name: String,
  pub t: String,
  pub active_users: Vec<String>,
  pub npcs: Vec<entities::Npc>
}

#[derive(Serialize, Deserialize)]
pub struct WorldLocationAttrs {
  pub neighbors: Vec<String>,
  pub active_users: Vec<String>,
  pub npcs: Vec<entities::Npc>,
  pub treasure_chest_spawn_rate: f32,
  pub biome: Biome,
  pub sublocations: Vec<SubLocation>
}

#[derive(Serialize, Deserialize)]
pub struct WorldLocation {
  pub name: String,
  pub attrs: WorldLocationAttrs
}

// returned by WorldLocation::move_user_from
pub struct MovementResult {
  username: String
}

impl MovementResult {
  pub fn to(&self, dest_location_id: &str) -> Result<String, io::Error> {
    let mut dest = match WorldLocation::from_location_id(dest_location_id) {
      Ok(dwl) => dwl,
      Err(e) => return Err(e)
    };
    let result = dest.move_user_to_self(&self.username, dest_location_id);
    dest.save_to_file(&get_path_from_location_id(dest_location_id));
    Ok(result)
  }
}

impl WorldLocation {
  pub fn from_file(file_path: &str) -> Result<Self, io::Error> {
    let original_json = match fs::read_to_string(file_path) {
      Ok(file_contents) => file_contents,
      Err(e) => return Err(e)
    };
    Ok(serde_json::from_str(&original_json).expect("unable to parse json"))
  }

  pub fn from_location_id(location_id: &str) -> Result<Self, io::Error> {
    Self::from_file(&get_path_from_location_id(location_id))
  }

  pub fn save_to_file(&self, output_file_path: &str) {
    let output_json = serde_json::to_string_pretty(self).unwrap();
    fs::write(output_file_path, output_json)
      .expect("unable to save world location");
  }

  pub fn move_user_to_self(&mut self, username: &str, location_id: &str) -> String {
    let loc_parts: Vec<&str> = location_id.split("::").collect();
    if loc_parts.len() == 1 {
      if !self.attrs.active_users.iter().any(|user| user == username) {
        self.attrs.active_users.push(String::from(username));
      }
      format!("You are at {}.", location_id.replace("_", " "))
    } else {
      if let Some(i) = self.attrs.sublocations.iter().position(|p| p.name == loc_parts[1]) {
        if !self.attrs.sublocations[i].active_users.iter().any(|user| user == username) {
          self.attrs.sublocations[i].active_users.push(String::from(username));
        }
        format!(
          "You are at the {} of {}.",
          loc_parts[1].replace("_", " "),
          loc_parts[0].replace("_", " ")
        )
      } else {
        format!(
          "{}: no such sublocation in {}",
          loc_parts[1].replace("_", " "),
          loc_parts[0].replace("_", " ")
        )
      }
    }
  }

  pub fn remove_user(&mut self, username: &str) {
    if let Some(i) = self.attrs.active_users.iter().position(|user| user == username) {
      self.attrs.active_users.swap_remove(i);
    } else {
      for sublocation in self.attrs.sublocations.iter_mut() {
        if let Some(i) = sublocation.active_users.iter().position(|user| user == username) {
          sublocation.active_users.swap_remove(i);
          break;
        }
      }
    }
  }

  pub fn move_user_from(&mut self, username: &str, src_location: &str) -> MovementResult {
    self.remove_user(username);
    self.save_to_file(&get_path_from_location_id(src_location));
    MovementResult { username: String::from(username) }
  }

  pub fn is_neighbor(&self, location_id: &str) -> bool {
    let parts: Vec<&str> = location_id.split("::").collect();
    self.attrs.neighbors.iter().any(|neighbor| neighbor == parts[0])
  }
}

pub fn get_path_from_location_id(location_id: &str) -> String {
  let parts: Vec<&str> = location_id.split("::").collect();
  format!("/home/runner/mudnix/map/{}.json", parts[0])
}
