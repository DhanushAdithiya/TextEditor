use crate::{TABSTOP, VERSION};
use crossterm::cursor::MoveTo;
use crossterm::event::{poll, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Stylize};
use crossterm::terminal::ClearType;
use crossterm::QueueableCommand;
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;
use std::time::Duration;
use std::{fmt, io};

use crate::keyboard::*;
use crate::terminal::*;

#[derive(Debug, PartialEq)]
pub enum EditorMode {
    NORMAL,
    INSERT,
}

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Erow {
    pub size: usize,
    pub chars: String,
    pub rsize: usize,
    pub render: String,
}

impl Erow {
    pub fn new() -> Self {
        Self {
            size: 0,
            chars: String::from(""),
            rsize: 0,
            render: String::from(""),
        }
    }

    pub fn from(chars: &str) -> Self {
        Self {
            size: chars.len(),
            chars: String::from(chars),
            rsize: 0,
            render: String::from(""),
        }
    }

    pub fn delete_char(&mut self, at: usize) {
        self.chars.remove(at - 1);
        self.size -= 1;
        self.editor_update_row();
    }

    pub fn editor_update_row(&mut self) {
        let mut render = String::new();
        let mut idx = 0;
        for c in self.chars.chars() {
            match c {
                '\t' => {
                    render.push(' ');
                    idx += 1;
                    while idx % TABSTOP != 0 {
                        render.push(' ');
                        idx += 1;
                    }
                }

                _ => {
                    render.push(c);
                    idx += 1;
                }
            }
        }
        self.rsize = idx;
        self.render = render;
    }

    fn editor_row_cx_to_rx(&mut self, cx: usize) -> usize {
        let mut rx = 0;
        for i in 0..cx {
            if let Some(c) = self.chars.chars().nth(i) {
                if c == '\t' {
                    rx += (TABSTOP - 1) - (rx % TABSTOP);
                }
                rx += 1;
            }
        }

        rx
    }

    pub fn editor_row_insert_char(&mut self, at: usize, key: char) {
        if at >= self.size {
            self.chars.push(key)
        } else {
            self.chars.insert(at, key);
        }
        self.size += 1;
        self.editor_update_row();
    }
}

#[derive(Debug)]
pub struct EditorState {
    pub dimensions: WindowSize,
    pub cx: usize,
    pub cy: usize,
    pub rx: usize,
    pub mode: EditorMode,
    pub row: Vec<Erow>,
    pub numrows: u16,
    pub rowoff: u16,
    pub coloff: u16,
    pub filename: Option<String>,
    path: Option<String>,
    message: Option<String>,
    pub dirty: bool,
}

