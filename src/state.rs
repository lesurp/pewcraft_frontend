use crate::api;
use log::debug;
use pewcraft_common::game::{Cell, Class, GameDefinition, GameMap, GameState, Id, Team};
use pewcraft_common::io::WireCreatedGame;

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
    CreateGame(CreateGameState<'a>),
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
            (unchanged, Input::Other) => unchanged,

            /* SelectMap */
            (GlobalState::SelectMap(mut s), Input::Right) => {
                if s.curr().curr_id == s.curr().map_ids.len() - 1 {
                    s.curr_mut().curr_id = 0
                } else {
                    s.curr_mut().curr_id += 1;
                }
                GlobalState::SelectMap(s)
            }
            (GlobalState::SelectMap(mut s), Input::Left) => {
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
                let created_game = s.prev().endpoint.create_game(map_id, 2);
                let map = s.prev().game.maps.get(map_id).unwrap();
                GlobalState::join_game(created_game, map, s)
            }

            (GlobalState::CreateGame(c), i) => c.next(i),
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

            classes: s.prev().game.classes.ids(),
            teams: map
                .teams
                .iter()
                .enumerate()
                .map(|(index, _)| Id::new(index))
                .collect(),
            map,
        };

        let create_game_state_data = CreateGameStateData::new(
            s.split().0,
            CreateGameStateDataImpl {
                game_id: created_game.0,
                map,
            },
        );

        let state_data = StateData::new(create_game_state_data, create_character_state_data);
        let create_character_state = CreateCharacterState::Team(state_data);
        let create_game_state = CreateGameState::CreateCharacter(create_character_state);
        GlobalState::CreateGame(create_game_state)
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
    Name(CreateCharacterStateData<'a>),
}
impl<'a> CreateCharacterState<'a> {
    pub fn next(self, i: Input) -> GlobalState<'a> {
        match (self, i) {
            // FIRST CHOOSE THE TEAM
            (CreateCharacterState::Team(mut s), Input::Right) => {
                if s.curr().team_index == s.curr().teams.len() - 1 {
                    s.curr_mut().team_index = 0;
                } else {
                    s.curr_mut().team_index += 1;
                }
                GlobalState::CreateGame(CreateGameState::CreateCharacter(
                    CreateCharacterState::Team(s),
                ))
            }
            (CreateCharacterState::Team(mut s), Input::Left) => {
                if s.curr_mut().team_index == 0 {
                    s.curr_mut().team_index = s.curr().teams.len() - 1;
                } else {
                    s.curr_mut().team_index -= 1;
                }
                GlobalState::CreateGame(CreateGameState::CreateCharacter(
                    CreateCharacterState::Team(s),
                ))
            }
            (CreateCharacterState::Team(s), Input::Confirm) => GlobalState::CreateGame(
                CreateGameState::CreateCharacter(CreateCharacterState::Class(s)),
            ),

            // THEN THE CLASS
            (CreateCharacterState::Class(mut s), Input::Right) => {
                if s.curr().class_index == s.curr().classes.len() - 1 {
                    s.curr_mut().class_index = 0;
                } else {
                    s.curr_mut().class_index += 1;
                }
                GlobalState::CreateGame(CreateGameState::CreateCharacter(
                    CreateCharacterState::Class(s),
                ))
            }
            (CreateCharacterState::Class(mut s), Input::Left) => {
                if s.curr_mut().class_index == 0 {
                    s.curr_mut().class_index = s.curr().classes.len() - 1;
                } else {
                    s.curr_mut().class_index -= 1;
                }
                GlobalState::CreateGame(CreateGameState::CreateCharacter(
                    CreateCharacterState::Class(s),
                ))
            }
            (CreateCharacterState::Class(s), Input::Confirm) => GlobalState::CreateGame(
                CreateGameState::CreateCharacter(CreateCharacterState::Name(s)),
            ),

            // THEN THE NAME
            (CreateCharacterState::Name(mut s), Input::PrintableChar(c)) => {
                s.curr_mut().name.push(c);
                GlobalState::CreateGame(CreateGameState::CreateCharacter(
                    CreateCharacterState::Name(s),
                ))
            }
            (CreateCharacterState::Name(mut s), Input::Backspace) => {
                s.curr_mut().name.pop();
                GlobalState::CreateGame(CreateGameState::CreateCharacter(
                    CreateCharacterState::Name(s),
                ))
            }
            (CreateCharacterState::Name(s), Input::Confirm) => {
                debug!("Creating character with name {}", s.curr().name);
                GlobalState::CreateGame(CreateGameState::WaitingForOtherPlayers(s.split().0))
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

    pub classes: Vec<Id<Class>>,
    pub teams: Vec<Id<Team>>,
    pub map: &'a GameMap,
}
pub type CreateCharacterStateData<'a> =
    StateData<CreateGameStateData<'a>, CreateCharacterStateDataImpl<'a>>;

#[derive(Debug)]
pub enum CreateGameState<'a> {
    CreateCharacter(CreateCharacterState<'a>),
    WaitingForOtherPlayers(CreateGameStateData<'a>),
}
impl<'a> CreateGameState<'a> {
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
}
#[derive(Debug)]
pub struct CreateGameStateDataImpl<'a> {
    pub game_id: String,
    pub map: &'a GameMap,
}
pub type CreateGameStateData<'a> = StateData<GlobalStateData<'a>, CreateGameStateDataImpl<'a>>;

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
