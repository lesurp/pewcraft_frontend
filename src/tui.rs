use pewcraft_common::game::{GameDefinition, GameMap, Id};
use std::fmt::{self, Display, Formatter};
use std::io::{Bytes, Read, Stdin, StdinLock, Stdout, Write};
//use termion::color;
use crate::state_machine::{Input, StateMachine};
use termion::{
    cursor::{Down, Left},
    raw::{IntoRawMode, RawTerminal},
};
//use std::di

struct Format<'a, T>(&'a T);
impl<'a> Display for Format<'a, GameMap> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let prev_out = 0;
        let out = format!("Name: {}{}", self.0.name, Down(1));

        let out = format!(
            "{}{}Width: {}{}",
            out,
            Left((out.len() - prev_out) as u16),
            self.0.width,
            Down(1)
        );
        let prev_out = out.len();

        let out = format!(
            "{}{}Height: {}{}",
            out,
            Left((out.len() - prev_out) as u16),
            self.0.height,
            Down(1)
        );
        let prev_out = out.len();

        let out = format!(
            "{}{}Max number of teams: {}{}",
            out,
            Left((out.len() - prev_out) as u16),
            self.0.teams.len(),
            Down(1)
        );
        //let move_left = (out.len() - move_left as usize) as u16;

        write!(f, "{}", out)

        //Max number of character per team: {}\n",
    }
}

pub struct Tui<'a> {
    game_definition: &'a GameDefinition,
    stdin: Bytes<StdinLock<'a>>,
    stdout: RawTerminal<std::io::StdoutLock<'a>>,
}

impl<'a> Tui<'a> {
    pub fn new(game_definition: &'a GameDefinition, stdin: &'a Stdin, stdout: &'a Stdout) -> Self {
        let stdout = stdout.lock().into_raw_mode().unwrap();
        let stdin = stdin.lock().bytes();
        Tui {
            game_definition,
            stdin,
            stdout,
        }
    }

    pub fn render(&mut self, s: &StateMachine) -> Input {
        match s {
            StateMachine::SelectMap(map_ids, curr_id) => self.select_map(map_ids, *curr_id),
            StateMachine::Exit => panic!("Should not try to render when in the 'Exit' state"),
        }

        self.get_input()
    }

    pub fn get_input(&mut self) -> Input {
        let b = self.stdin.next().unwrap().unwrap();
        match b {
            // Quit
            b'q' => Input::Exit,
            b'l' => Input::Right,
            b'h' => Input::Left,
            _ => Input::Other,
        }
    }

    pub fn select_map(&mut self, map_ids: &[Id<GameMap>], curr_id: usize) {
        let curr_map = self
            .game_definition
            .maps
            .get(*map_ids.get(curr_id).unwrap())
            .unwrap();

        write!(
            self.stdout,
            "{}{}{}Select your map!\n{}/{}\n{}",
            termion::clear::All,
            termion::style::Reset,
            termion::cursor::Goto(5, 5),
            curr_id + 1,
            map_ids.len(),
            Format(curr_map),
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }
}
