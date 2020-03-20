mod api;
mod state_machine;
mod tui;

use state_machine::StateMachine;
use std::io::{stdin, stdout};

fn main() {
    let url = "http://localhost:8000";
    let endpoint = api::Endpoint::new(url);
    let game = endpoint.load_game();

    let map_ids = game.maps.ids();
    dbg!(&map_ids);

    let stdin = stdin();
    let stdout = stdout();
    let mut tui = tui::Tui::new(&game, &stdin, &stdout);
    let mut state_machine = state_machine::StateMachine::SelectMap(map_ids, 0);

    loop {
        let input = tui.render(&state_machine);
        state_machine = state_machine.next(input);

        if matches!(state_machine, StateMachine::Exit) {
            break;
        }
    }
}
