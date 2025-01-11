use crossterm::terminal;
use std::io;
use text_editor::screen::{CleanUp, Editor};

fn main() -> io::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new("YAY!\nMore tests!\nEven More!");
    while editor.run()? {}

    Ok(())
}
