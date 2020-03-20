use pewcraft_common::game::{GameMap, Id};

pub enum StateMachine {
    //CreateOrJoin,
    SelectMap(Vec<Id<GameMap>>, usize),
    Exit,
}

impl StateMachine {
    pub fn next(self, i: Input) -> Self {
        match self {
            StateMachine::SelectMap(map_ids, curr_id) => StateMachine::select_map(i, map_ids, curr_id),
            StateMachine::Exit => panic!("Should not call next if the state is already 'Exit'. This is a logic error in your application!"),
        }
    }

    fn select_map(i: Input, map_ids: Vec<Id<GameMap>>, mut curr_id: usize) -> Self {
        match i {
            Input::Exit => return StateMachine::Exit,
            Input::Left => {
                if curr_id == 0 {
                    curr_id = map_ids.len() - 1;
                } else {
                    curr_id -= 1;
                }
            }
            Input::Right => {
                if curr_id == map_ids.len() - 1 {
                    curr_id = 0
                } else {
                    curr_id += 1;
                }
            }
            Input::Other => {}
        }

        StateMachine::SelectMap(map_ids, curr_id)
    }
}

pub enum Input {
    Exit,
    Left,
    Right,
    Other,
}
