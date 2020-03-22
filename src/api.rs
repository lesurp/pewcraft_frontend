use log::{debug, info};
use pewcraft_common::game::{GameDefinition, GameMap, Id};
use pewcraft_common::io::{WireCreatedGame, WireNewGameRequest};
use reqwest::{blocking::Client, Url};
use std::fmt;

pub struct Endpoint {
    url: Url,
    client: Client,
}

impl Endpoint {
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        info!("API endpoint: {}", url.as_ref());
        Endpoint {
            url: Url::parse(url.as_ref()).unwrap(),
            client: Client::new(),
        }
    }

    pub fn load_game(&self) -> GameDefinition {
        self.client
            .get(self.url.join("game").unwrap())
            .send()
            .unwrap()
            .json()
            .unwrap()
    }

    pub fn create_game(&self, map: Id<GameMap>, team_size: usize) -> WireCreatedGame {
        let new_game_request = WireNewGameRequest { map, team_size };
        debug!("Creating game with request: {:?}", new_game_request);
        self.client
            .post(self.url.join("new_game").unwrap())
            .json(&new_game_request)
            .send()
            .unwrap()
            .json()
            .unwrap()
    }
}

impl fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Endpoint {{ url: {:?}, client: <hidden> }}", self.url)
    }
}
