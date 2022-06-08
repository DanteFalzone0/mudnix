//extern crate rand;
//use crate::rand::Rng;
use std::time::SystemTime;
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use serde_json;

type Inventory = Vec<Item>;
pub trait ItemContainer {
  fn add_item(&mut self, item: &Item);
  fn remove_item(&mut self, item: &Item);
}
impl ItemContainer for Inventory {
  fn add_item(&mut self, item: &Item) {
    if let Some(i) = self.iter().position(|_item| _item.t == item.t) {
      self[i].qty += 1;
    } else {
      self.push(item.clone());
    }
  }

  fn remove_item(&mut self, item: &Item) {
    if let Some(i) = self.iter().position(|_item| _item.t == item.t) {
      if self[i].qty > 0 {
        self[i].qty -= 1;
      } else {
        self.swap_remove(i);
      }
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub username: String,
  pub password_hash: String, // SHA-256 hash
  pub inventory: Inventory,
  pub active_treasure_chest: Option<TreasureChest>,
  pub world_location: String,
  pub last_activity_timestamp: u64, // seconds since Unix epoch
  pub account_creation_timestamp: u64
}

impl User {
  pub fn new(username: &str, password_hash: &str, world_location: &str) -> Self {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
      .unwrap().as_secs();

    Self {
      username: username.to_string(),
      password_hash: password_hash.to_string(),
      inventory: vec![],
      active_treasure_chest: None,
      world_location: world_location.to_string(),
      last_activity_timestamp: now,
      account_creation_timestamp: now
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

#[derive(Serialize, Deserialize)]
pub struct Npc {
  pub name: String,
  // TODO finish implementation
}

#[derive(Serialize, Deserialize)]
pub struct ActionResult {
  pub info: String,
  pub succeeded: bool,
  pub data: serde_json::Value
}

pub trait Entity {
  fn inspect(&self) -> ActionResult;
  fn action(&mut self, verb: &str) -> ActionResult {
    ActionResult {
      info: format!("{}:<Entity>", verb),
      succeeded: true,
      data: serde_json::json!({
        "info": "Entity::action() not yet implemented for this type"
      })
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Item {
  pub t: String,
  pub qty: u32,
  pub name: String,
  pub description: String,
  pub rarity: String
}

impl Item {
  pub fn path_of(item_type: &str) -> String {
    format!("/home/runner/mudnix/items/{}.json", item_type)
  }

  // pub fn new(item_type: &str, name: &str, description: &str, rarity: &str) -> Self {
  //   Self {
  //     t: String::from(item_type),
  //     qty: 1,
  //     name: String::from(name),
  //     description: String::from(description),
  //     rarity: String::from(rarity)
  //   }
  // }

  pub fn from_file(file_path: &str) -> Result<Self, io::Error> {
    let original_json = match fs::read_to_string(file_path) {
      Ok(file_contents) => file_contents,
      Err(e) => return Err(e)
    };
    Ok(serde_json::from_str(&original_json).expect("unable to parse json"))
  }
}

impl Entity for Item {
  fn inspect(&self) -> ActionResult {
    ActionResult {
      info: String::from("inspect:Item"),
      succeeded: true,
      data: serde_json::value::to_value(self).unwrap()
    }
  }

  // fn action(&mut self, verb: &str) -> ActionResult {
  // TODO
  // }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TreasureChest {
  pub contents: Inventory
}

impl TreasureChest {
  pub fn new() -> Self {
    let mut result = TreasureChest { contents: vec![] };
    // TODO randomize item spawning
    result.contents.add_item(
      &Item::from_file(&Item::path_of("bar_of_soap")).unwrap()
    );
    result
  }
}

impl Entity for TreasureChest {
  fn inspect(&self) -> ActionResult {
    ActionResult {
      info: String::from("inspect:TreasureChest"),
      succeeded: true,
      data: serde_json::json!({
        "description": "The surface is made of real wood. It does not appear to be a mimic."
      })
    }
  }
}
