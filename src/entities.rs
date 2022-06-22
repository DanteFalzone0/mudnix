//extern crate rand;
//use crate::rand::Rng;
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use serde_json;

pub type Inventory = Vec<Item>;
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
      if self[i].qty > 1 {
        self[i].qty -= 1;
      } else {
        self.swap_remove(i);
      }
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
  // TODO
}
