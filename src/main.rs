use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{size, Clear, ClearType};
use errno::errno;
use std::time::Duration;
use std::{env, io, io::{Write, Read}, fs::File,};

const VERSION: &str = "0.0.1";

#[derive(Debug, PartialEq)]
enum EditorMode {
    NORMAL,
    INSERT,
}


#[derive(Debug)]
struct Erow {
   size: usize,
   chars: String,
}

impl Erow {
    fn new() -> Self {
        Self {
            size: 0,
            chars: String::from(""),
        }
    }
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
    row: Erow,
    numrows: u16,
}

impl EditorState {
    fn new() -> Self {
        if let Ok((height, width)) = size() {
            let dimensions = WindowSize {
                rows: width - 1,
                columns: height - 1,
            };

            let row = Erow::new();

            Self {
                dimensions,
                cx: 1,
                cy: 1,
                mode: EditorMode::NORMAL,
                numrows: 0,
                row,
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
    let mut stdout = io::stdout();
    match key.code {
        KeyCode::Char('j') => {
            if terminal_state.cy <= terminal_state.dimensions.rows as usize - 1 {
                terminal_state.cy = terminal_state.cy + 1;
            }
        }
        KeyCode::Char('h') => {
            if terminal_state.cx > 1 {
                terminal_state.cx = terminal_state.cx - 1;
            }
        }
        KeyCode::Char('k') => {
            if terminal_state.cy > 1 {
                terminal_state.cy = terminal_state.cy - 1;
            }
        }
        KeyCode::Char('l') => {
            if terminal_state.cx < terminal_state.dimensions.columns as usize {
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

fn normal_mode_shortcuts(terminal_state: &mut EditorState, key: char) {
    match key {
        '$' => {
            terminal_state.cx = terminal_state.dimensions.columns.into();
            move_cursor(terminal_state);
        }
        '_' => {
            terminal_state.cx = 1;
            move_cursor(terminal_state);
        }
        _ => {}
    }
}

fn move_cursor(terminal_state: &mut EditorState) {
    crossterm::execute!(io::stdout(), MoveTo(terminal_state.cx as u16, terminal_state.cy as u16)).unwrap();
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
            KeyCode::Char(c) if terminal_state.mode == EditorMode::NORMAL => {
                normal_mode_shortcuts(terminal_state, c);
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
        if i >= terminal_state.numrows {
            if i == terminal_state.dimensions.rows / 3 {
                let welcome_str = format!("BREAD EDITOR - VERSION : {VERSION}");
                let w = (terminal_state.dimensions.columns as usize - welcome_str.len()) / 2;
                let padding = format!("~{:width$}", " ", width = w);
                buffer.push_str(&padding);
                buffer.push_str(&welcome_str);
            } else {
                buffer.push_str("~");
            }
        } else {
            let mut len = terminal_state.row.size as u16;
            if len > terminal_state.dimensions.columns {
                len = terminal_state.dimensions.columns;
            }
            let text = format!("{}",terminal_state.row.chars);
            buffer.push_str("HELLO");
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


fn editor_open(terminal_state: &mut EditorState, filename: &str){
    if let Ok(mut f) = File::open(filename) {
        let mut buffer = String::new();
        f.read_to_string(&mut buffer);
        terminal_state.numrows = 1;
        terminal_state.row.chars = buffer.lines().nth(0).unwrap().to_string(); // bad error handling
        terminal_state.row.size = buffer.len();

    } else {
        die("File not found, please check directory");
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut term = EditorState::new();
    crossterm::terminal::enable_raw_mode()?;
    refresh_screen();
    if args.len() > 2 {
        editor_open(&mut term, &args[2]);
    }
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
