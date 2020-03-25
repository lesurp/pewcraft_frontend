use crate::api;
use log::debug;
use pewcraft_common::game::{Cell, Character, Class, GameDefinition, GameMap, GameState, Id, Team};
use pewcraft_common::io::{
    WireCreatedChar, WireCreatedGame, WireNewCharRequest, WireNewGameRequest,
};

#[derive(Debug)]
pub enum Input {
    Timeout,
    Exit,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Confirm,
    Other,
    PrintableChar(char),
}

#[derive(Debug)]
pub struct StateData<Prev, Curr>(Prev, Curr);
impl<P, C> StateData<P, C> {
    pub fn new(p: P, c: C) -> Self {
        StateData(p, c)
    }

    pub fn prev(&self) -> &P {
        &self.0
    }

    pub fn curr(&self) -> &C {
        &self.1
    }

    fn prev_mut(&mut self) -> &mut P {
        &mut self.0
    }

    fn curr_mut(&mut self) -> &mut C {
        &mut self.1
    }

    fn split(self) -> (P, C) {
        (self.0, self.1)
    }
}

#[derive(Debug)]
pub enum GlobalState<'a> {
    //CreateOrJoin,
    SelectMap(SelectMapData<'a>),
    WaitForGameCreation(WaitForGameCreationData<'a>),
    CreateCharacter(CreateCharacterState<'a>),
    PlayGame(PlayGameState<'a>),
    //JoinedGame(JoinedGameState<'a>),
    Exit,
}

#[derive(Debug)]
pub struct GlobalStateData<'a> {
    pub game: &'a GameDefinition,
    endpoint: &'a api::Endpoint,
}

impl<'a> GlobalState<'a> {
    pub fn exit(&self) -> bool {
        matches!(self, GlobalState::Exit)
    }

    pub fn new(game: &'a GameDefinition, endpoint: &'a api::Endpoint) -> Self {
        let global_state_data = GlobalStateData { game, endpoint };
        let select_map_state_data = SelectMapDataImpl {
            map_ids: game.maps.ids(),
            curr_id: 0,
        };

        GlobalState::SelectMap(SelectMapData::new(global_state_data, select_map_state_data))
    }

    pub fn next(self, i: Input) -> Self {
        match (self, i) {
            (_, Input::Exit) => GlobalState::Exit,
            unchanged @ (_, Input::Other) => unchanged.0,

            /* SelectMap */
            (GlobalState::SelectMap(mut s), Input::PrintableChar('l'))
            | (GlobalState::SelectMap(mut s), Input::Right) => {
                if s.curr().curr_id == s.curr().map_ids.len() - 1 {
                    s.curr_mut().curr_id = 0
                } else {
                    s.curr_mut().curr_id += 1;
                }
                GlobalState::SelectMap(s)
            }
            (GlobalState::SelectMap(mut s), Input::PrintableChar('h'))
            | (GlobalState::SelectMap(mut s), Input::Left) => {
                if s.curr().curr_id == 0 {
                    s.curr_mut().curr_id = s.curr().map_ids.len() - 1;
                } else {
                    s.curr_mut().curr_id -= 1;
                }
                GlobalState::SelectMap(s)
            }
            (GlobalState::SelectMap(s), Input::Confirm) => {
                let map_id = *s.curr().map_ids.get(s.curr().curr_id).unwrap();
                // TODO hardcoded team size
                // TODO this can fail :)
                let request = WireNewGameRequest {
                    map: map_id,
                    team_size: 2,
                };
                let created_game = s.prev().endpoint.create_game(request);
                let map = s.prev().game.maps.get(map_id).unwrap();
                GlobalState::join_game(created_game, map, s)
            }

            (GlobalState::CreateCharacter(c), i) => c.next(i),
            unchanged @ (GlobalState::WaitForGameCreation(_), _) => unchanged.0,
            (s, i) => {
                panic!("Input: {:?}\nState: {:?}", i, s);
            }
        }
    }

    pub fn join_game(
        created_game: WireCreatedGame,
        map: &'a GameMap,
        s: SelectMapData<'a>,
    ) -> GlobalState<'a> {
        let create_character_state_data = CreateCharacterStateDataImpl {
            name: String::new(),
            class_index: 0,
            team_index: 0,
            position_index: 0,

            classes: s.prev().game.classes.ids(),
            teams: map
                .teams
                .iter()
                .enumerate()
                .map(|(index, _)| Id::new(index))
                .collect(),
            map,
            game_id: created_game.0,
        };

        let state_data = StateData::new(s.split().0, create_character_state_data);
        let create_character_state = CreateCharacterState::Team(state_data);
        GlobalState::CreateCharacter(create_character_state)
    }
}

/*
#[derive(Debug)]
pub enum JoinedGameState {
    CreateCharacter,
}

#[derive(Debug)]
pub struct JoinedGameState<'a> {
    game_id: String,
    game_state: GameState,
    map: &'a GameMap,
    substate: JoinedGameState,
}
*/
#[derive(Debug)]
pub struct SelectMapDataImpl {
    pub map_ids: Vec<Id<GameMap>>,
    pub curr_id: usize,
}
pub type SelectMapData<'a> = StateData<GlobalStateData<'a>, SelectMapDataImpl>;

#[derive(Debug)]
pub enum CreateCharacterState<'a> {
    Class(CreateCharacterStateData<'a>),
    Team(CreateCharacterStateData<'a>),
    Position(CreateCharacterStateData<'a>),
    Name(CreateCharacterStateData<'a>),
}
impl<'a> CreateCharacterState<'a> {
    pub fn next(self, i: Input) -> GlobalState<'a> {
        match (self, i) {
            // FIRST CHOOSE THE TEAM
            (CreateCharacterState::Team(mut s), Input::PrintableChar('l'))
            | (CreateCharacterState::Team(mut s), Input::Right) => {
                if s.curr().team_index == s.curr().teams.len() - 1 {
                    s.curr_mut().team_index = 0;
                } else {
                    s.curr_mut().team_index += 1;
                }
                GlobalState::CreateCharacter(CreateCharacterState::Team(s))
            }
            (CreateCharacterState::Team(mut s), Input::PrintableChar('h'))
            | (CreateCharacterState::Team(mut s), Input::Left) => {
                if s.curr_mut().team_index == 0 {
                    s.curr_mut().team_index = s.curr().teams.len() - 1;
                } else {
                    s.curr_mut().team_index -= 1;
                }
                GlobalState::CreateCharacter(CreateCharacterState::Team(s))
            }
            (CreateCharacterState::Team(s), Input::Confirm) => {
                GlobalState::CreateCharacter(CreateCharacterState::Class(s))
            }

            // THEN THE CLASS
            (CreateCharacterState::Class(mut s), Input::PrintableChar('l'))
            | (CreateCharacterState::Class(mut s), Input::Right) => {
                if s.curr().class_index == s.curr().classes.len() - 1 {
                    s.curr_mut().class_index = 0;
                } else {
                    s.curr_mut().class_index += 1;
                }
                GlobalState::CreateCharacter(CreateCharacterState::Class(s))
            }
            (CreateCharacterState::Class(mut s), Input::PrintableChar('h'))
            | (CreateCharacterState::Class(mut s), Input::Left) => {
                if s.curr_mut().class_index == 0 {
                    s.curr_mut().class_index = s.curr().classes.len() - 1;
                } else {
                    s.curr_mut().class_index -= 1;
                }
                GlobalState::CreateCharacter(CreateCharacterState::Class(s))
            }
            (CreateCharacterState::Class(s), Input::Confirm) => {
                GlobalState::CreateCharacter(CreateCharacterState::Position(s))
            }

            // THEN THE POSITION
            (CreateCharacterState::Position(mut s), Input::PrintableChar('l'))
            | (CreateCharacterState::Position(mut s), Input::Right) => {
                let positions = &s.curr().map.teams.get(s.curr().team_index).unwrap().1;
                if s.curr().position_index == positions.len() - 1 {
                    s.curr_mut().position_index = 0;
                } else {
                    s.curr_mut().position_index += 1;
                }
                GlobalState::CreateCharacter(CreateCharacterState::Position(s))
            }
            (CreateCharacterState::Position(mut s), Input::PrintableChar('h'))
            | (CreateCharacterState::Position(mut s), Input::Left) => {
                let positions = &s.curr().map.teams.get(s.curr().team_index).unwrap().1;
                if s.curr_mut().position_index == 0 {
                    s.curr_mut().position_index = positions.len() - 1;
                } else {
                    s.curr_mut().position_index -= 1;
                }
                GlobalState::CreateCharacter(CreateCharacterState::Position(s))
            }
            (CreateCharacterState::Position(s), Input::Confirm) => {
                GlobalState::CreateCharacter(CreateCharacterState::Name(s))
            }

            // THEN THE NAME
            (CreateCharacterState::Name(mut s), Input::PrintableChar(c)) => {
                s.curr_mut().name.push(c);
                GlobalState::CreateCharacter(CreateCharacterState::Name(s))
            }
            (CreateCharacterState::Name(mut s), Input::Backspace) => {
                s.curr_mut().name.pop();
                GlobalState::CreateCharacter(CreateCharacterState::Name(s))
            }
            (CreateCharacterState::Name(s), Input::Confirm) => {
                debug!("Creating character with name {}", s.curr().name);
                let (global, create_char) = s.split();
                let name = create_char.name;
                let class = *create_char.classes.get(create_char.class_index).unwrap();
                let team = *create_char.teams.get(create_char.team_index).unwrap();
                let position = *create_char
                    .map
                    .teams
                    .get(team.raw())
                    .unwrap()
                    .1
                    .get(create_char.position_index)
                    .unwrap();
                let WireCreatedChar(login, id) = global.endpoint.create_char(
                    &create_char.game_id,
                    WireNewCharRequest {
                        name,
                        class,
                        team,
                        position,
                    },
                );
                GlobalState::WaitForGameCreation(WaitForGameCreationData::new(
                    global,
                    WaitForGameCreationDataImpl {
                        map: create_char.map,
                        game_id: create_char.game_id,
                        login,
                        id,
                    },
                ))
            }
            //
            _ => unimplemented!(),
        }
    }
}
#[derive(Debug)]
pub struct CreateCharacterStateDataImpl<'a> {
    pub name: String,
    pub class_index: usize,
    pub team_index: usize,
    pub position_index: usize,

    pub classes: Vec<Id<Class>>,
    pub teams: Vec<Id<Team>>,
    pub map: &'a GameMap,
    pub game_id: String,
}
pub type CreateCharacterStateData<'a> =
    StateData<GlobalStateData<'a>, CreateCharacterStateDataImpl<'a>>;

/*
    pub fn next(self, i: Input) -> GlobalState<'a> {
        match self {
            CreateGameState::WaitingForOtherPlayers(c) => {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let game_state = c.prev().endpoint.game_state(&c.curr().game_id);
                if let Some(game_state) = game_state {
                    GlobalState::PlayGame(PlayGameState::NotOurTurn(StateData::new(
                        c.split().0,
                        PlayGameStateDataImpl {
                            cell: Id::new(0),
                            game_state,
                        },
                    )))
                } else {
                    GlobalState::CreateGame(CreateGameState::WaitingForOtherPlayers(c))
                }
            }
            CreateGameState::CreateCharacter(c) => c.next(i),
        }
}
*/

#[derive(Debug)]
pub enum PlayGameState<'a> {
    OurTurn(PlayGameStateData<'a>),
    NotOurTurn(PlayGameStateData<'a>),
}
impl<'a> PlayGameState<'a> {
    pub fn next(self, i: Input) -> GlobalState<'a> {
        unimplemented!()
    }
}
#[derive(Debug)]
pub struct PlayGameStateDataImpl {
    pub cell: Id<Cell>,
    pub game_state: GameState,
}
pub type PlayGameStateData<'a> = StateData<GlobalStateData<'a>, PlayGameStateDataImpl>;

#[derive(Debug)]
pub struct WaitForGameCreationDataImpl<'a> {
    pub map: &'a GameMap,
    pub game_id: String,
    pub login: String,
    pub id: Id<Character>,
}
pub type WaitForGameCreationData<'a> =
    StateData<GlobalStateData<'a>, WaitForGameCreationDataImpl<'a>>;
