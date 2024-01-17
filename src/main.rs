use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{window_size, Clear, ClearType, WindowSize};
use errno::errno;
use std::time::Duration;
use std::{io, io::Write};

#[derive(Debug)]
struct EditorState {
    dimensions: WindowSize,
    cx: usize,
    cy: usize,
}

impl EditorState {
    fn new() -> Self {
        if let Ok(dimensions) = window_size() {
            Self {
                dimensions,
                cx: 0,
                cy: 0,
            }
        } else {
            panic!("Couldn't get terminal size");
        }
    }
}

fn die<M: Into<String>>(msg: M) {
    let _ = crossterm::terminal::disable_raw_mode();
    eprintln!("{}:{}", msg.into(), errno());
    std::process::exit(1);
}

fn read_character() -> Option<KeyEvent> {
    if let Ok(e) = read() {
        if let Key(key_event) = e {
            return Some(key_event);
        } else {
            None
        }
    } else {
        die("read failed");
        None
    }
}

fn editor_draw_rows(terminal_state: EditorState) {
    let mut stdout = io::stdout();
    for i in 0..terminal_state.dimensions.rows {
        write!(stdout, "~").unwrap();

        if i < terminal_state.dimensions.rows - 1 {
            write!(stdout, "\r\n").unwrap();
        }
    }
    crossterm::execute!(io::stdout(), MoveTo(0, 0)).unwrap();
}

fn refresh_screen() {
    crossterm::execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    crossterm::execute!(io::stdout(), MoveTo(0, 0)).unwrap();
}

fn main() -> io::Result<()> {
    let term = EditorState::new();
    crossterm::terminal::enable_raw_mode()?;
    refresh_screen();
    editor_draw_rows(term);

    loop {
        let mut c = None;

        crossterm::terminal::Clear(crossterm::terminal::ClearType::All);

        match poll(Duration::from_millis(100)) {
            Ok(true) => match read_character() {
                Some(key) => c = Some(key),
                None => {}
            },
            _ => {
                let msg = errno();

                //HORRBLE ERROR HANDLING HERE FIX IT
                match msg.to_string().as_str() {
                    "Success" => {
                        continue;
                    }
                    "Resource temporarily unavailable" => {
                        continue;
                    }
                    _ => {
                        die("pool failed");
                    }
                }
            }
        }

        if let Some(c) = c {
            if c.code == KeyCode::Char('q') && c.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            } else {
                println!("{c:?}\r");
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    refresh_screen();
    Ok(())
}
