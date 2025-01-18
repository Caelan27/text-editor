use crate::editor::{CursorController, KeyHandler, Mode};
use crate::file;
use crate::piece_table::PieceTable;
use crate::utils::find_index;
use crossterm::event;
use crossterm::event::*;
use std::io;

impl Default for KeyHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyHandler {
    pub fn new() -> Self {
        KeyHandler { mode: Mode::Normal }
    }

    pub fn insert_keypress(
        &mut self,
        key_event: KeyEvent,
        lines: Vec<String>,
        piece_table: &mut PieceTable,
        cursor_controller: &mut CursorController,
    ) -> io::Result<bool> {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => handle_escape_key(cursor_controller, &mut self.mode),

            KeyEvent {
                code: KeyCode::Char(ch),
                ..
            } => type_char(lines, piece_table, cursor_controller, ch),

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => enter(lines, piece_table, cursor_controller),

            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => backspace(lines, piece_table, cursor_controller),

            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => delete(lines, piece_table, cursor_controller),
            _ => {}
        }
        cursor_controller.desired_cursor_x = cursor_controller.cursor_x;
        Ok(true)
    }

    pub fn normal_keypress(
        &mut self,
        key_event: KeyEvent,
        lines: Vec<String>,
        file_path: String,
        piece_table: &mut PieceTable,
        cursor_controller: &mut CursorController,
    ) -> io::Result<bool> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return quit(),

            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => {
                write_file(piece_table, file_path);
            }

            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => move_left(cursor_controller),

            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            } => move_down(cursor_controller, &lines),

            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => move_up(cursor_controller, &lines),

            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => move_right(cursor_controller, &lines),

            KeyEvent {
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
                ..
            } => handle_insert_key(cursor_controller, &mut self.mode, false),

            KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
                ..
            } => handle_insert_key(cursor_controller, &mut self.mode, true),

            _ => {}
        }

        Ok(true)
    }
}

fn quit() -> io::Result<bool> {
    Ok(false)
}

fn write_file(piece_table: &mut PieceTable, file_path: String) {
    let _ = file::save_file(&file_path, piece_table.to_string());
}

fn move_left(cursor_controller: &mut CursorController) {
    cursor_controller.cursor_x = cursor_controller.cursor_x.saturating_sub(1);
    cursor_controller.desired_cursor_x = cursor_controller.cursor_x;
}

fn move_down(cursor_controller: &mut CursorController, lines: &[String]) {
    if cursor_controller.cursor_y < lines.len() - 1 {
        cursor_controller.cursor_y += 1;
        cursor_controller.cursor_x = cursor_controller
            .desired_cursor_x
            .min(lines[cursor_controller.cursor_y].len().saturating_sub(1));
    }
}

fn move_up(cursor_controller: &mut CursorController, lines: &[String]) {
    cursor_controller.cursor_y = cursor_controller.cursor_y.saturating_sub(1);
    cursor_controller.cursor_x = cursor_controller
        .desired_cursor_x
        .min(lines[cursor_controller.cursor_y].len().saturating_sub(1));
}

fn move_right(cursor_controller: &mut CursorController, lines: &[String]) {
    cursor_controller.cursor_x += 1;
    cursor_controller.cursor_x = cursor_controller
        .cursor_x
        .min(lines[cursor_controller.cursor_y].len().saturating_sub(1));
    cursor_controller.desired_cursor_x = cursor_controller.cursor_x;
}

fn delete(
    lines: Vec<String>,
    piece_table: &mut PieceTable,
    cursor_controller: &mut CursorController,
) {
    if cursor_controller.cursor_x != lines[cursor_controller.cursor_y].len() {
        if let Some(position) = find_index(
            &lines,
            cursor_controller.cursor_x,
            cursor_controller.cursor_y,
        ) {
            piece_table.delete(position);
            dbg!("1");
        }
    } else if let Some(position) = find_index(&lines, 0, cursor_controller.cursor_y + 1) {
        dbg!("dbg");
        piece_table.delete(position - 1);
    }
}

fn backspace(
    lines: Vec<String>,
    piece_table: &mut PieceTable,
    cursor_controller: &mut CursorController,
) {
    let x = cursor_controller.cursor_x.saturating_sub(1);
    let y = cursor_controller.cursor_y;

    if cursor_controller.cursor_x == 0 {
        if cursor_controller.cursor_y == 0 {
            return;
        }
        if let Some(position) = find_index(&lines, 0, cursor_controller.cursor_y) {
            piece_table.delete(position - 1);
            cursor_controller.cursor_x = lines[cursor_controller.cursor_y - 1].len();
            cursor_controller.cursor_y -= 1;
        }
    } else if let Some(position) = find_index(&lines, x, y) {
        piece_table.delete(position);
        cursor_controller.cursor_x = x;
    }
}

fn enter(
    lines: Vec<String>,
    piece_table: &mut PieceTable,
    cursor_controller: &mut CursorController,
) {
    let x = cursor_controller.cursor_x;
    let y = cursor_controller.cursor_y;
    let position = find_index(&lines, x, y).unwrap();
    piece_table.insert(position, "\n");
    cursor_controller.cursor_y += 1;
    cursor_controller.cursor_x = 0;
}

fn type_char(
    lines: Vec<String>,
    piece_table: &mut PieceTable,
    cursor_controller: &mut CursorController,
    ch: char,
) {
    let x = cursor_controller.cursor_x;
    let y = cursor_controller.cursor_y;
    let position = find_index(&lines, x, y).unwrap();
    piece_table.insert(position, &ch.to_string());
    cursor_controller.cursor_x += 1;
}

fn handle_escape_key(cursor_controller: &mut CursorController, mode: &mut Mode) {
    if cursor_controller.cursor_x != 0 {
        cursor_controller.cursor_x -= 1;
    }
    switch_mode(Mode::Normal, mode);
}

fn handle_insert_key(cursor_controller: &mut CursorController, mode: &mut Mode, shift_right: bool) {
    switch_mode(Mode::Insert, mode);
    if shift_right {
        cursor_controller.cursor_x += 1;
    }
}

fn switch_mode(target_mode: Mode, cur_mode: &mut Mode) {
    *cur_mode = target_mode;
}
