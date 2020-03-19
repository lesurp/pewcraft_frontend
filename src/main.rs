use std::io::{stdin, stdout, Read, Write};
use termion::color;
use termion::raw::IntoRawMode;

mod api;

fn main() {
    // Initialize 'em all.
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();

    let url = "http://localhost:8000";
    let endpoint = api::Endpoint::new(url);
    let game = endpoint.load_game();

    let map_ids = game.maps.ids();
    let max_id = map_ids.len() as i32;
    let mut curr_id = 0 as i32;

    let mut bytes = stdin.bytes();
    loop {
        let curr_map = game
            .maps
            .get(*map_ids.get(curr_id as usize).unwrap())
            .unwrap();
        write!(
            stdout,
            "{}{}Select your map!\n{}{}\n{:?}",
            termion::clear::All,
            termion::style::Reset,
            termion::style::Bold,
            curr_id,
            curr_map,
        )
        .unwrap();
        stdout.flush().unwrap();

        let b = bytes.next().unwrap().unwrap();

        match b {
            // Quit
            b'q' => return,

            b'l' => {
                curr_id = (curr_id + 1) % max_id;
            }

            b'h' => {
                curr_id = (curr_id - 1) % max_id;
            }
            _ => {}
        }
    }
}
