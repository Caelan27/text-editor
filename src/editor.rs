use crate::metadata::FileMetadata;
use crate::piece_table::PieceTable;
use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, queue, terminal};
use log::{error, info};
use std::io;
use std::io::{stdout, Write};

pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        if let Err(e) = terminal::disable_raw_mode() {
            error!("Could not turn off raw mode: {}", e);
        }
        if let Err(e) = Output::clear_screen() {
            error!("Failed to clear screen: {}", e);
        }
    }
}

pub enum TextType {
    PieceTable(PieceTable),
    String(String),
}

#[derive(PartialEq, Clone)]
pub enum BarMode {
    Write,
}

#[derive(PartialEq, Clone)]
pub enum Mode {
    Normal(Option<BarMode>),
    Insert,
    Command { previous_chars: String },
    // TODO - Visual Mode
    // TODO - Replace Mode
}

pub struct KeyHandler {
    mode: Mode,
}

impl Default for KeyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyHandler {
    pub fn new() -> Self {
        KeyHandler {
            mode: Mode::Normal(None),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn get_mode_mut(&mut self) -> &mut Mode {
        &mut self.mode
    }
}

pub struct CursorController {
    cursor_x: usize,
    desired_cursor_x: usize,
    cursor_y: usize,

    relative_y: usize,

    screen_columns: usize,
    screen_rows: usize,
}

impl CursorController {
    fn new(window_size: (usize, usize)) -> Self {
        Self {
            cursor_x: 0,
            desired_cursor_x: 0,
            cursor_y: 0,
            relative_y: 0,
            screen_columns: window_size.0,
            screen_rows: window_size.1,
        }
    }

    pub fn cursor_x(&self) -> usize {
        self.cursor_x
    }

    pub fn cursor_y(&self) -> usize {
        self.cursor_y
    }

    pub fn screen_size(&self) -> (usize, usize) {
        (self.screen_columns, self.screen_rows)
    }

    // Doesn't let you go past the end of the line
    pub fn set_cursor_x_normal_mode(&mut self, x: usize, line_length: usize) {
        if x >= line_length {
            self.cursor_x = line_length - 1;
        } else {
            self.cursor_x = x;
        }

        self.desired_cursor_x = self.cursor_x;
    }

    // Lets you go one space past the end of the line
    pub fn set_cursor_x_insert_mode(&mut self, x: usize, line_length: usize) {
        if x > line_length {
            self.cursor_x = line_length;
        } else {
            self.cursor_x = x;
        }

        self.desired_cursor_x = self.cursor_x;
    }

    pub fn set_cursor_x_no_checks(&mut self, x: usize) {
        self.cursor_x = x;
        self.desired_cursor_x = self.cursor_x;
    }

    pub fn set_cursor_y(&mut self, y: usize, num_lines: usize) {
        if y >= num_lines {
            self.cursor_y = num_lines - 1;
        } else {
            self.cursor_y = y;
        }
    }

    pub fn update_desired_x(&mut self) {
        self.desired_cursor_x = self.cursor_x;
    }

    pub fn update_desired_x_if_needed(&mut self, lines: &[String]) {
        self.cursor_x = self
            .desired_cursor_x
            .min(lines[self.cursor_y].len().saturating_sub(1));
    }
}

pub struct EditorView {
    cursor_controller: CursorController,
    scroll_y: usize,
}

impl EditorView {
    fn new(window_size: (usize, usize)) -> Self {
        Self {
            cursor_controller: CursorController::new(window_size),
            scroll_y: 0,
        }
    }

    fn update_scroll(&mut self) {
        self.scroll_y = self.adjust_scroll();
        self.cursor_controller.relative_y = self
            .cursor_controller
            .cursor_y
            .saturating_sub(self.scroll_y);
    }

    fn adjust_scroll(&self) -> usize {
        let content_rows = self.cursor_controller.screen_rows - Output::STATUS_BAR_ROWS;
        let scroll_bottom = self.scroll_y + content_rows - 1;

        if self.cursor_controller.cursor_y > scroll_bottom {
            self.cursor_controller.cursor_y - scroll_bottom + self.scroll_y
        } else if self.cursor_controller.cursor_y < self.scroll_y {
            self.cursor_controller.cursor_y
        } else {
            self.scroll_y
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
    editor_view: EditorView,
}

impl Output {
    const STATUS_BAR_ROWS: usize = 2;
    const INSERT_MODE_LABEL: &'static str = "-- INSERT --";
    const DEFAULT_WINDOW_SIZE: (usize, usize) = (80, 24);
    const COMMAND_CURSOR_Y_OFFSET: usize = 1;

    fn new() -> Self {
        let window_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap_or({
                info!("Could not get window size, using default");
                Self::DEFAULT_WINDOW_SIZE
            });
        Self {
            editor_contents: EditorContents::new(),
            editor_view: EditorView::new(window_size),
        }
    }

    fn clear_screen() -> io::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn calculate_line_percent(lines: &[String], cursor_y: usize) -> String {
        if lines.is_empty() || lines.len() == 1 {
            "Top".to_string()
        } else {
            match 100 * cursor_y / (lines.len() - 1) {
                100 => "Bot".to_string(),
                0 => "Top".to_string(),
                percent => format!("{}%", percent),
            }
        }
    }

    fn draw_rows(&mut self, content: &[String]) {
        self.editor_contents.push_str(&content.join("\n"));
    }

    fn draw_content(&mut self, piece_table: &PieceTable) {
        dbg!(self.editor_view.scroll_y);
        self.editor_view.update_scroll();

        dbg!(self.editor_view.scroll_y);
        let lines = piece_table.lines();
        let start = self.editor_view.scroll_y;
        let end = std::cmp::min(
            lines.len(),
            start + self.editor_view.cursor_controller.screen_rows - 2,
        );

        self.draw_rows(&lines[start..end]);
        self.fill_screen(start + self.editor_view.cursor_controller.screen_rows - end - 1);
    }

    fn fill_screen(&mut self, empty_lines: usize) {
        let lines = vec!["".to_string(); empty_lines];
        self.draw_rows(&lines);
    }

    fn format_status_bar(
        cursor_controller: &CursorController,
        metadata: &FileMetadata,
        line_position: &str,
    ) -> String {
        let right_part = format!(
            "{},{}        {}",
            cursor_controller.cursor_y + 1,
            cursor_controller.cursor_x + 1,
            line_position
        );

        let remaining_space =
            cursor_controller.screen_columns - metadata.file_path.len() - right_part.len();

        format!(
            "\n{}{}{}",
            metadata.file_path,
            " ".repeat(remaining_space),
            right_part
        )
    }

    fn draw_status_bar(&mut self, piece_table: &PieceTable, mode: &Mode, metadata: &FileMetadata) {
        let lines = &piece_table.lines();

        let line_percent =
            Self::calculate_line_percent(lines, self.editor_view.cursor_controller.cursor_y);

        let status_bar =
            Self::format_status_bar(&self.editor_view.cursor_controller, metadata, &line_percent);

        self.editor_contents.push_str(&status_bar);

        let mode_label = match mode {
            Mode::Insert => Self::INSERT_MODE_LABEL.to_string(),
            Mode::Command { previous_chars } => {
                format!(":{}", previous_chars)
            }
            Mode::Normal(Some(BarMode::Write)) => format!(
                "\"{}\" {}L, {}B written",
                metadata.file_path,
                lines.len(),
                metadata.file_size.unwrap_or({
                    info!("File size not found");
                    0
                })
            ),
            _ => "".to_string(),
        };

        self.editor_contents.push_str(&format!("\n{mode_label}"));
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

        self.draw_content(piece_table);
        self.draw_status_bar(piece_table, mode, metadata);

        let (cursor_x, cursor_y) = match mode {
            Mode::Insert => (
                self.editor_view.cursor_controller.cursor_x,
                self.editor_view
                    .cursor_controller
                    .cursor_y
                    .saturating_sub(self.editor_view.scroll_y),
            ),
            Mode::Command { previous_chars } => {
                let cursor_y = self.editor_view.cursor_controller.screen_rows - 1;
                let cursor_x = previous_chars.len() + Output::COMMAND_CURSOR_Y_OFFSET;
                (cursor_x, cursor_y)
            }
            Mode::Normal(Some(BarMode::Write)) => (
                self.editor_view.cursor_controller.cursor_x,
                self.editor_view
                    .cursor_controller
                    .cursor_y
                    .saturating_sub(self.editor_view.scroll_y),
            ),
            _ => (
                self.editor_view.cursor_controller.cursor_x,
                self.editor_view
                    .cursor_controller
                    .cursor_y
                    .saturating_sub(self.editor_view.scroll_y),
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
                &mut self.output.editor_view.cursor_controller,
            ),
            Mode::Insert => self.key_handler.insert_keypress(
                key_event,
                lines,
                &mut self.piece_table,
                &mut self.output.editor_view.cursor_controller,
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
                &mut self.output.editor_view.cursor_controller,
            ),
            Mode::Insert => self.key_handler.insert_keypress(
                key_event,
                lines,
                &mut self.piece_table,
                &mut self.output.editor_view.cursor_controller,
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
