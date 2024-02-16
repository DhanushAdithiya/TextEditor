use crossterm::cursor::MoveTo;
use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{style, Attribute, Color, Stylize};
use crossterm::terminal::{size, Clear, ClearType};
use crossterm::QueueableCommand;
use io::Result;
use std::time::Duration;
use std::{env, fmt, fs::File, io, io::Read};

const VERSION: &str = "0.0.1";
const TABSTOP: usize = 4;

#[derive(Debug, PartialEq)]
enum EditorMode {
    NORMAL,
    INSERT,
}

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Erow {
    size: usize,
    chars: String,
    rsize: usize,
    render: String,
}

impl Erow {
    fn new() -> Self {
        Self {
            size: 0,
            chars: String::from(""),
            rsize: 0,
            render: String::from(""),
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
    rx: usize,
    mode: EditorMode,
    row: Vec<Erow>,
    numrows: u16,
    rowoff: u16,
    coloff: u16,
    filename: Option<String>,
}

impl EditorState {
    fn new() -> Self {
        let row = Vec::new();
        let dimensions = resize_terminal();

        Self {
            dimensions,
            cx: 0,
            cy: 0,
            rx: 0,
            mode: EditorMode::NORMAL,
            numrows: 0,
            row,
            rowoff: 0,
            coloff: 0,
            filename: None,
        }
    }
}

fn display_editor_mode(terminal_state: &mut EditorState) -> Result<()> {
    let mut stdout = io::stdout();
    let status;

    if let Some(filename) = &terminal_state.filename {
        let status_content = format!("{} | {}", terminal_state.mode, filename);
        let padding = format!(
            "~{:width$}",
            " ",
            width = terminal_state.dimensions.columns as usize - status_content.len() - 1
        );
        status = format!("{status_content}{}", padding)
            .with(Color::Black)
            .on(Color::White);
    } else {
        let padding = format!(
            "~{:width$}",
            " ",
            width =
                terminal_state.dimensions.columns as usize - terminal_state.mode.to_string().len()
        );
        status = format!("{}|{}", terminal_state.mode, padding)
            .with(Color::Black)
            .on(Color::White);
    }
    stdout
        .queue(crossterm::cursor::MoveTo(
            0,
            terminal_state.dimensions.rows + 1,
        ))?
        .queue(crossterm::style::Print(status))?;

    Ok(())
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
    //TODO
    match key.code {
        KeyCode::Char('j') => {
            if terminal_state.cy < terminal_state.numrows as usize - 1 {
                let next_line = terminal_state.row[terminal_state.cy as usize + 1].rsize;
                terminal_state.cy = terminal_state.cy + 1;
                if terminal_state.cx > next_line {
                    terminal_state.cx = next_line;
                }
            }
        }
        KeyCode::Char('h') => {
            if terminal_state.cx > 0 {
                terminal_state.cx = terminal_state.cx - 1;
            }
        }
        KeyCode::Char('k') => {
            if terminal_state.cy > 0 {
                let prev_line = terminal_state.row[terminal_state.cy as usize - 1].rsize;
                terminal_state.cy -= 1;
                if terminal_state.cx > prev_line {
                    terminal_state.cx = prev_line;
                }
            }
        }
        KeyCode::Char('l') => {
            let line = terminal_state.row[terminal_state.cy as usize].rsize;
            if terminal_state.cx < line {
                terminal_state.cx += 1;
            }
        }
        _ => {}
    }
}

fn normal_mode_shortcuts(terminal_state: &mut EditorState, key: char) {
    match key {
        '$' => {
            if (terminal_state.row[terminal_state.cy as usize].size) > 0 {
                terminal_state.cx = (terminal_state.row[terminal_state.cy].size) - 1;
            } else {
                terminal_state.cx = 0;
            }

            move_cursor(terminal_state);
        }
        '_' => {
            terminal_state.cx = 0;
            move_cursor(terminal_state);
        }
        'w' => {}
        'b' => {}
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

fn editor_draw_rows(terminal_state: &EditorState) -> Result<()> {
    let mut stdout = io::stdout();
    for i in 0..terminal_state.dimensions.rows {
        let filerow = i + terminal_state.rowoff;
        if filerow >= terminal_state.numrows {
            if i == terminal_state.dimensions.rows / 3 && terminal_state.numrows == 0 {
                let welcome_str = format!("BREAD EDITOR - VERSION : {VERSION}\r\n");
                let w = (terminal_state.dimensions.columns as usize - welcome_str.len()) / 2;
                let padding = format!("~{:width$}", " ", width = w);
                stdout
                    .queue(crossterm::style::Print(padding))?
                    .queue(crossterm::style::Print(welcome_str))?;
            } else {
                stdout.queue(crossterm::style::Print("~\r\n"))?;
            }
        } else {
            let mut len = terminal_state.row[filerow as usize].rsize;
            if len < terminal_state.coloff as usize {
                continue;
            }
            len -= terminal_state.coloff as usize;
            let start = terminal_state.coloff as usize;
            let end = start
                + if len >= terminal_state.dimensions.columns as usize {
                    terminal_state.dimensions.columns as usize
                } else {
                    len
                };
            stdout
                .queue(crossterm::cursor::MoveTo(0, i))?
                .queue(crossterm::style::Print(
                    &terminal_state.row[filerow as usize].render[start..end],
                ))?;
        }
    }
    //stdout.queue(crossterm::style::Print("\r\n"))?;
    Ok(())
}

fn editor_update_row(row: &mut Erow) {
    //let mut tabs = 0;
    //for i in 0..row.size {
    //    if row.chars.chars().nth(i).unwrap() == '\t' {
    //        tabs += 1;
    //    }
    //}

    let mut idx = 0;
    for i in 0..row.size {
        if row.chars.chars().nth(i).unwrap() == '\t' {
            row.render.push(' ');
            idx += 1;
            while idx % TABSTOP != 0 {
                row.render.push(' ');
                idx += 1
            }
        } else {
            row.render.push(row.chars.chars().nth(i).unwrap());
            idx += 1;
        }
    }
    row.rsize = idx;
}

fn editor_row_cx_to_rx(row: &mut Erow, cx: usize) -> usize {
    let mut rx = 0;
    for i in 0..cx {
        if let Some(c) = row.chars.chars().nth(i) {
            if c == '\t' {
                rx += (TABSTOP - 1) - (rx % TABSTOP);
            }
            rx += 1;
        }
    }

    rx
}

fn refresh_screen(terminal_state: &mut EditorState) {
    editor_scroll(terminal_state);
    clear().unwrap();
    editor_draw_rows(terminal_state).unwrap();
}

fn clear() -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .queue(crossterm::terminal::Clear(ClearType::All))?
        .queue(crossterm::cursor::MoveTo(0, 0))?;

    Ok(())
}

fn editor_append_row(chars: String, length: usize, terminal_state: &mut EditorState) {
    let loc = terminal_state.numrows as usize;
    terminal_state.row[loc].size = length;
    terminal_state.row[loc].chars = chars;
    terminal_state.numrows += 1;

    terminal_state.row[loc].rsize = 0;
    terminal_state.row[loc].render = String::new();
    editor_update_row(&mut terminal_state.row[loc]);
}

fn editor_scroll(terminal_state: &mut EditorState) {
    terminal_state.rx = 0;

    if terminal_state.cy < terminal_state.numrows.into() {
        terminal_state.rx = editor_row_cx_to_rx(
            &mut terminal_state.row[terminal_state.cy],
            terminal_state.cx,
        );
    }

    if terminal_state.cy < terminal_state.rowoff.into() {
        terminal_state.rowoff = terminal_state.cy as u16;
    }

    if terminal_state.rx < terminal_state.coloff.into() {
        terminal_state.coloff = terminal_state.rx as u16;
    }

    if terminal_state.cy >= (terminal_state.rowoff + terminal_state.dimensions.rows) as usize {
        terminal_state.rowoff =
            (terminal_state.cy as u16 - terminal_state.dimensions.rows + 1) as u16;
    }

    if terminal_state.rx >= (terminal_state.coloff + terminal_state.dimensions.columns) as usize {
        terminal_state.coloff =
            (terminal_state.rx as u16 - terminal_state.dimensions.columns + 1) as u16;
    }
}

fn editor_open(terminal_state: &mut EditorState, filename: &str) {
    if let Ok(mut f) = File::open(filename) {
        terminal_state.filename = Some(filename.to_string());
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
    term.dimensions.rows -= 1;
    crossterm::terminal::enable_raw_mode()?;
    refresh_screen(&mut term);
    if args.len() == 2 {
        editor_open(&mut term, &args[1]);
    }

    loop {
        refresh_screen(&mut term);
        display_editor_mode(&mut term)?;
        crossterm::execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(term.rx as u16 - term.coloff, term.cy as u16 - term.rowoff),
        )
        .unwrap();
        if process_char(&mut term)? {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    clear()?;
    Ok(())
}
