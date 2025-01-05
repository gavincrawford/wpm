use std::{io::stdout, panic::PanicHookInfo};

use clap::{arg, Command};
use crossterm::{
    cursor::Show,
    queue,
    terminal::{disable_raw_mode, enable_raw_mode},
};

mod config;
mod profile;
mod render;

fn main() -> Result<(), std::io::Error> {
    // get args
    let args = Command::new("WPM")
        .arg(arg!(--profile <PATH> "Runs the app with the specified profile"))
        .arg(arg!(--"no-profile" "Runs the app without a profile to save to"))
        .get_matches();

    // set panic hook in case anything goes wrong
    std::panic::set_hook(Box::new(panic_handler));

    // enable raw terminal
    enable_raw_mode().expect("failed to enable raw mode");

    // render menu, which can create and administer tests
    if args.get_flag("no-profile") {
        render::menu::MenuRenderer::new(None).render()?;
    } else if let Some(profile) = args.get_one::<String>("profile") {
        render::menu::MenuRenderer::new(Some(profile.clone())).render()?;
    } else {
        render::menu::MenuRenderer::new(Some(String::from("profile"))).render()?;
    }

    // disable raw terminal
    disable_raw_mode().expect("failed to disable raw mode");

    // done
    Ok(())
}

/// Handles any panics that may occur. This handler ensures that, if not already, the terminal is
/// not in raw mode and the cursor is shown, so that the terminal environment does not get
/// disrupted by a panic within the program.
fn panic_handler(info: &PanicHookInfo) {
    disable_raw_mode().unwrap();
    queue!(stdout(), Show).unwrap();
    println!("{}", info);
}
