use clap::{arg, Command};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod profile;
mod render;

fn main() -> Result<(), std::io::Error> {
    // get args
    let args = Command::new("WPM")
        .arg(arg!(--"no-profile" "Runs the app without a profile to save to"))
        .get_matches();

    // enable raw terminal
    enable_raw_mode().expect("failed to enable raw mode");

    // render menu, which can create and administer tests
    // load the default profile, and if it does not exist, make a new one
    if *args.get_one::<bool>("no-profile").unwrap_or(&false) {
        render::menu::MenuRenderer::new(None).render()?;
    } else if let Ok(profile) = profile::Profile::read_from("profile") {
        render::menu::MenuRenderer::new(Some(profile)).render()?;
    } else {
        let profile = profile::Profile::default();
        render::menu::MenuRenderer::new(Some(profile)).render()?;
    }

    // disable raw terminal
    disable_raw_mode().expect("failed to disable raw mode");

    // done
    Ok(())
}
