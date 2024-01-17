use crossterm::event::{poll, read, Event::Key, KeyCode};
use errno::errno;
use std::io;
use std::time::Duration;

fn die<M: Into<String>>(msg: M) {
    let _ = crossterm::terminal::disable_raw_mode();
    eprintln!("{}:{}", msg.into(), errno());
    std::process::exit(1);
}

fn main() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    loop {
        let mut c = None;

        match poll(Duration::from_millis(100)) {
            Ok(true) => {
                if let Ok(e) = read() {
                    if let Key(key_event) = e {
                        c = Some(key_event)
                    }
                } else {
                    die("read failed");
                }
            }
            _ => {
                let msg = errno();
                if msg.to_string() == "Success" {
                    continue;
                } else {
                    die("pool failed");
                }
            }
        }

        if let Some(c) = c {
            if c.code == KeyCode::Char('q') {
                break;
            } else {
                println!("{c:?}\r");
            }
        } else {
            println!("0\r");
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
