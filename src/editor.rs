use crate::metadata::FileMetadata;
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

pub enum TextType {
    PieceTable(PieceTable),
    String(String),
}

#[derive(PartialEq)]
pub enum BarMode {
    Write,
}

#[derive(PartialEq)]
pub enum Mode {
    Normal(Option<BarMode>),
    Insert,
    Command { previous_chars: Vec<char> },
    // TODO - Visual Mode
    // TODO - Replace Mode
}

pub struct KeyHandler {
    pub mode: Mode,
}

pub struct CursorController {
    pub cursor_x: usize,
    pub desired_cursor_x: usize,
    pub cursor_y: usize,

    pub relative_x: usize,
    pub relative_y: usize,

    pub screen_columns: usize,
    pub screen_rows: usize,
}

impl CursorController {
    fn new(win_size: (usize, usize)) -> Self {
        Self {
            cursor_x: 0,
            desired_cursor_x: 0,
            cursor_y: 0,
            relative_x: 0,
            relative_y: 0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
        }
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
    scroll_y: usize,
}

impl Output {
    fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap_or((80, 24));
        Self {
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(win_size),
            scroll_y: 0,
        }
    }

    fn clear_screen() -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self, content: String) {
        self.editor_contents.push_str(&content);
    }

    fn refresh_screen(
        &mut self,
        piece_table: &PieceTable,
        mode: &Mode,
        metadata: &FileMetadata,
    ) -> io::Result<()> {
        queue!(
            self.editor_contents,
            cursor::Hide,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        let content_rows = self.cursor_controller.screen_rows - 2;
        let cursor_y = self.cursor_controller.cursor_y;

        let cur_top_of_screen = self.scroll_y;
        let cur_bottom_of_screen = cur_top_of_screen + content_rows - 1;

        if cursor_y > cur_bottom_of_screen {
            self.scroll_y += cursor_y - cur_bottom_of_screen;
        } else if cursor_y < cur_top_of_screen {
            self.scroll_y = cursor_y;
        }

        self.cursor_controller.relative_y = cursor_y.saturating_sub(self.scroll_y);

        let cur_top_of_screen = self.scroll_y;
        let cur_bottom_of_screen = cur_top_of_screen + content_rows - 1;

        let lines = &piece_table.lines();
        let num_lines = lines.len();
        let mut displayed_lines = if num_lines < cur_bottom_of_screen + 1 {
            &lines[self.scroll_y..num_lines]
        } else {
            &lines[self.scroll_y..cur_bottom_of_screen + 1]
        }
        .to_vec();
        if num_lines < cur_bottom_of_screen + 1 {
            for _ in 0..(cur_bottom_of_screen + 1 - num_lines) {
                displayed_lines.push("".to_string());
            }
        }

        self.draw_rows(displayed_lines.join("\n"));

        let file_path = metadata.file_path.to_string();
        let line_percent = if lines.is_empty() || lines.len() == 1 {
            "Top"
        } else {
            match 100 * self.cursor_controller.cursor_y / (lines.len() - 1) {
                100 => "Bot",
                0 => "Top",
                percent => &format!("{}%", percent),
            }
        };
        let left_part = format!(
            "{},{}        {}",
            self.cursor_controller.cursor_y + 1,
            self.cursor_controller.cursor_x + 1,
            line_percent
        );

        let remaining_space =
            self.cursor_controller.screen_columns - file_path.len() - left_part.len();

        let spaces = " ".repeat(remaining_space);

        self.draw_rows(format!("\n{file_path}{spaces}{left_part}"));

        let (cursor_x, cursor_y) = match mode {
            Mode::Insert => {
                self.draw_rows("\n-- INSERT --".to_string());
                (
                    self.cursor_controller.cursor_x,
                    self.cursor_controller
                        .cursor_y
                        .saturating_sub(self.scroll_y),
                )
            }
            Mode::Command { previous_chars } => {
                let mut chars = ":".to_string();
                for ch in previous_chars {
                    chars.push(*ch);
                }
                self.draw_rows(format!("\n{chars}"));
                let cursor_y = self.cursor_controller.screen_rows - 1;
                let cursor_x = previous_chars.len() + 1;
                (cursor_x, cursor_y)
            }
            Mode::Normal(Some(BarMode::Write)) => {
                let file_path = metadata.file_path.clone();
                let num_lines = lines.len();
                let file_size = metadata.file_size.unwrap();

                let file_path = format!(r##""{file_path}""##);
                let num_lines = format!("{num_lines}L");
                let file_size = format!("{file_size}B written");

                self.draw_rows(format!("\n{file_path} {num_lines}, {file_size}"));
                (
                    self.cursor_controller.cursor_x,
                    self.cursor_controller
                        .cursor_y
                        .saturating_sub(self.scroll_y),
                )
            }
            _ => (
                self.cursor_controller.cursor_x,
                self.cursor_controller
                    .cursor_y
                    .saturating_sub(self.scroll_y),
            ),
        };

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
    key_handler: KeyHandler,
    metadata: FileMetadata,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            piece_table: PieceTable::default(),
            key_handler: KeyHandler::new(),
            metadata: FileMetadata::new(String::new()),
        }
    }
}

impl Editor {
    pub fn new(original_text: &str, file_path: String) -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            piece_table: PieceTable::new(original_text),
            key_handler: KeyHandler::new(),
            metadata: FileMetadata::new(file_path),
        }
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        let key_event = self.reader.read_key()?;
        let lines = self.piece_table.lines();

        match self.key_handler.mode {
            Mode::Normal(_) => self.key_handler.normal_keypress(
                key_event,
                lines,
                self.metadata.file_path.clone(),
                &mut self.piece_table,
                &mut self.output.cursor_controller,
            ),
            Mode::Insert => self.key_handler.insert_keypress(
                key_event,
                lines,
                &mut self.piece_table,
                &mut self.output.cursor_controller,
            ),
            Mode::Command { .. } => self.key_handler.command_keypress(
                key_event,
                &mut self.piece_table,
                &mut self.metadata,
            ),
        }
    }

    fn test_process_keypress(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        let lines = self.piece_table.lines();

        match self.key_handler.mode {
            Mode::Normal(_) => self.key_handler.normal_keypress(
                key_event,
                lines,
                self.metadata.file_path.clone(),
                &mut self.piece_table,
                &mut self.output.cursor_controller,
            ),
            Mode::Insert => self.key_handler.insert_keypress(
                key_event,
                lines,
                &mut self.piece_table,
                &mut self.output.cursor_controller,
            ),
            Mode::Command { .. } => self.key_handler.command_keypress(
                key_event,
                &mut self.piece_table,
                &mut self.metadata,
            ),
        }
    }

    pub fn run(&mut self) -> io::Result<bool> {
        self.output
            .refresh_screen(&self.piece_table, &self.key_handler.mode, &self.metadata)?;
        self.piece_table.merge();
        self.process_keypress()
    }

    pub fn test_run(&mut self, key_event: KeyEvent) -> io::Result<bool> {
        self.output
            .refresh_screen(&self.piece_table, &self.key_handler.mode, &self.metadata)?;
        self.test_process_keypress(key_event)
    }
}
