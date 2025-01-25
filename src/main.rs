use crossterm::terminal;
use log::{error, info};
use std::env;
use std::io;
use text_editor::editor::{CleanUp, Editor};
use text_editor::file;

fn main() -> io::Result<()> {
    env_logger::init();
    info!("Logging initialized");

    let _clean_up = CleanUp;
    if let Err(e) = terminal::enable_raw_mode() {
        error!("Error enabling raw mode: {}", e);
    };

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let file_path = args[1].clone();
        let original_text = file::load_file(&file_path)?;
        let mut editor = Editor::new(&original_text, file_path);

        while editor.run()? {}
    } else {
        error!("Please provide one argument - The file to read");
        panic!("Please provide one argument - The file to read");
    }

    Ok(())
}
