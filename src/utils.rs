use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

pub fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

pub fn control_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

pub fn string_to_key_events(text: String) -> Vec<KeyEvent> {
    let mut key_events = Vec::new();
    for ch in text.chars() {
        key_events.push(create_key_event(KeyCode::Char(ch)))
    }
    key_events
}

pub fn find_index(lines: &[String], x: usize, y: usize) -> Option<usize> {
    let mut char_count = 0;
    for (cur_y, line) in lines.iter().enumerate() {
        if line.is_empty() && cur_y == y {
            return Some(char_count);
        }

        for (cur_x, _) in line.chars().enumerate() {
            if cur_x == x && cur_y == y {
                return Some(char_count);
            }
            char_count += 1;
            if cur_x + 1 == x && cur_y == y {
                return Some(char_count);
            }
        }
        char_count += 1;
    }
    None
}
