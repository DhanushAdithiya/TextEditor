use std::{env, io};

mod editor;
mod keyboard;
mod terminal;

use editor::*;

const VERSION: &str = "0.0.1";
const TABSTOP: usize = 4;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut term = EditorState::new();
    term.dimensions.rows -= 1;
    crossterm::terminal::enable_raw_mode()?;
    term.refresh_screen();
    if args.len() == 2 {
        term.editor_open(&args[1]);
    }

    loop {
        term.refresh_screen();
        term.change_cursor()?;
        term.editor_status_line()?;

        crossterm::execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(term.rx as u16 - term.coloff, term.cy as u16 - term.rowoff),
        )
        .unwrap();

        if term.process_char()? {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    clear()?;
    Ok(())
}
