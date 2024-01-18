use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{window_size, Clear, ClearType, WindowSize};
use errno::errno;
use std::time::Duration;
use std::{io, io::Write};

const VERSION: &str = "0.0.1";

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

fn process_movement(terminal_state: &mut EditorState, key: KeyEvent) {
    let mut stdout = io::stdout();
    match key.code {
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::NONE) => {
            terminal_state.cy = terminal_state.cy + 1;
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::NONE) => {
            if terminal_state.cx > 0 {
                terminal_state.cx = terminal_state.cx - 1;
            }
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::NONE) => {
            if terminal_state.cy > 0 {
                terminal_state.cy = terminal_state.cy - 1;
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::NONE) => {
            terminal_state.cx = terminal_state.cx + 1;
        }
        _ => {}
    }

    crossterm::execute!(
        stdout,
        MoveTo(terminal_state.cx as u16, terminal_state.cy as u16)
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
            KeyCode::Char('w') | KeyCode::Char('a') | KeyCode::Char('s') | KeyCode::Char('d') => {
                process_movement(terminal_state, c)
            }
            _ => {}
        },
        None => {} // Handle the case where there's no character
    }

    //    if let Some(c) = c {
    //        if c.code == KeyCode::Char('q') && c.modifiers.contains(KeyModifiers::CONTROL) {
    //            return Ok(true); // Exit the loop
    //        } else {
    //            println!("{c:?}\r");
    //        }
    //    }

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
        if process_char(&mut term)? {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    refresh_screen();
    Ok(())
}
