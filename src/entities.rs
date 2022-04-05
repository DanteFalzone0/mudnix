use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct User {
  pub username: String,
  pub password_hash: String, // SHA-256 hash
  pub world_location: String
}

impl User {
  pub fn new(username: &str, password_hash: &str, world_location: &str) -> Self {
    Self {
      username: username.to_string(),
      password_hash: password_hash.to_string(),
      world_location: world_location.to_string()
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
}
