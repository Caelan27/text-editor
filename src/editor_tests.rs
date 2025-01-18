#[cfg(test)]
mod tests {
    use crate::editor::Editor;
    use crate::file;
    use crate::utils::{control_key_event, create_key_event, string_to_key_events};
    use crossterm::event::{KeyCode, KeyEvent};
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_editor_flow() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file, "Hello, world!\nThis is a test file. DELETETHIS")
            .expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let original_text = file::load_file(file_path)?;
        let mut editor = Editor::new(&original_text, file_path.to_string());

        fn add_repeated_keys(key_events: &mut Vec<KeyEvent>, key: KeyCode, count: usize) {
            for _ in 0..count {
                key_events.push(create_key_event(key));
            }
        }

        let mut key_events = Vec::new();
        key_events.push(create_key_event(KeyCode::Char('j')));
        add_repeated_keys(&mut key_events, KeyCode::Char('l'), 9);

        key_events.push(create_key_event(KeyCode::Char('i')));

        key_events.extend(string_to_key_events(String::from("n edited")));

        key_events.push(create_key_event(KeyCode::Esc));
        add_repeated_keys(&mut key_events, KeyCode::Char('l'), 12);
        key_events.push(create_key_event(KeyCode::Char('i')));
        add_repeated_keys(&mut key_events, KeyCode::Delete, 11);
        key_events.push(create_key_event(KeyCode::Esc));

        key_events.push(control_key_event(KeyCode::Char('w')));
        key_events.push(control_key_event(KeyCode::Char('q')));

        for key_event in key_events {
            println!();
            editor.test_run(key_event)?;
        }

        let saved_content = fs::read_to_string(file_path).expect("Failed to read saved file");
        println!("{}", saved_content);

        assert_eq!(saved_content, "Hello, world!\nThis is an edited test file.");
        Ok(())
    }
}
