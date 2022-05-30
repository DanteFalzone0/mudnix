use std::fs;
use std::path::Path;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use rand::{distributions::Alphanumeric, Rng};

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
  pub text: String,
  pub timestamp: u64,
  pub user: String,
  pub location_id: String
}

impl Message {
  pub fn new(text: &str, user: &str, location_id: &str) -> Self {
    Self {
      text: String::from(text),
      timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
        .unwrap().as_secs(),
      user: String::from(user),
      location_id: String::from(location_id)
    }
  }

  pub fn from_file(file_path: &str) -> Self {
    let original_json = fs::read_to_string(file_path).unwrap();
    serde_json::from_str(&original_json).expect("unable to parse json")
  }

  // messages expire after ten seconds
  pub fn is_not_expired(&self) -> bool {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
      .unwrap().as_secs();
    now - self.timestamp < 10
  }

  pub fn is_expired(&self) -> bool {
    !self.is_not_expired()
  }
}

pub struct MessageQueue {
  // path of directory where files are kept
  message_queue_path: String,

  // file paths of messages
  message_paths: Vec<String>
}

impl MessageQueue {
  pub fn new(message_queue_path: &str) -> Self {
    let mut message_paths: Vec<String> = vec![];
    let path = Path::new(message_queue_path);
    for entry in fs::read_dir(path).expect("Unable to read message files") {
      let entry = entry.expect("unable to get entry");
      message_paths.push(format!("{}", entry.path().display()));
    }
    Self {
      message_queue_path: String::from(message_queue_path),
      message_paths
    }
  }

  pub fn send_message(&mut self, message: Message) {
    let random_string: String = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(8)
      .map(char::from)
      .collect();

    // example: 1000212360@Quux_Plains::northern_region@ASk87Fy3.json
    let out_file_path = format!(
      "{}/{}@{}@{}.json",
      self.message_queue_path,
      message.timestamp,
      message.location_id,
      random_string
    );

    let output_json = serde_json::to_string_pretty(&message).unwrap();
    fs::write(&out_file_path, output_json).expect("unable to save message");

    self.message_paths.push(out_file_path);
  }

  // Remove expired messages from the queue and add new messages
  pub fn flush_queue(&mut self) {
    let path = Path::new(&self.message_queue_path);
    for entry in fs::read_dir(path).expect("Unable to read message files") {
      let entry = entry.expect("unable to get entry");
      let entry_path = format!("{}", entry.path().display());
      if !self.message_paths.iter().any(|p| p == &entry_path) {
        self.message_paths.push(entry_path);
      }
    }

    for path in self.message_paths.iter_mut() {
      if Path::new(&path).exists() {
        if Message::from_file(&path).is_expired() {
          fs::remove_file(&path).expect("unable to remove file");
        }
      }
    }
    self.message_paths.retain(|path| Path::new(&path).exists());
  }

  pub fn get_messages(&self, location_id: &str) -> Vec<Message> {
    let mut result: Vec<Message> = vec![];
    for path in self.message_paths.iter() {
      let message = Message::from_file(&path);
      if message.location_id == location_id /*&& message.is_not_expired()*/ {
        result.push(message);
      }
    }
    result
  }
}
