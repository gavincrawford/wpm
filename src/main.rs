use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use clap::{arg, Command};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use tokens::{str_to_tokens, tokens_to_phrase, ENG_10K, ENG_1K};

mod tokens;

fn main() -> Result<(), std::io::Error> {
    // get args
    let args = Command::new("WPM")
        .arg(arg!(<difficulty> "test difficulty (easy/hard)"))
        .get_matches();
    let wordlist = match args
        .get_one::<String>("difficulty")
        .unwrap_or(&String::from("easy"))
        .as_str()
    {
        // wordlists
        "easy" => ENG_1K,
        "hard" => ENG_10K,
        // something went wrong
        _ => std::process::exit(1),
    };

    // get phrase from wordlist
    let tokens: Vec<&str> = str_to_tokens(wordlist);
    let phrase = tokens_to_phrase(25, &tokens);

    // basic terminal renderer
    let mut stdout = stdout();
    let mut pos: usize = 0;
    let mut miss: bool = false;
    enable_raw_mode().expect("failed to enable raw mode");
    clear(&mut stdout);
    loop {
        // render
        let size = size().expect("Failed to read screen size.");
        queue!(stdout, MoveTo(0, 0), Hide)?;
        for (i, c) in phrase.chars().enumerate() {
            // style regular characters
            let mut style;
            if i < pos {
                style = c.black().on_green().italic();
            } else if c == ' ' {
                style = '_'.dark_grey().on_grey();
            } else {
                style = c.black().on_grey();
            }

            // style on miss
            if i == pos && miss == true {
                style = style.on_red();
                miss = false;
            }

            // print out styled content
            queue!(stdout, Print(style))?;
        }
        queue!(
            stdout,
            Show,
            MoveTo(
                (pos % size.0 as usize) as u16,
                (pos / size.0 as usize) as u16
            )
        )?;
        stdout.flush()?;

        // end condition
        if pos == phrase.len() {
            break;
        }

        // handle events
        use Event::*;
        use KeyCode::*;
        if !poll(Duration::from_millis(200))? {
            continue;
        }
        match read()? {
            Key(key) => match key.code {
                Esc => break,
                Backspace => pos -= 1,
                Char(char) => {
                    let next = phrase.chars().nth(pos);
                    if char == next.unwrap_or('~') {
                        pos += 1;
                    } else {
                        miss = true;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    disable_raw_mode().expect("failed to disable raw mode");
    clear(&mut stdout);

    // done
    Ok(())
}

/// Clear the screen via the given `stdout` handle.
fn clear(io: &mut Stdout) {
    execute!(
        io,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Clear(ClearType::Purge)
    )
    .expect("failed to clear screen")
}
