use std::sync::Mutex;

pub struct FilePathMutex {
  pub mutex: Mutex<String>
}
