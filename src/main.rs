#[macro_use] extern crate rocket;
extern crate hex;
extern crate rand;
use std::sync::Mutex;
use rocket::fs::FileServer;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::time::{self, Duration};
use serde_json;

mod entities;
mod user;
mod world_map;
mod mudnix_utils;
mod message;
mod game_endpoints;
mod user_endpoints;

// https://stackoverflow.com/a/69342225/10942736
pub struct CORS;
#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Attaching CORS headers to responses",
      kind: Kind::Response
    }
  }

  async fn on_response<'r>(
    &self,
    _request: &'r Request<'_>,
    response: &mut Response<'r>
  ) {
    response.set_header(
      Header::new("Access-Control-Allow-Origin", "*")
    );
    response.set_header(
      Header::new(
        "Access-Control-Allow-Methods",
        "POST, GET, PATCH, OPTIONS"
      )
    );
    response.set_header(
      Header::new("Access-Control-Allow-Headers", "*")
    );
    response.set_header(
      Header::new("Access-Control-Allow-Credentials", "true")
    );
  }
}

#[get("/version")]
fn version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

#[get("/check-connection")]
fn check_connection() -> EventStream![] {
  EventStream! {
    let mut interval = time::interval(Duration::from_secs(1));
    let mut i = 0;
    loop {
      yield Event::data(serde_json::json!({
        "alive": true,
        "count": i
      }).to_string());
      i += 1;
      interval.tick().await;
    }
  }
}

#[launch]
fn rocket() -> _ {
  rocket::build()
    .manage(mudnix_utils::UsersFileMutex {
      mutex: Mutex::new(String::from("/home/runner/mudnix/users.json"))
    })
    .manage(mudnix_utils::LoggedInUserPool {
      user_list_mutex: Mutex::new(user::UserList::new())
    })
    .attach(CORS)
    .mount("/", FileServer::from("/home/runner/mudnix/static"))
    .mount("/", routes![version, check_connection])
    .mount("/hash", routes![mudnix_utils::hash])
    .mount("/user", routes![
      user_endpoints::new_user,
      user_endpoints::login,
      user_endpoints::logout,
      user_endpoints::inventory,
      user_endpoints::autologout
    ])
    .mount("/game", routes![
      game_endpoints::teleport,
      game_endpoints::goto,
      game_endpoints::map,
      game_endpoints::close_chest,
      game_endpoints::say,
      game_endpoints::get_messages,
      game_endpoints::whos_here
    ])
}
