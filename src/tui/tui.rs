use crate::tui::map::{FormatMap, FormatMapSelection};
use crate::{CreatingGameState, CreatingGameSubstate, GlobalState, Input};
use log::{debug, info};
use pewcraft_common::game::{Cell, GameDefinition, GameMap, Id};
use std::fmt::{self, Display, Formatter};
use std::io::{Stdin, StdinLock, Stdout, Write};
use termion::{
    clear,
    cursor::{self, DetectCursorPos, Down, Goto},
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
    style,
};

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

    pub fn render(&mut self, s: &GlobalState) -> Input {
        debug!("tui.rs:render");

        self.pre_render();
        debug!("Current state: {:?}", s);

        match s {
            GlobalState::CreatingGame(created_game) => self.render_created_game(created_game),
            //GlobalState::JoinedGame(joined_game) => self.render_joined_game(joined_game),
            GlobalState::SelectMap(map_ids, curr_id) => self.select_map(map_ids, *curr_id),
            GlobalState::Exit => panic!("Should not try to render when in the 'Exit' state"),
        }

        self.stdout.flush().unwrap();
        self.get_input()
    }

    pub fn render_created_game(&mut self, created_game: &CreatingGameState) {
        debug!("render_created_game");

        // used to align the character name prompt
        let pos_first_line = self.stdout.cursor_pos().unwrap();
        write!(self.stdout, "Game id: {}", &created_game.game_id,).unwrap();
        write!(
            self.stdout,
            "{}{}Choose your name: ",
            Goto(pos_first_line.0, pos_first_line.1),
            Down(1)
        )
        .unwrap();
        // now the prompt points after the "choose your name" - we save it for later
        let prompt_pos = self.stdout.cursor_pos().unwrap();

        // move the cursor to the map drawing area
        let beg_map = (5, 5);
        write!(self.stdout, "{}", Goto(beg_map.0, beg_map.1)).unwrap();

        // ... and draw the map
        match created_game.substate {
            _ => write!(
                self.stdout,
                "{}",
                FormatMap(created_game.map, None, beg_map)
            )
            .unwrap(),
        }

        // now go back to the prompt...
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Show,
            cursor::BlinkingBlock,
            Goto(prompt_pos.0, prompt_pos.1)
        )
        .unwrap();
    }

    pub fn pre_render(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}",
            clear::All,
            style::Reset,
            cursor::Hide,
            Goto(2, 2)
        )
        .unwrap()
    }

    pub fn select_map(&mut self, map_ids: &[Id<GameMap>], curr_id: usize) {
        debug!("select_map");

        let curr_map = self
            .game_definition
            .maps
            .get(*map_ids.get(curr_id).unwrap())
            .unwrap();

        write!(self.stdout, "Select your map:\t").unwrap();
        let pos_first_line = self.stdout.cursor_pos().unwrap();
        let pos = (pos_first_line.0, pos_first_line.1 + 1);

        write!(
            self.stdout,
            "{}{}/{}{}{}",
            termion::style::Invert,
            curr_id + 1,
            map_ids.len(),
            termion::style::NoInvert,
            termion::cursor::Goto(pos.0, pos.1),
        )
        .unwrap();
        write!(self.stdout, "{}", FormatMapSelection(curr_map, pos)).unwrap();
        write!(
            self.stdout,
            "{}{}",
            Goto(5, 5),
            FormatMap(curr_map, None, (5, 5))
        )
        .unwrap()
    }
}
