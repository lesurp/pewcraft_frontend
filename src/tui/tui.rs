use crate::tui::map::FormatMap;
use crate::{CreatingGameState, CreatingGameSubstate, GlobalState, Input};
use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, info};
use pewcraft_common::game::{Cell, GameDefinition, GameMap, Id};
use std::fmt::{self, Display, Formatter};
use std::io::{Bytes, Read, Stdin, StdinLock, Stdout, StdoutLock, Write};
use tui::backend::CrosstermBackend;
use tui::layout::Alignment;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, Widget};
use tui::widgets::{Paragraph, Text};
use tui::Terminal;

const SELECT_MAP_BLOCK_TITLE: &str = "Select map";
const CREATING_CHARACTER_BLOCK_TITLE: &str = "Create character";

pub struct Tui<'a> {
    game_definition: &'a GameDefinition,
    //stdin: Bytes<StdinLock<'a>>,
    stdout: Terminal<CrosstermBackend<StdoutLock<'a>>>,
}

impl<'a> Tui<'a> {
    pub fn new(game_definition: &'a GameDefinition, _: &'a Stdin, stdout: &'a mut Stdout) -> Self {
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout.lock());
        //let stdin = stdin.lock().bytes();
        let stdout = Terminal::new(backend).unwrap();
        Tui {
            game_definition,
            //stdin,
            stdout,
        }
    }

    pub fn get_input(&mut self) -> Input {
        //let b = &self.stdin.next().unwrap().unwrap();
        match read().unwrap() {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Input::Exit,
                KeyCode::Char('l') => Input::Right,
                KeyCode::Char('h') => Input::Left,
                KeyCode::Enter => Input::Confirm,
                _ => Input::Other,
            },
            _ => Input::Other,
        }
    }

    pub fn render(&mut self, s: &GlobalState) -> Input {
        debug!("tui.rs:render");
        let game_definition = self.game_definition;
        self.stdout.hide_cursor().unwrap();
        self.stdout
            .draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.size());

                match s {
                    GlobalState::CreatingGame(created_game) => {
                        FormatMap(created_game.map, None).render(&mut f, chunks[0]);
                    }
                    //GlobalState::JoinedGame(joined_game) => self.render_joined_game(joined_game),
                    GlobalState::SelectMap(map_ids, curr_id) => {
                        let map = game_definition
                            .maps
                            .get(*map_ids.get(*curr_id).unwrap())
                            .unwrap();
                        FormatMap(map, None).render(&mut f, chunks[0]);

                        let text = [
                            Text::styled(
                                format!("    {} / {}", curr_id + 1, map_ids.len()),
                                Style::default().modifier(Modifier::BOLD),
                            ),
                            Text::raw("\n    Name:      "),
                            Text::raw(format!("{}", map.name)),
                            Text::raw("\n    Width:     "),
                            Text::raw(format!("{}", map.width)),
                            Text::raw("\n    Height:    "),
                            Text::raw(format!("{}", map.height)),
                            Text::raw("\n    Max teams: "),
                            Text::raw(format!("{}", map.teams.len())),
                        ];

                        Paragraph::new(text.iter())
                            .block(
                                Block::default()
                                    .title(SELECT_MAP_BLOCK_TITLE)
                                    .borders(Borders::ALL),
                            )
                            .alignment(Alignment::Left)
                            .render(&mut f, chunks[1]);
                    }
                    GlobalState::Exit => {
                        panic!("Should not try to render when in the 'Exit' state")
                    }
                };
            })
            .unwrap();

        debug!("Current state: {:?}", s);

        self.get_input()
    }

    pub fn render_created_game(created_game: &CreatingGameState) -> Block<'static> {
        Block::default()
            .title(CREATING_CHARACTER_BLOCK_TITLE)
            .borders(Borders::ALL)
        /*
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
        */
    }
}

impl<'a> Drop for Tui<'a> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.stdout.backend_mut(), LeaveAlternateScreen).unwrap();
    }
}
