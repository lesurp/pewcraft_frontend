use log::{debug, info};
use pewcraft_common::game::{Cell, Character, GameDefinition, GameMap, Id, IdMap};
use std::fmt::{self, Display, Formatter};
use termion::{
    cursor::{DetectCursorPos, Down, Goto},
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};

struct CellCorner(char, char, char, char);
impl CellCorner {
    pub fn from_map_size(x: u16, y: u16, width: usize, height: usize) -> Self {
        let max_x = width as u16 - 1;
        let max_y = height as u16 - 1;

        let tl = match (x, y) {
            (0, 0) => '╔',
            (0, _) => '╠',
            (_, 0) => '╦',
            (_, _) => '╬',
        };

        let tr = if x == max_x {
            if y == 0 {
                '╗'
            } else {
                '╣'
            }
        } else if y == 0 {
            '╦'
        } else {
            '╬'
        };

        let bl = if x == 0 {
            if y == max_y {
                '╚'
            } else {
                '╠'
            }
        } else if y == max_y {
            '╩'
        } else {
            '╬'
        };

        let br = if x == max_x {
            if y == max_y {
                '╝'
            } else {
                '╣'
            }
        } else if y == max_y {
            '╩'
        } else {
            '╬'
        };

        CellCorner(tl, tr, bl, br)
    }
}

struct FormatCell<'a>(&'a Cell, Option<&'a Character>, CellCorner, (u16, u16));
impl<'a> Display for FormatCell<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cell = self.0;
        let character = self.1;
        let corners = &self.2;
        let pos = self.3;

        let mut line = 0;
        let mut nl = move || {
            line += 1;
            //debug!("Cell pos: {}/{}", pos.0, pos.1 + line);
            Goto(pos.0, pos.1 + line)
        };

        write!(
            f,
            "{}═════════════{}{}║             ║{}║             ║{}║             ║{}║             ║{}║             ║{}{}═════════════{}",
            corners.0,
            corners.1,
            nl(),
            nl(),
            nl(),
            nl(),
            nl(),
            nl(),
            corners.2,
            corners.3
        )
    }
}

pub(super) struct FormatMap<'a>(
    pub(super) &'a GameMap,
    pub(super) Option<&'a IdMap<Character>>,
    pub(super) (u16, u16),
);
impl<'a> FormatMap<'a> {
    fn character(&self, id: Id<Cell>) -> Option<&'a Character> {
        match self.1 {
            None => None,
            Some(characters) => characters
                .iter()
                .find(|(_, c)| c.position == id)
                .map(|(_, c)| c),
        }
    }
}
impl<'a> Display for FormatMap<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let map = &self.0;
        let ref_pos = &self.2;

        let mut id = Id::new(0);
        let mut out = String::new();
        for cell in map.data.iter() {
            let (x, y) = map.id_to_xy(id);
            let (x, y) = (x as u16, y as u16);
            debug!("Offset of curr cell: {}/{}", x, y);

            // convert our map coordinates to the screen coordinates (top left corner of each cell)
            let pos = (ref_pos.0 + 14 * x as u16, ref_pos.1 + 6 * y as u16);
            // get the character in the cell (if any)
            let character = self.character(id);
            let corners = CellCorner::from_map_size(x, y, map.width, map.height);

            out = format!(
                "{}{}{}",
                out,
                Goto(pos.0, pos.1),
                FormatCell(cell, character, corners, pos),
            );

            id = Id::new(id.raw() + 1);
        }

        write!(f, "{}", out)
    }
}

pub(super) struct FormatMapSelection<'a>(pub(super) &'a GameMap, pub(super) (u16, u16));
impl<'a> Display for FormatMapSelection<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut line = 0;
        let map = &self.0;
        let pos = &self.1;
        let mut nl = move || {
            line += 1;
            Goto(pos.0, pos.1 + line)
        };

        // TODO: add nb of char per team!
        write!(
            f,
            "Name: {}{}Width: {}{}Height: {}{}Max number of teams: {}",
            map.name,
            nl(),
            map.width,
            nl(),
            map.height,
            nl(),
            map.teams.len(),
        )
    }
}
