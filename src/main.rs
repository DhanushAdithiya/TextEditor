use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{size, Clear, ClearType};
use std::time::Duration;
use std::{
    env,
    fs::File,
    io,
    io::{Read, Write},
};

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
    row: Vec<Erow>,
    numrows: u16,
    rowoff: u16,
}

impl EditorState {
    fn new() -> Self {
        let row = Vec::new();
        let dimensions = resize_terminal();

        Self {
            dimensions,
            cx: 0,
            cy: 0,
            mode: EditorMode::NORMAL,
            numrows: 0,
            row,
            rowoff: 0,
        }
    }
}

fn display_editor_mode(terminal_state: &mut EditorState) {
    let mut stdout = io::stdout();
    let dimensions = resize_terminal();

    crossterm::execute!(stdout, MoveTo(0, terminal_state.dimensions.rows)).unwrap();
    crossterm::execute!(stdout, Clear(ClearType::CurrentLine)).unwrap();
    terminal_state.dimensions = dimensions;

    crossterm::execute!(stdout, MoveTo(2, terminal_state.dimensions.rows)).unwrap();
    write!(stdout, "{:?}", terminal_state.mode).unwrap();
    crossterm::execute!(
        stdout,
        MoveTo(terminal_state.cx as u16, terminal_state.cy as u16)
    )
    .unwrap();
}

fn resize_terminal() -> WindowSize {
    if let Ok((height, width)) = size() {
        let dimensions = WindowSize {
            rows: width,
            columns: height,
        };

        return dimensions;
    } else {
        panic!("could not get terminal size");
    }
}

fn read_character() -> Option<KeyEvent> {
    if let Ok(e) = read() {
        if let Key(key_event) = e {
            return Some(key_event);
        } else {
            None
        }
    } else {
        panic!("read failed");
    }
}

fn process_movement(terminal_state: &mut EditorState, key: KeyEvent) {
    let mut stdout = io::stdout();
    match key.code {
        KeyCode::Char('j') => {
            if terminal_state.cy <= terminal_state.numrows as usize {
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
                terminal_state.cy -= 1;
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
        MoveTo(terminal_state.cx as u16, terminal_state.cy as u16)
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
    crossterm::execute!(
        io::stdout(),
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
        _ => {}
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
        None => {}
    }

    Ok(false)
}

fn editor_draw_rows(terminal_state: &EditorState) {
    let mut buffer = String::new();
    let mut stdout = io::stdout();
    for i in 0..terminal_state.dimensions.rows {
        let filerow = i + terminal_state.rowoff;
        if filerow >= terminal_state.numrows {
            if i == terminal_state.dimensions.rows / 3 && terminal_state.numrows == 0 {
                let welcome_str = format!("BREAD EDITOR - VERSION : {VERSION}");
                let w = (terminal_state.dimensions.columns as usize - welcome_str.len()) / 2;
                let padding = format!("~{:width$}", " ", width = w);
                buffer.push_str(&padding);
                buffer.push_str(&welcome_str);
            } else {
                buffer.push_str("~");
            }
        } else {
            let mut length = terminal_state.row[filerow as usize].size;
            if length > terminal_state.dimensions.columns as usize {
                length = terminal_state.dimensions.columns as usize;
            }
            buffer.push_str(&terminal_state.row[filerow as usize].chars[..length]);
        }

        if i < terminal_state.dimensions.rows - 1 {
            buffer.push_str("\r\n");
        }
    }
    write!(stdout, "{buffer}").unwrap();
    crossterm::execute!(io::stdout(), MoveTo(0, 0)).unwrap();
}

fn refresh_screen(terminal_state: &mut EditorState) {
    editor_scroll(terminal_state);
    crossterm::execute!(io::stdout(), crossterm::terminal::Clear(ClearType::All)).unwrap();
    crossterm::execute!(io::stdout(), crossterm::cursor::MoveTo(0, 0)).unwrap();
    editor_draw_rows(terminal_state);
}

fn editor_append_row(chars: String, length: usize, terminal_state: &mut EditorState) {
    let loc = terminal_state.numrows as usize;
    terminal_state.row[loc].size = length;
    terminal_state.row[loc].chars = chars;
    terminal_state.numrows += 1;
}

fn editor_scroll(terminal_state: &mut EditorState) {
    if terminal_state.cy < terminal_state.rowoff.into() {
        terminal_state.rowoff = terminal_state.cy as u16;
    }

    if terminal_state.cy >= (terminal_state.rowoff + terminal_state.dimensions.rows) as usize {
        terminal_state.rowoff =
            (terminal_state.cy as u16 - terminal_state.dimensions.rows + 1) as u16;
    }
}

fn editor_open(terminal_state: &mut EditorState, filename: &str) {
    if let Ok(mut f) = File::open(filename) {
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();

        buffer.lines().for_each(|l| {
            let new_row = Erow::new();
            terminal_state.row.push(new_row);
            editor_append_row(l.to_string(), l.len(), terminal_state);
        });
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut term = EditorState::new();
    crossterm::terminal::enable_raw_mode()?;
    refresh_screen(&mut term);
    if args.len() == 2 {
        editor_open(&mut term, &args[1]);
    }

    loop {
        display_editor_mode(&mut term);
        refresh_screen(&mut term);
        crossterm::execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(term.cx as u16, term.cy as u16 - term.rowoff),
        )
        .unwrap();
        if process_char(&mut term)? {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    refresh_screen(&mut term);
    Ok(())
}
