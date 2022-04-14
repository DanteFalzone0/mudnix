use std::time::SystemTime;
use serde::{Serialize, Deserialize};

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
}
