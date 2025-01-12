use crossterm::terminal;
use std::env;
use std::io;
use text_editor::editor::{CleanUp, Editor};
use text_editor::file;

fn main() -> io::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_path = args[1].clone();
        let original_text = file::load_file(&file_path)?;
        let mut editor = Editor::new(&original_text, file_path);

        while editor.run()? {}
    } else {
        panic!("Please provide one argument - The file to read");
    }
    dbg!("Editor stopped");

    Ok(())
}
