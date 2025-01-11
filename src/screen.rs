use crate::piece_table::{self, PieceTable};
use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, queue, terminal};
use std::io;
use std::io::{stdout, Write};

const VERSION: &str = "0.0";

pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode");
        Output::clear_screen().expect("Error");
    }
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

    fn move_cursor(&mut self, keycode: KeyCode, lines: Vec<String>) {
        match keycode {
            KeyCode::Char('h') => {
                self.max_cursor_x = self.max_cursor_x.min(lines[self.cursor_y].len() - 1);
                self.max_cursor_x = self.max_cursor_x.saturating_sub(1);
            }
            KeyCode::Char('j') => {
                if self.cursor_y != self.screen_rows - 1 && self.cursor_y < lines.len() - 1 {
                    self.cursor_y += 1
                }
            }
            KeyCode::Char('k') => self.cursor_y = self.cursor_y.saturating_sub(1),
            KeyCode::Char('l') => {
                self.max_cursor_x += 1;
                self.max_cursor_x = self.max_cursor_x.min(lines[self.cursor_y].len() - 1)
            }
            _ => unimplemented!(),
        }
        self.cursor_x = self.max_cursor_x.min(lines[self.cursor_y].len() - 1);
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
    win_size: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController,
}

impl Output {
    fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self {
            win_size,
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

    fn move_cursor(&mut self, keycode: KeyCode, lines: Vec<String>) {
        self.cursor_controller.move_cursor(keycode, lines);
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
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            piece_table: PieceTable::default(),
        }
    }
}

impl Editor {
    pub fn new(original_text: &str) -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            piece_table: PieceTable::new(original_text),
        }
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(false),

            KeyEvent {
                code: keycode @ KeyCode::Char('h' | 'j' | 'k' | 'l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => self.output.move_cursor(keycode, self.piece_table.lines()),
            _ => {}
        }
        Ok(true)
    }

    pub fn run(&mut self) -> io::Result<bool> {
        self.output.refresh_screen(&self.piece_table)?;
        self.process_keypress()
    }
}