impl EditorState {
    pub fn new() -> Self {
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
            message: None,
            filename: None,
            path: None,
            dirty: false,
        }
    }

    fn editor_satus_message(&mut self) -> String {
        if let Some(msg) = &self.message {
            let padding_len = self.dimensions.columns as usize - msg.len();
            let padding = format!("{:width$}", " ", width = padding_len);
            let status = format!("{msg}{}", padding);
            status
        } else {
            let status;
            if let Some(filename) = &self.filename {
                let status_content = format!("{} | {}", self.mode, filename);
                let padding = format!(
                    "~{:width$}",
                    " ",
                    width = self.dimensions.columns as usize - status_content.len() - 1
                );
                status = format!("{status_content}{}", padding);
                status
            } else {
                let status_content = format!("{}", self.mode);
                let padding = format!(
                    "~{:width$}",
                    " ",
                    width = self.dimensions.columns as usize - status_content.len() - 1
                );
                status = format!("{status_content}{}", padding);
                status
            }
        }
    }

    pub fn editor_status_line(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        let status = self
            .editor_satus_message()
            .with(Color::Black)
            .on(Color::White);

        stdout
            .queue(crossterm::cursor::MoveTo(0, self.dimensions.rows + 1))?
            .queue(crossterm::style::Print(status))?;

        Ok(())
    }

    pub fn erow_to_string(&self) -> String {
        let mut buffer = String::new();
        for i in 0..self.numrows {
            buffer.push_str(&self.row[i as usize].chars);
            buffer.push('\n');
        }

        buffer
    }

    pub fn editor_save(&mut self) -> io::Result<()> {
        if let Some(filepath) = &self.path {
            let buffer = self.erow_to_string();
            std::fs::write(filepath, buffer)?;
            self.dirty = false;
            let msg = format!("{} has been saved!", self.filename.clone().unwrap());
            self.message = Some(msg);
        } else {
        }

        Ok(())
    }

    pub fn move_cursor(&mut self) {
        crossterm::execute!(io::stdout(), MoveTo(self.cx as u16, self.cy as u16)).unwrap();
    }

    pub fn process_char(&mut self) -> io::Result<bool> {
        static mut QUIT_TIMES: u8 = 1;
        match poll(Duration::from_millis(100)) {
            Ok(true) => match read_character() {
                Some(key) => match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        if self.dirty && unsafe { QUIT_TIMES } > 0 {
                            // TODO STATUS MESSAGE
                            unsafe { QUIT_TIMES -= 1 }
                        } else {
                            let mut stdout = io::stdout();
                            stdout.queue(crossterm::cursor::SetCursorStyle::SteadyBlock)?;
                            return Ok(true);
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => self.mode = EditorMode::NORMAL,

                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => self.editor_save()?,

                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        self.dirty = true;

                        if self.mode == EditorMode::INSERT {
                            if self.cx > 0 {
                                self.row[self.cy].delete_char(self.cx);
                                self.cx -= 1;
                            } else if self.cx == 0 && self.cy > 0 {
                                let buffer = &self.row[self.cy].chars.clone();
                                self.row[self.cy - 1].chars.push_str(buffer);

                                //self.row[self.cy - 1].chars.push('\n');
                                self.row.remove(self.cy);
                                self.numrows -= 1;
                                self.row[self.cy - 1].editor_update_row();
                                self.cy -= 1;
                                self.cx = self.row[self.cy - 1].rsize;
                            }
                        } else if self.mode == EditorMode::NORMAL && self.cx > 0 {
                            self.cx -= 1;
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if self.mode == EditorMode::NORMAL {
                            if self.cy <= self.numrows.into() {
                                self.cy += 1;
                            }
                        } else if self.mode == EditorMode::INSERT {
                            let buffer = Erow::from(&self.row[self.cy].chars[self.cx..]);
                            self.row[self.cy].chars.insert(self.cx, '\n');
                            self.row.insert(self.cy + 1, buffer);
                            self.numrows += 1;
                            self.row[self.cy].editor_update_row();
                            self.row[self.cy + 1].editor_update_row();
                            self.cy += 1;
                            self.cx = 0;
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        self.message = None;
                        if self.mode == EditorMode::NORMAL {
                            normal_mode_shortcuts(self, c);
                        } else if self.mode == EditorMode::INSERT {
                            self.editor_insert_char(c);
                        }
                    }
                    _ => {}
                },
                None => {}
            },
            _ => {}
        }

        Ok(false)
    }

    pub fn editor_draw_rows(&self) -> Result<()> {
        let mut stdout = io::stdout();
        for i in 0..self.dimensions.rows {
            let filerow = i + self.rowoff;
            if filerow >= self.numrows {
                if i == self.dimensions.rows / 3 && self.numrows == 0 {
                    let welcome_str = format!("BREAD EDITOR - VERSION : {VERSION}");
                    let w = (self.dimensions.columns as usize - welcome_str.len()) / 2;
                    let padding = format!("{:width$}", " ", width = w);
                    stdout
                        .queue(crossterm::style::Print(padding))?
                        .queue(crossterm::style::Print(welcome_str))?;
                } else {
                    stdout.queue(crossterm::style::Print("\r\n~"))?;
                }
            } else {
                let mut len = self.row[filerow as usize].rsize;
                if len < self.coloff as usize {
                    continue;
                }
                len -= self.coloff as usize;
                let start = self.coloff as usize;
                let end = start
                    + if len >= self.dimensions.columns as usize {
                        self.dimensions.columns as usize
                    } else {
                        len
                    };
                stdout
                    .queue(crossterm::cursor::MoveTo(0, i))?
                    .queue(crossterm::terminal::Clear(ClearType::CurrentLine))?
                    .queue(crossterm::style::Print(
                        &self.row[filerow as usize].render[start..end],
                    ))?;
            }
        }
        Ok(())
    }

    pub fn editor_insert_char(&mut self, key: char) {
        if self.dirty == false {
            self.dirty = true
        }

        if self.cy == self.numrows.into() {
            self.editor_append_row(String::new(), 0);
        }
        self.row[self.cy].editor_row_insert_char(self.cx, key);
        self.cx += 1;
    }

    pub fn change_cursor(&self) -> Result<()> {
        let mut stdout = io::stdout();
        if self.mode == EditorMode::NORMAL {
            stdout.queue(crossterm::cursor::SetCursorStyle::SteadyBlock)?;
        } else if self.mode == EditorMode::INSERT {
            stdout.queue(crossterm::cursor::SetCursorStyle::BlinkingBar)?;
        }
        Ok(())
    }

    pub fn refresh_screen(&mut self) {
        self.editor_scroll();
        clear().unwrap();
        self.editor_draw_rows().unwrap();
    }

    pub fn editor_append_row(&mut self, chars: String, length: usize) {
        let loc = self.numrows as usize;
        self.row[loc].size = length;
        self.row[loc].chars = chars;
        self.numrows += 1;

        self.row[loc].rsize = 0;
        self.row[loc].render = String::new();
        self.row[loc].editor_update_row();
    }

    pub fn editor_scroll(&mut self) {
        self.rx = 0;

        if self.cy < self.numrows.into() {
            self.rx = self.row[self.cy].editor_row_cx_to_rx(self.cx);
        }

        if self.cy < self.rowoff.into() {
            self.rowoff = self.cy as u16;
        }

        if self.rx < self.coloff.into() {
            self.coloff = self.rx as u16;
        }

        if self.cy >= (self.rowoff + self.dimensions.rows) as usize {
            self.rowoff = (self.cy as u16 - self.dimensions.rows + 1) as u16;
        }

        if self.rx >= (self.coloff + self.dimensions.columns) as usize {
            self.coloff = (self.rx as u16 - self.dimensions.columns + 1) as u16;
        }
    }

    pub fn editor_open(&mut self, filename: &str) {
        if let Ok(mut f) = File::open(filename) {
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();

            self.path = Some(filename.to_string());
            self.filename = Path::new(filename)
                .file_name()
                .map(|os_str| os_str.to_string_lossy().into());

            buffer.lines().for_each(|l| {
                let new_row = Erow::new();
                self.row.push(new_row);
                self.editor_append_row(l.to_string(), l.len());
            });
        } else {
            let folders = Path::new(filename)
                .parent()
                .map(|parent| parent.to_str().unwrap());
            if let Some(folder) = folders {
                std::fs::create_dir_all(folder).unwrap();
            }
            std::fs::write(filename, "").unwrap();
            self.editor_open(filename);
        }
    }
}

pub fn clear() -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .queue(crossterm::terminal::Clear(ClearType::All))?
        .queue(crossterm::cursor::MoveTo(0, 0))?;

    Ok(())
}
