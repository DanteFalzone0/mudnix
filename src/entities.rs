use std::time::SystemTime;
use std::fs;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub username: String,
  pub password_hash: String, // SHA-256 hash
  pub world_location: String,
  pub last_activity_timestamp: u64 // seconds since Unix epoch
}

impl User {
  pub fn new(username: &str, password_hash: &str, world_location: &str) -> Self {
    Self {
      username: username.to_string(),
      password_hash: password_hash.to_string(),
      world_location: world_location.to_string(),
      last_activity_timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
        .unwrap().as_secs()
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct UserList {
  pub users: Vec<User>
}

impl UserList {
  pub fn new() -> Self {
    Self {
      users: vec![]
    }
  }

  pub fn contains(&self, username: &str) -> bool {
    self.users.iter().any(|user| user.username == username)
  }

  pub fn get_index_if_valid_creds(
    &self,
    username: &str,
    password_hash: &str
  ) -> Option<usize> {
    self.users.iter().position(|user|
      user.username == username && user.password_hash == password_hash 
    )
  }

  pub fn update_timestamp_of_index(&mut self, i: usize) {
    self.users[i].last_activity_timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
  }

  pub fn from_file(file_path: &str) -> Self {
    let original_json = fs::read_to_string(file_path).unwrap();
    serde_json::from_str(&original_json).expect("unable to parse json")
  }

  pub fn save_to_file(&self, output_file_path: &str) {
    let output_json = serde_json::to_string_pretty(self).unwrap();
    fs::write(output_file_path, output_json)
      .expect("unable to save user");
  }

  pub fn remove_user_if_exists(&mut self, username: &str) {
    if let Some(i) = self.users.iter().position(|user| user.username == username) {
      self.users.swap_remove(i);
    }
  }
}
