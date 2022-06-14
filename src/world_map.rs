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
  pub neighbors: Vec<String>,
  pub active_users: Vec<String>,
  pub npcs: Vec<entities::Npc>
}

impl SubLocation {
  pub fn is_neighbor(&self, location_id: &str) -> bool {
    self.neighbors.iter().any(|neighbor| neighbor == location_id)
  }
}

#[derive(Serialize, Deserialize)]
pub struct WorldLocationAttrs {
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
    let result = match dest.move_user_to_self(&self.username, dest_location_id) {
      Ok(r) => r,
      Err(e) => return Err(e)
    };
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

  pub fn move_user_to_self(
    &mut self,
    username: &str,
    location_id: &str
  ) -> Result<String, io::Error> {
    let loc_parts: Vec<&str> = location_id.split("::").collect();
    if loc_parts.len() == 1 {
      Err(io::Error::new(io::ErrorKind::InvalidInput, "no sublocation specified"))
    } else {
      if let Some(i) = self.attrs.sublocations.iter().position(|p| p.name == loc_parts[1]) {
        if !self.attrs.sublocations[i].active_users.iter().any(|user| user == username) {
          self.attrs.sublocations[i].active_users.push(String::from(username));
        }
        Ok(format!(
          "You are at {}.",
          location_id_to_human_readable(location_id)
        ))
      } else {
        Ok(format!(
          "{}: no such sublocation in {}",
          loc_parts[1].replace("_", " "),
          loc_parts[0].replace("_", " ")
        ))
      }
    }
  }

  pub fn remove_user(&mut self, username: &str) {
    for sublocation in self.attrs.sublocations.iter_mut() {
      if let Some(i) = sublocation.active_users.iter().position(|user| user == username) {
        sublocation.active_users.swap_remove(i);
        break;
      }
    }
  }

  pub fn move_user_from(&mut self, username: &str, src_location: &str) -> MovementResult {
    self.remove_user(username);
    self.save_to_file(&get_path_from_location_id(src_location));
    MovementResult { username: String::from(username) }
  }

  pub fn sublocation_index(&self, sublocation_id: &str) -> Result<usize, io::Error> {
    if let Some(i) = self.attrs.sublocations.iter().position(|sl| sl.name == sublocation_id) {
      Ok(i)
    } else {
      Err(io::Error::new(io::ErrorKind::InvalidInput, "sublocation does not exist"))
    }
  }

  pub fn get_users_from_sublocation(
    &self,
    sublocation_id: &str
  ) -> Result<Vec<String>, io::Error> {
    let i = match self.sublocation_index(sublocation_id) {
      Ok(i) => i,
      Err(e) => return Err(e)
    };
    Ok(self.attrs.sublocations[i].active_users.clone())
  }
}

pub fn get_path_from_location_id(location_id: &str) -> String {
  format!(
    "/home/runner/mudnix/map/{}.json",
    get_parent_location_from_id(location_id)
  )
}

pub fn location_id_to_human_readable(location_id: &str) -> String {
  let loc_parts: Vec<&str> = location_id.split("::").collect();
  if loc_parts.len() == 1 {
    String::from(loc_parts[0].replace("_", " "))
  } else {
    format!(
      "the {} of {}",
      loc_parts[1].replace("_", " "),
      loc_parts[0].replace("_", " ")
    )
  }
}

/**
 * This function returns the parent location of a location ID.
 * get_parent_location_from_id("foo::bar") will evaluate to "foo".
 */
pub fn get_parent_location_from_id(location_id: &str) -> String {
  let loc_parts: Vec<&str> = location_id.split("::").collect();
  String::from(loc_parts[0])
}

pub fn get_sublocation_from_id(location_id: &str) -> Result<String, io::Error> {
  let loc_parts: Vec<&str> = location_id.split("::").collect();
  if loc_parts.len() >= 2 {
    Ok(String::from(loc_parts[1]))
  } else {
    Err(io::Error::new(io::ErrorKind::InvalidInput, "location id has no sublocation"))
  }
}
