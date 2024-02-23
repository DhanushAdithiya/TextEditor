use crossterm::terminal::size;

#[derive(Debug)]
pub struct WindowSize {
    pub rows: u16,
    pub columns: u16,
}

pub fn resize_terminal() -> WindowSize {
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
