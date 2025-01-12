use crate::file;
use crate::piece_table::PieceTable;
use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, queue, terminal};
use std::io;
use std::io::{stdout, Write};

pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
        Output::clear_screen().expect("Error");
    }
}

pub fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn find_index(lines: &Vec<String>, x: usize, y: usize) -> Option<usize> {
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

enum Mode {
    Normal,
    Insert,
    // TODO - Command Mode
    // TODO - Visual Mode
    // TODO - Replace Mode
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
    max_cursor_x: usize,
    screen_columns: usize,
    screen_rows: usize,
}

impl CursorController {
    fn new(win_size: (usize, usize)) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            max_cursor_x: 0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
        }
    }

    fn insert_keypress(
        &mut self,
        key_event: KeyEvent,
        lines: Vec<String>,
        mode: &mut Mode,
        piece_table: &mut PieceTable,
    ) -> io::Result<bool> {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                *mode = Mode::Normal;
                if self.cursor_x != 0 {
                    self.cursor_x -= 1;
                }
            }

            KeyEvent {
                code: KeyCode::Char(ch),
                ..
            } => {
                let x = self.cursor_x;
                let y = self.cursor_y;
                let position = find_index(&lines, x, y).unwrap();
                piece_table.insert(position, &ch.to_string());
                self.cursor_x += 1;
            }

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                let x = self.cursor_x;
                let y = self.cursor_y;
                let position = find_index(&lines, x, y).unwrap();
                piece_table.insert(position, "\n");
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                let x = self.cursor_x.saturating_sub(1);
                let y = self.cursor_y;

                if self.cursor_x == 0 {
                    if self.cursor_y == 0 {
                        return Ok(true);
                    }
                    if let Some(position) = find_index(&lines, 0, self.cursor_y) {
                        piece_table.delete(position - 1);
                        self.cursor_x = lines[self.cursor_y - 1].len();
                        self.cursor_y -= 1;
                    }
                } else if let Some(position) = find_index(&lines, x, y) {
                    piece_table.delete(position);
                    self.cursor_x = x;
                }
            }
            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {
                if self.cursor_x != lines[self.cursor_y].len() {
                    if let Some(position) = find_index(&lines, self.cursor_x, self.cursor_y) {
                        piece_table.delete(position);
                    }
                }
            }
            _ => {}
        }
        self.max_cursor_x = self.cursor_x;
        Ok(true)
    }

    fn normal_keypress(
        &mut self,
        key_event: KeyEvent,
        lines: Vec<String>,
        mode: &mut Mode,
        file_path: String,
        piece_table: &mut PieceTable,
    ) -> io::Result<bool> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(false),

            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => {
                let _ = file::save_file(&file_path, piece_table.to_string());
            }

            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.max_cursor_x = self
                    .max_cursor_x
                    .min(lines[self.cursor_y].len().saturating_sub(1));
                self.max_cursor_x = self.max_cursor_x.saturating_sub(1);
            }

            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.cursor_y != self.screen_rows - 1 && self.cursor_y < lines.len() - 1 {
                    self.cursor_y += 1
                }
            }
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => self.cursor_y = self.cursor_y.saturating_sub(1),
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.max_cursor_x += 1;
                self.max_cursor_x = self
                    .max_cursor_x
                    .min(lines[self.cursor_y].len().saturating_sub(1))
            }
            KeyEvent {
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
                ..
            } => *mode = Mode::Insert,
            _ => {
                return Ok(true);
            }
        }
        self.cursor_x = self
            .max_cursor_x
            .min(lines[self.cursor_y].len().saturating_sub(1));
        Ok(true)
    }
}

#[derive(Debug)]
struct EditorContents {
    content: String,
}

impl EditorContents {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        let mut result = String::new();
        for ch in string.chars() {
            result.push(ch);
            if ch == '\n' {
                result.push('\r');
            }
        }
        self.content.push_str(&result)
    }
}

impl Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct Output {
    editor_contents: EditorContents,
    cursor_controller: CursorController,
}

impl Output {
    fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap_or((80, 24));
        Self {
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(win_size),
        }
    }

    fn clear_screen() -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self, piece_table: &PieceTable) {
        let text = piece_table.to_string();
        self.editor_contents.push_str(&text);
    }

    fn refresh_screen(&mut self, piece_table: &PieceTable) -> io::Result<()> {
        queue!(
            self.editor_contents,
            cursor::Hide,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let cursor_x = self.cursor_controller.cursor_x;
        let cursor_y = self.cursor_controller.cursor_y;
        self.draw_rows(piece_table);
        queue!(
            self.editor_contents,
            cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    fn normal_keypress(&mut self, key_event: KeyEvent, lines: Vec<String>) -> io::Result<bool> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(false),

            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.cursor_controller.max_cursor_x = self
                    .cursor_controller
                    .max_cursor_x
                    .min(lines[self.cursor_controller.cursor_y].len() - 1);
                self.cursor_controller.max_cursor_x =
                    self.cursor_controller.max_cursor_x.saturating_sub(1);
            }

            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if self.cursor_controller.cursor_y != self.cursor_controller.screen_rows - 1
                    && self.cursor_controller.cursor_y < lines.len() - 1
                {
                    self.cursor_controller.cursor_y += 1
                }
            }
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.cursor_controller.cursor_y = self.cursor_controller.cursor_y.saturating_sub(1)
            }
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.cursor_controller.max_cursor_x += 1;
                self.cursor_controller.max_cursor_x = self
                    .cursor_controller
                    .max_cursor_x
                    .min(lines[self.cursor_controller.cursor_y].len() - 1)
            }
            _ => {
                return Ok(true);
            }
        }
        self.cursor_controller.cursor_x = self
            .cursor_controller
            .max_cursor_x
            .min(lines[self.cursor_controller.cursor_y].len() - 1);
        Ok(true)
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> io::Result<KeyEvent> {
        loop {
            if let Event::Key(event) = event::read()? {
                return Ok(event);
            }
        }
    }
}

pub struct Editor {
    reader: Reader,
    output: Output,
    piece_table: PieceTable,
    mode: Mode,
    file_path: String,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            piece_table: PieceTable::default(),
            mode: Mode::Normal,
            file_path: String::new(),
        }
    }
}

impl Editor {
    pub fn new(original_text: &str, file_path: String) -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            piece_table: PieceTable::new(original_text),
            mode: Mode::Normal,
            file_path,
        }
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        let key_event = self.reader.read_key()?;
        let lines = self.piece_table.lines();

        match self.mode {
            Mode::Normal => self.output.cursor_controller.normal_keypress(
                key_event,
                lines,
                &mut self.mode,
                self.file_path.clone(),
                &mut self.piece_table,
            ),
            Mode::Insert => self.output.cursor_controller.insert_keypress(
                key_event,
                lines,
                &mut self.mode,
                &mut self.piece_table,
            ),
        }
    }

    fn test_process_keypress(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        let lines = self.piece_table.lines();

        match self.mode {
            Mode::Normal => self.output.cursor_controller.normal_keypress(
                key_event,
                lines,
                &mut self.mode,
                self.file_path.clone(),
                &mut self.piece_table,
            ),
            Mode::Insert => self.output.cursor_controller.insert_keypress(
                key_event,
                lines,
                &mut self.mode,
                &mut self.piece_table,
            ),
        }
    }

    pub fn run(&mut self) -> io::Result<bool> {
        self.output.refresh_screen(&self.piece_table)?;
        self.process_keypress()
    }

    pub fn test_run(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        self.output.refresh_screen(&self.piece_table)?;
        self.test_process_keypress(key_event)
    }
}
