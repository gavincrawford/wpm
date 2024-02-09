use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod profile;
mod render;

fn main() -> Result<(), std::io::Error> {
    // enable raw terminal
    enable_raw_mode().expect("failed to enable raw mode");

    // render menu, which can create and administer tests
    render::menu::MenuRenderer::new().render()?;

    // disable raw terminal
    disable_raw_mode().expect("failed to disable raw mode");

    // done
    Ok(())
}
