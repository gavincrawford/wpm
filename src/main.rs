use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use rand::seq::SliceRandom;

fn main() -> Result<(), std::io::Error> {
    // get phrase from wordlist
    const ENG_1K: &str = include_str!("../wordlist/eng_1k.txt");
    let tokens: Vec<&str> = str_to_tokens(ENG_1K);
    let phrase = tokens_to_phrase(25, &tokens);

    // basic terminal renderer
    let mut stdout = stdout();
    let mut pos = 0;
    enable_raw_mode().expect("failed to enable raw mode");
    clear(&mut stdout);
    loop {
        // render
        clear(&mut stdout);
        for (i, c) in phrase.chars().enumerate() {
            let style;
            if i < pos {
                style = c.black().on_green().italic()
            } else if c == ' ' {
                style = '_'.dark_grey().on_grey()
            } else {
                style = c.black().on_grey();
            }
            queue!(stdout, Print(style))?;
        }
        queue!(stdout, MoveTo(pos as u16, 0))?;
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

/// Split a string into a vector of its lines.
fn str_to_tokens(src: &str) -> Vec<&str> {
    src.lines().collect::<Vec<&str>>()
}

/// Select `n` number of tokens to create a random phrase.
fn tokens_to_phrase(n: usize, tokens: &Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let mut str = String::new();
    for _ in 0..n {
        str += tokens.choose(&mut rng).unwrap();
        str += " ";
    }
    str.trim().to_string()
}
