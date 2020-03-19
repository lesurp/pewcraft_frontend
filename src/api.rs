use reqwest::{Url, blocking};
use pewcraft_common::game::GameDefinition;

pub struct Endpoint {
    url: Url,
}

impl Endpoint {
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        Endpoint { url: Url::parse(url.as_ref()).unwrap() }
    }

    pub fn load_game(&self) -> GameDefinition {
        blocking::get(self.url.join("/game").unwrap()).unwrap().json().unwrap()
    }
}
