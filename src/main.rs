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
        // run with no profile
        render::menu::MenuRenderer::new(None).render()?;
    } else if let Some(profile) = args.get_one::<String>("profile") {
        // run with user-provided profile
        if let Ok(profile) = profile::Profile::read_from(profile) {
            render::menu::MenuRenderer::new(Some(profile)).render()?;
        } else {
            eprintln!("This profile does not exist.");
        }
    } else if let Ok(profile) = profile::Profile::read_from("profile") {
        // run with default profile
        render::menu::MenuRenderer::new(Some(profile)).render()?;
    } else {
        // if no other option, run with new, default profile
        let profile = profile::Profile::default();
        render::menu::MenuRenderer::new(Some(profile)).render()?;
    }

    // disable raw terminal
    disable_raw_mode().expect("failed to disable raw mode");

    // done
    Ok(())
}
