use env_logger;
use log::{debug, info};
use pewcraft_common::game::{GameMap, GameState, Id};
use pewcraft_common::io::WireCreatedGame;
use std::io::{stdin, stdout};

mod api;
mod tui;

/*
#[derive(Debug)]
pub enum JoinedGameSubstate {
    CreateCharacter,
}

#[derive(Debug)]
pub struct JoinedGameState<'a> {
    game_id: String,
    game_state: GameState,
    map: &'a GameMap,
    substate: JoinedGameSubstate,
}
*/

#[derive(Debug)]
pub enum CreatingGameSubstate {
    CreateCharacter,
    WaitingForOtherPlayers,
}

#[derive(Debug)]
pub struct CreatingGameState<'a> {
    game_id: String,
    map: &'a GameMap,
    substate: CreatingGameSubstate,
}

#[derive(Debug)]
pub enum GlobalState<'a> {
    //CreateOrJoin,
    SelectMap(Vec<Id<GameMap>>, usize),
    CreatingGame(CreatingGameState<'a>),
    //JoinedGame(JoinedGameState<'a>),
    Exit,
}

impl<'a> GlobalState<'a> {
    pub fn join_game(created_game: WireCreatedGame, map: &'a GameMap) -> Self {
        GlobalState::CreatingGame(CreatingGameState {
            game_id: created_game.0,
            substate: CreatingGameSubstate::CreateCharacter,
            map,
        })
    }
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
    let mut stdout = stdout();
    let mut tui = tui::Tui::new(&game, &stdin, &mut stdout);
    let mut state_machine = GlobalState::SelectMap(map_ids, 0);

    loop {
        debug!("Current state: {:?}", state_machine);
        let input = tui.render(&state_machine);
        debug!("Received input: {:?}", input);

        match (state_machine, input) {
            (_, Input::Exit) => break,
            (unchanged, Input::Other) => state_machine = unchanged,
            (GlobalState::SelectMap(map_ids, mut curr_id), Input::Right) => {
                if curr_id == map_ids.len() - 1 {
                    curr_id = 0
                } else {
                    curr_id += 1;
                }
                state_machine = GlobalState::SelectMap(map_ids, curr_id);
            }
            (GlobalState::SelectMap(map_ids, mut curr_id), Input::Left) => {
                if curr_id == 0 {
                    curr_id = map_ids.len() - 1;
                } else {
                    curr_id -= 1;
                }
                state_machine = GlobalState::SelectMap(map_ids, curr_id);
            }
            (GlobalState::SelectMap(map_ids, curr_id), Input::Confirm) => {
                let map_id = *map_ids.get(curr_id).unwrap();
                // TODO hardcoded team size
                // TODO this can fail :)
                let created_game = endpoint.create_game(map_id, 2);
                let map = game.maps.get(map_id).unwrap();
                state_machine = GlobalState::join_game(created_game, map);
            }
            (GlobalState::CreatingGame(created_game), _) => {
                state_machine = GlobalState::CreatingGame(created_game)
            }
            _ => unimplemented!(),
        }
    }

    info!("Exiting tui");
}
