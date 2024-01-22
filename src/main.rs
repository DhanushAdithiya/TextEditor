use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{size, Clear, ClearType};
use errno::errno;
use std::time::Duration;
use std::{io, io::Write};

const VERSION: &str = "0.0.1";

#[derive(Debug, PartialEq)]
enum EditorMode {
    NORMAL,
    INSERT,
}

#[derive(Debug)]
struct WindowSize {
    rows: u16,
    columns: u16,
}

#[derive(Debug)]
struct EditorState {
    dimensions: WindowSize,
    cx: usize,
    cy: usize,
    mode: EditorMode,
}

impl EditorState {
    fn new() -> Self {
        if let Ok((height, width)) = size() {
            let dimensions = WindowSize {
                rows: width - 1,
                columns: height - 1,
            };

            Self {
                dimensions,
                cx: 1,
                cy: 1,
                mode: EditorMode::NORMAL,
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

fn display_editor_mode(terminal_state: &EditorState) {
    let mut stdout = io::stdout();

    crossterm::execute!(stdout, MoveTo(2, terminal_state.dimensions.rows)).unwrap();
    write!(stdout, "{:?}", terminal_state.mode).unwrap();
    crossterm::execute!(
        stdout,
        MoveTo(terminal_state.cx as u16 - 1, terminal_state.cy as u16 - 1)
    )
    .unwrap();
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

fn process_movement(terminal_state: &mut EditorState, key: KeyEvent) {
    // TODO - ADD NORMAL MODE AND INSERT MODE
    let mut stdout = io::stdout();
    match key.code {
        KeyCode::Char('j') => {
            if terminal_state.cy <= terminal_state.dimensions.rows.into() {
                println!(
                    "  {:?}:{:?}\r",
                    terminal_state.cy, terminal_state.dimensions.rows
                );
                terminal_state.cy = terminal_state.cy + 1;
            }
        }
        KeyCode::Char('h') => {
            if terminal_state.cx > 0 {
                terminal_state.cx = terminal_state.cx - 1;
            }
        }
        KeyCode::Char('k') => {
            if terminal_state.cy > 0 {
                terminal_state.cy = terminal_state.cy - 1;
            }
        }
        KeyCode::Char('l') => {
            if terminal_state.cx != terminal_state.dimensions.columns as usize {
                terminal_state.cx = terminal_state.cx + 1;
            }
        }
        _ => {}
    }
    crossterm::execute!(
        stdout,
        MoveTo(terminal_state.cx as u16 - 1, terminal_state.cy as u16 - 1)
    )
    .unwrap();
}

fn process_char(terminal_state: &mut EditorState) -> io::Result<bool> {
    let mut c = None;
    match poll(Duration::from_millis(100)) {
        Ok(true) => match read_character() {
            Some(key) => c = Some(key),
            None => {}
        },
        _ => {
            let msg = errno();
            match msg.to_string().as_str() {
                "Success" | "Resource temporarily unavailable" => {}
                _ => die("poll failed"),
            }
        }
    }

    match c {
        Some(c) => match c.code {
            KeyCode::Char('q') if c.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(true); // Exit the loop
            }
            KeyCode::Char('h') | KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Char('l')
                if terminal_state.mode == EditorMode::NORMAL =>
            {
                process_movement(terminal_state, c)
            }
            KeyCode::Char('i') => {
                terminal_state.mode = EditorMode::INSERT;
            }
            KeyCode::Esc => {
                terminal_state.mode = EditorMode::NORMAL;
            }
            _ => {}
        },
        None => {} // Handle the case where there's no character
    }

    Ok(false) // Continue the loop
}

fn editor_draw_rows(terminal_state: &EditorState) {
    let mut buffer = String::new();
    let mut stdout = io::stdout();
    for i in 0..terminal_state.dimensions.rows {
        if i == terminal_state.dimensions.rows / 3 {
            let welcome_str = format!("BREAD EDITOR - VERSION : {VERSION}");
            let w = (terminal_state.dimensions.columns as usize - welcome_str.len()) / 2;
            let padding = format!("~{:width$}", " ", width = w);
            buffer.push_str(&padding);
            buffer.push_str(&welcome_str);
        } else {
            buffer.push_str("~");
        }

        if i < terminal_state.dimensions.rows - 1 {
            buffer.push_str("\r\n");
        }
    }
    write!(stdout, "{buffer}").unwrap();
    crossterm::execute!(io::stdout(), MoveTo(0, 0)).unwrap();
}

fn refresh_screen() {
    crossterm::execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    crossterm::execute!(io::stdout(), MoveTo(0, 0)).unwrap();
}

fn main() -> io::Result<()> {
    let mut term = EditorState::new();
    crossterm::terminal::enable_raw_mode()?;
    refresh_screen();
    editor_draw_rows(&term);

    loop {
        display_editor_mode(&term);
        if process_char(&mut term)? {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    refresh_screen();
    Ok(())
}
