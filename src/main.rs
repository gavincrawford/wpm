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
use tokens::{str_to_tokens, tokens_to_phrase, ENG_1K};

mod tokens;

fn main() -> Result<(), std::io::Error> {
    // get phrase from wordlist
    let tokens: Vec<&str> = str_to_tokens(ENG_1K);
    let phrase = tokens_to_phrase(25, &tokens);

    // basic terminal renderer
    let mut stdout = stdout();
    let mut pos = 0;
    let mut miss = false;
    enable_raw_mode().expect("failed to enable raw mode");
    clear(&mut stdout);
    loop {
        // render
        clear(&mut stdout);
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
