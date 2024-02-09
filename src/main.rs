use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod profile;
mod render;

fn main() -> Result<(), std::io::Error> {
    // enable raw terminal
    enable_raw_mode().expect("failed to enable raw mode");

    // render menu, which can create and administer tests
    // we also attempt to load the default profile. if it does not exist, we will make a new one
    // for the user that will be saved when they close the program
    if let Ok(profile) = profile::Profile::read_from("profile") {
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
