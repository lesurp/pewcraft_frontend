use crate::{Input, State};
use pewcraft_common::game::{GameDefinition, GameMap, Id};
use std::fmt::{self, Display, Formatter};
use std::io::{Stdin, StdinLock, Stdout, Write};
use termion::{
    cursor::{DetectCursorPos, Goto},
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};

struct Format<'a, T>(&'a T, (u16, u16));
impl<'a> Display for Format<'a, GameMap> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut line = 0;
        let mut nl = move || {
            line += 1;
            Goto((self.1).0, (self.1).1 + line)
        };

        write!(
            f,
            "Name: {}{}Width: {}{}Height: {}{}Max number of teams: {}",
            self.0.name,
            nl(),
            self.0.width,
            nl(),
            self.0.height,
            nl(),
            self.0.teams.len(),
        )
        //Max number of character per team: {}\n",
    }
}

pub struct Tui<'a> {
    game_definition: &'a GameDefinition,
    stdin: Keys<StdinLock<'a>>,
    stdout: AlternateScreen<RawTerminal<std::io::StdoutLock<'a>>>,
}

impl<'a> Tui<'a> {
    pub fn new(game_definition: &'a GameDefinition, stdin: &'a Stdin, stdout: &'a Stdout) -> Self {
        let stdout = AlternateScreen::from(stdout.lock().into_raw_mode().unwrap());
        let stdin = stdin.lock().keys();
        Tui {
            game_definition,
            stdin,
            stdout,
        }
    }

    pub fn render(&mut self, s: &State) -> Input {
        self.pre_render();

        #[cfg(debug_assertions)]
        write!(self.stdout, "[DEBUG BUILD ONLY] Current state: {:?}", s).unwrap();

        match s {
            State::CreateCharacter => unimplemented!(),
            State::SelectMap(map_ids, curr_id) => self.select_map(map_ids, *curr_id),
            State::Exit => panic!("Should not try to render when in the 'Exit' state"),
        }

        self.get_input()
    }

    pub fn pre_render(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            termion::clear::All,
            termion::style::Reset,
            Goto(1, 1)
        )
        .unwrap()
    }

    pub fn get_input(&mut self) -> Input {
        let b = &self.stdin.next().unwrap().unwrap();
        match b {
            // Quit
            Key::Char('q') => Input::Exit,
            Key::Char('l') => Input::Right,
            Key::Char('h') => Input::Left,
            Key::Char('\n') => Input::Confirm,
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
            "{}Select your map!{}{}{}/{}{}{}{}",
            Goto(5, 5),
            termion::cursor::Down(1),
            termion::style::Invert,
            curr_id + 1,
            map_ids.len(),
            termion::style::NoInvert,
            termion::cursor::Down(1),
            termion::cursor::Hide,
        )
        .unwrap();
        let pos = self.stdout.cursor_pos().unwrap();
        write!(self.stdout, "{}", Format(curr_map, pos)).unwrap();
        self.stdout.flush().unwrap();
    }
}
