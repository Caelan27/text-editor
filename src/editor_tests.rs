#[cfg(test)]
mod tests {
    use crate::editor::{create_key_event, CleanUp, Editor};
    use crate::file;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
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
        add_repeated_keys(&mut key_events, KeyCode::Char('l'), 9);
        key_events.push(create_key_event(KeyCode::Char('j')));

        key_events.push(create_key_event(KeyCode::Char('i')));
        key_events.push(create_key_event(KeyCode::Char('n')));
        key_events.push(create_key_event(KeyCode::Char(' ')));
        key_events.push(create_key_event(KeyCode::Char('e')));
        key_events.push(create_key_event(KeyCode::Char('d')));
        key_events.push(create_key_event(KeyCode::Char('i')));
        key_events.push(create_key_event(KeyCode::Char('t')));
        key_events.push(create_key_event(KeyCode::Char('e')));
        key_events.push(create_key_event(KeyCode::Char('d')));
        key_events.push(create_key_event(KeyCode::Esc));
        add_repeated_keys(&mut key_events, KeyCode::Char('l'), 11);
        key_events.push(create_key_event(KeyCode::Char('i')));
        add_repeated_keys(&mut key_events, KeyCode::Delete, 11);
        key_events.push(create_key_event(KeyCode::Esc));
        key_events.push(KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

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
