use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Biome {
  pub eco: String,
  pub urban: bool
}

#[derive(Serialize, Deserialize)]
pub struct Place {
  pub name: String,
  pub t: String,
  pub active_users: Vec<String>
}

#[derive(Serialize, Deserialize)]
pub struct WorldLocationAttrs {
  pub neighbors: Vec<String>,
  pub active_users: Vec<String>,
  pub treasure_chest_spawn_rate: f32,
  pub biome: Biome,
  pub places: Vec<Place>
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
  pub fn to(&self, dest_location: &str) -> Result<String, io::Error> {
    let mut dest = match WorldLocation::from_location(dest_location) {
      Ok(dwl) => dwl,
      Err(e) => return Err(e)
    };
    let result = dest.move_user_to_self(&self.username, dest_location);
    dest.save_to_file(&get_path_from_location(dest_location));
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

  pub fn from_location(location: &str) -> Result<Self, io::Error> {
    Self::from_file(&get_path_from_location(location))
  }

  pub fn save_to_file(&self, output_file_path: &str) {
    let output_json = serde_json::to_string_pretty(self).unwrap();
    fs::write(output_file_path, output_json)
      .expect("unable to save world location");
  }

  pub fn move_user_to_self(&mut self, username: &str, location: &str) -> String {
    let loc_parts: Vec<&str> = location.split("::").collect();
    if loc_parts.len() == 1 {
      if !self.attrs.active_users.iter().any(|user| user == username) {
        self.attrs.active_users.push(String::from(username));
      }
      format!("You are at {}.", location.replace("_", " "))
    } else {
      if let Some(i) = self.attrs.places.iter().position(|p| p.name == loc_parts[1]) {
        if !self.attrs.places[i].active_users.iter().any(|user| user == username) {
          self.attrs.places[i].active_users.push(String::from(username));
        }
        format!(
          "You are at the {} of {}.",
          loc_parts[1].replace("_", " "),
          loc_parts[0].replace("_", " ")
        )
      } else {
        format!(
          "{}: no such place in {}",
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
      for place in self.attrs.places.iter_mut() {
        if let Some(i) = place.active_users.iter().position(|user| user == username) {
          place.active_users.swap_remove(i);
          break;
        }
      }
    }
  }

  pub fn move_user_from(&mut self, username: &str, src_location: &str) -> MovementResult {
    self.remove_user(username);
    self.save_to_file(&get_path_from_location(src_location));
    MovementResult { username: String::from(username) }
  }
}

pub fn get_path_from_location(location: &str) -> String {
  let parts: Vec<&str> = location.split("::").collect();
  format!("/home/runner/mudnix/map/{}.json", parts[0])
}
