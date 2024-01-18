use std::{
    io::{stdout, Stdout, Write},
    time::{Duration, Instant},
};

use clap::{arg, Command};
use crossterm::{
    cursor::MoveTo,
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
    let phrase = tokens_to_phrase(10, &tokens);

    // set up variables for the renderer
    let mut stdout = stdout(); // stdout handle
    let mut n_miss: usize = 0; // miss counter (error count)
    let mut miss: bool = false; // miss render flag
    let mut n_hit: usize = 0; // hit counter (cursor pos)
    let mut hit: bool = false; // hit render flag
    let timer = Instant::now(); // timer for WPM calculation
    enable_raw_mode().expect("failed to enable raw mode");
    clear(&mut stdout);

    // render base text
    queue!(stdout, MoveTo(0, 0))?;
    for c in phrase.chars() {
        if c == ' ' {
            queue!(stdout, Print('_'.dark_grey().on_grey()))?;
        } else {
            queue!(stdout, Print(c.black().on_grey()))?;
        }
    }

    // play loop
    loop {
        // render
        let size = size().expect("Failed to read screen size.");
        if hit {
            queue!(
                stdout,
                move_to_wrap(n_hit - 1, size),
                Print((phrase.as_bytes()[n_hit - 1] as char).black().on_green())
            )?;
            hit = false;
        } else if miss {
            queue!(
                stdout,
                move_to_wrap(n_hit, size),
                Print((phrase.as_bytes()[n_hit] as char).on_red())
            )?;
            miss = false;
        }
        queue!(stdout, move_to_wrap(n_hit, size))?;
        stdout.flush()?;

        // end condition
        if n_hit == phrase.len() {
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
                Backspace => n_hit -= 1,
                Char(char) => {
                    let next = phrase.chars().nth(n_hit);
                    if char == next.unwrap_or('~') {
                        n_hit += 1;
                        hit = true;
                    } else {
                        n_miss += 1;
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

    // give user wpm
    println!(
        "GROSS: {:.2}wpm\nNET:   {:.2}wpm",
        wpm_gross(phrase.len(), timer.elapsed()),
        wpm_net(phrase.len(), n_miss, timer.elapsed())
    );

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

/// Move to position by char, with wrap in respect to `size`.
fn move_to_wrap(pos: usize, size: (u16, u16)) -> MoveTo {
    MoveTo(
        (pos % size.0 as usize) as u16,
        (pos / size.0 as usize) as u16,
    )
}

/// Calculate raw WPM from typed characters and time.
fn wpm_gross(k: usize, dur: Duration) -> f32 {
    (k as f32 / 5.) / (dur.as_secs() as f32 / 60.)
}

/// Calculate net WPM from typed characters and time, with consideration for errors.
fn wpm_net(k: usize, e: usize, dur: Duration) -> f32 {
    wpm_gross(k, dur) - (e as f32 / (dur.as_secs() as f32 / 60.))
}
