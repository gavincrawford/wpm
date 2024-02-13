use clap::{arg, Command};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod profile;
mod render;

fn main() -> Result<(), std::io::Error> {
    // get args
    let args = Command::new("WPM")
        .arg(arg!(--profile <PATH> "Runs the app with the specified profile"))
        .arg(arg!(--"no-profile" "Runs the app without a profile to save to"))
        .get_matches();

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
