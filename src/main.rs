use env_logger;
use log::{debug, info};
use pewcraft_common::game::{GameMap, Id};
use std::io::{stdin, stdout};

mod api;
mod tui;

#[derive(Debug)]
pub enum State {
    //CreateOrJoin,
    SelectMap(Vec<Id<GameMap>>, usize),
    CreateCharacter,
    Exit,
}

#[derive(Debug)]
pub enum Input {
    Exit,
    Left,
    Right,
    Confirm,
    Other,
}

fn main() {
    env_logger::init();

    let url = "http://localhost:8000";
    let endpoint = api::Endpoint::new(url);
    let game = endpoint.load_game();

    let map_ids = game.maps.ids();

    let stdin = stdin();
    let stdout = stdout();
    let mut tui = tui::Tui::new(&game, &stdin, &stdout);
    let mut state_machine = State::SelectMap(map_ids, 0);

    loop {
        debug!("Current state: {:?}", state_machine);
        let input = tui.render(&state_machine);
        debug!("Received input: {:?}", input);

        match (state_machine, input) {
            (_, Input::Exit) => break,
            (unchanged, Input::Other) => state_machine = unchanged,
            (State::SelectMap(map_ids, mut curr_id), Input::Right) => {
                if curr_id == map_ids.len() - 1 {
                    curr_id = 0
                } else {
                    curr_id += 1;
                }
                state_machine = State::SelectMap(map_ids, curr_id);
            }
            (State::SelectMap(map_ids, mut curr_id), Input::Left) => {
                if curr_id == 0 {
                    curr_id = map_ids.len() - 1;
                } else {
                    curr_id -= 1;
                }
                state_machine = State::SelectMap(map_ids, curr_id);
            }
            (State::SelectMap(map_ids, curr_id), Input::Confirm) => {
                let map_id = *map_ids.get(curr_id).unwrap();
                // TODO hardcoded team size
                endpoint.create_game(map_id, 2);
                state_machine = State::CreateCharacter;
            }
            _ => unimplemented!(),
        }
    }

    info!("Exiting tui");
}
