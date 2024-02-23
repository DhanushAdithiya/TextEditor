use std::io;

use crossterm::{
    event::{read, Event::Key},
    terminal::ClearType,
    QueueableCommand,
};

use crate::editor::*;

pub fn read_character() -> Option<crossterm::event::KeyEvent> {
    if let Ok(e) = read() {
        if let Key(key_event) = e {
            return Some(key_event);
        }
    }
    panic!("read failed");
}

pub fn normal_mode_shortcuts(terminal_state: &mut EditorState, key: char) {
    match key {
        'i' => {
            terminal_state.mode = EditorMode::INSERT;
            if terminal_state.numrows == 0 {
                // This is to prevent out of bounds error when we create a new file and try to append text to it.
                let new_row = Erow::new();
                terminal_state.row.push(new_row);
                let mut stdout = io::stdout();
                stdout
                    .queue(crossterm::terminal::Clear(ClearType::All))
                    .unwrap();
                terminal_state.editor_append_row(String::new(), 0);
                terminal_state.row[terminal_state.cy].editor_update_row();
            }
        }
        '$' => {
            if (terminal_state.row[terminal_state.cy as usize].size) > 0 {
                terminal_state.cx = (terminal_state.row[terminal_state.cy].size) - 1;
            } else {
                terminal_state.cx = 0;
            }

            terminal_state.move_cursor();
        }
        '_' => {
            terminal_state.cx = 0;
            terminal_state.move_cursor();
        }
        'w' => {
            // REALLY DISGUSTING CODE
            if terminal_state.row[terminal_state.cy]
                .chars
                .chars()
                .nth(terminal_state.cx)
                .unwrap()
                .is_whitespace()
                && !terminal_state.row[terminal_state.cy]
                    .chars
                    .chars()
                    .nth(terminal_state.cx + 1)
                    .unwrap()
                    .is_whitespace()
            {
                terminal_state.cx += 1;
            }
            if terminal_state.row[terminal_state.cy]
                .chars
                .chars()
                .nth(terminal_state.cx)
                .unwrap()
                .is_whitespace()
                || terminal_state.row[terminal_state.cy]
                    .chars
                    .chars()
                    .nth(terminal_state.cx + 1)
                    .unwrap()
                    .is_whitespace()
            {
                while terminal_state.row[terminal_state.cy]
                    .chars
                    .chars()
                    .nth(terminal_state.cx + 1)
                    .unwrap()
                    .is_whitespace()
                {
                    terminal_state.cx += 1;
                }
            } else {
                let mut iter = terminal_state.row[terminal_state.cy].chars[terminal_state.cx..]
                    .split_whitespace();

                iter.next();
                if let Some(n_word) = iter.next() {
                    terminal_state.cx = terminal_state.row[terminal_state.cy].chars
                        [terminal_state.cx..]
                        .find(n_word)
                        .unwrap()
                        + terminal_state.cx;
                }
            }
        }
        'b' => {
            if terminal_state.row[terminal_state.cy]
                .chars
                .chars()
                .nth(terminal_state.cx)
                .unwrap()
                .is_whitespace()
                && !terminal_state.row[terminal_state.cy]
                    .chars
                    .chars()
                    .nth(terminal_state.cx - 1)
                    .unwrap()
                    .is_whitespace()
            {
                terminal_state.cx -= 1;
            }
            if terminal_state.row[terminal_state.cy]
                .chars
                .chars()
                .nth(terminal_state.cx)
                .unwrap()
                .is_whitespace()
                || terminal_state.row[terminal_state.cy]
                    .chars
                    .chars()
                    .nth(terminal_state.cx - 1)
                    .unwrap()
                    .is_whitespace()
            {
                while terminal_state.row[terminal_state.cy]
                    .chars
                    .chars()
                    .nth(terminal_state.cx - 1)
                    .unwrap()
                    .is_whitespace()
                {
                    terminal_state.cx -= 1;
                }
            } else {
                let mut iter = terminal_state.row[terminal_state.cy].chars[..terminal_state.cx]
                    .split_whitespace();

                iter.next();
                if let Some(n_word) = iter.next() {
                    terminal_state.cx = terminal_state.row[terminal_state.cy].chars
                        [..terminal_state.cx]
                        .find(n_word)
                        .unwrap()
                }
            }
        }
        'j' => {
            if terminal_state.numrows > 0 && terminal_state.cy < terminal_state.numrows as usize - 1
            {
                // The -1 is required as the dimensions are 0 indexed.
                let next_line = terminal_state.row[terminal_state.cy as usize + 1].rsize;
                terminal_state.cy = terminal_state.cy + 1;
                if terminal_state.cx > next_line {
                    terminal_state.cx = next_line;
                }
            }
            terminal_state.move_cursor();
        }
        'h' => {
            if terminal_state.cx > 0 {
                terminal_state.cx = terminal_state.cx - 1;
            }
            terminal_state.move_cursor();
        }
        'k' => {
            if terminal_state.cy > 0 {
                let prev_line = terminal_state.row[terminal_state.cy as usize - 1].rsize;
                terminal_state.cy -= 1;
                if terminal_state.cx > prev_line {
                    terminal_state.cx = prev_line;
                }
            }
            terminal_state.move_cursor();
        }
        'l' => {
            let line = terminal_state.row[terminal_state.cy as usize].rsize;
            if terminal_state.cx <= line {
                terminal_state.cx += 1;
            }
            terminal_state.move_cursor();
        }
        _ => {}
    }
}
