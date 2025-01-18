use std::fmt;

#[derive(Debug)]
pub enum FindIndexError {
    OutOfBounds,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BufferType {
    Original,
    Added,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Piece {
    pub source: BufferType,
    pub start_index: usize,
    pub length: usize,
}

pub struct PieceTable {
    pub original: String,
    pub added: String,
    pub table: Vec<Piece>,
}

fn substring(s: &str, start: usize, length: usize) -> &str {
    let mut char_indices = s.char_indices();
    let start_byte = char_indices
        .nth(start)
        .map(|(idx, _)| idx)
        .unwrap_or(s.len());
    let end_byte = char_indices
        .nth(length - 1)
        .map(|(idx, _)| idx)
        .unwrap_or(s.len());
    &s[start_byte..end_byte]
}

impl Default for PieceTable {
    fn default() -> Self {
        PieceTable::new("")
    }
}

impl PieceTable {
    pub fn new(original_contents: &str) -> Self {
        if original_contents.is_empty() {
            return Self {
                original: "".to_string(),
                added: "".to_string(),
                table: vec![],
            };
        }

        Self {
            original: original_contents.to_string(),
            added: "".to_string(),
            table: vec![Piece {
                source: BufferType::Original,
                start_index: 0,
                length: original_contents.len(),
            }],
        }
    }

    pub fn merge(&mut self) {
        let mut entry_no = 1;
        while entry_no < self.table.len() {
            let last_entry = &self.table[entry_no - 1];
            let cur_entry = &self.table[entry_no];

            if last_entry.start_index + last_entry.length == cur_entry.start_index
                && last_entry.source == cur_entry.source
            {
                let new_entry = Piece {
                    source: last_entry.source.clone(),
                    start_index: last_entry.start_index,
                    length: last_entry.length + cur_entry.length,
                };
                self.table.insert(entry_no - 1, new_entry);
                self.table.remove(entry_no);
                self.table.remove(entry_no);
            } else {
                entry_no += 1;
            }
        }
    }

    pub fn append(&mut self, text: &str) {
        let position = self.table.iter().map(|piece| piece.length).sum::<usize>();

        self.insert(position, text);
    }

    pub fn lines(&self) -> Vec<String> {
        self.to_string()
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    pub fn find_index(&self, x: usize, y: usize) -> Result<usize, FindIndexError> {
        let mut cur_index = 0;
        for (cur_y, line) in self.to_string().lines().enumerate() {
            if line.is_empty() && cur_y == y {
                return Ok(cur_index);
            }
            for (cur_x, _) in line.chars().enumerate() {
                if cur_x == x && cur_y == y {
                    return Ok(cur_index);
                }
                cur_index += 1;
                if cur_x == x && cur_y == y {
                    return Ok(cur_index);
                }
            }
            cur_index += 1;
        }
        Err(FindIndexError::OutOfBounds)
    }

    pub fn index(&self, i: usize) -> Option<char> {
        let total_length = self.table.iter().map(|piece| piece.length).sum();
        if i >= total_length {
            return None;
        }

        let mut cur_index = 0;
        for entry in self.table.iter() {
            let end_index = cur_index + entry.length;
            if i >= cur_index && i < end_index {
                let buffer_index = entry.start_index + i - cur_index;
                return match &entry.source {
                    BufferType::Original => self.original.chars().nth(buffer_index),
                    BufferType::Added => self.added.chars().nth(buffer_index),
                };
            }
            cur_index += entry.length;
        }
        None
    }

    pub fn delete(&mut self, position: usize) {
        let mut cur_index = 0;
        let mut split_index = None;
        for (entry_index, entry) in self.table.iter().enumerate() {
            let end_index = cur_index + entry.length;
            if position >= cur_index && position < end_index {
                split_index = Some((entry_index, position - cur_index));
            }
            cur_index += entry.length;
        }

        if let Some((entry_index, entry_position)) = split_index {
            let original_entry = self.table[entry_index].clone();
            let original_buffer_type = &original_entry.source;

            if entry_position == 0 {
                let new_entry = Piece {
                    source: original_buffer_type.clone(),
                    start_index: original_entry.start_index + 1,
                    length: original_entry.length - 1,
                };

                self.table.remove(entry_index);
                self.table.insert(entry_index, new_entry);
            } else if entry_position == original_entry.length {
                let new_entry = Piece {
                    source: original_buffer_type.clone(),
                    start_index: original_entry.start_index,
                    length: original_entry.length,
                };

                self.table.remove(entry_index);
                self.table.insert(entry_index, new_entry);
            } else {
                let first_entry = Piece {
                    source: original_buffer_type.clone(),
                    start_index: original_entry.start_index,
                    length: entry_position,
                };

                let second_length = original_entry.length - entry_position;
                let second_entry = Piece {
                    source: original_buffer_type.clone(),
                    start_index: original_entry.start_index + entry_position + 1,
                    length: second_length - 1,
                };

                self.table.remove(entry_index);
                self.table.insert(entry_index, second_entry);
                self.table.insert(entry_index, first_entry);
            }
        }
    }

    pub fn insert(&mut self, position: usize, text: &str) {
        let added_start_index = self.added.len();
        self.added.push_str(text);
        let mut cur_index = 0;
        let mut split_index = None;
        for (entry_index, entry) in self.table.iter_mut().enumerate() {
            let end_index = cur_index + entry.length;
            if position >= cur_index && position < end_index {
                split_index = Some((entry_index, position - cur_index));
                break;
            }
            cur_index += entry.length;
        }
        if let Some((entry_index, entry_position)) = split_index {
            let original_entry = self.table[entry_index].clone();

            let original_buffer_type = &original_entry.source;
            if entry_position == 0 {
                let added = Piece {
                    source: BufferType::Added,
                    start_index: added_start_index,
                    length: text.len(),
                };

                self.table.insert(entry_index, added)
            } else if entry_position == original_entry.length {
                let added = Piece {
                    source: BufferType::Added,
                    start_index: added_start_index,
                    length: text.len(),
                };

                self.table.insert(entry_index + 1, added);
            } else {
                let first_length = entry_position;
                let first = Piece {
                    source: original_buffer_type.clone(),
                    length: first_length,
                    start_index: original_entry.start_index,
                };

                let second_length = text.len();
                let middle = Piece {
                    source: BufferType::Added,
                    length: second_length,
                    start_index: added_start_index,
                };

                let last_length = original_entry.length - first_length;
                let last = Piece {
                    source: original_buffer_type.clone(),
                    length: last_length,
                    start_index: first_length + original_entry.start_index,
                };

                self.table.remove(entry_index);
                self.table.insert(entry_index, last);
                self.table.insert(entry_index, middle);
                self.table.insert(entry_index, first);
            }
        } else {
            let added = Piece {
                source: BufferType::Added,
                length: text.len(),
                start_index: added_start_index,
            };
            self.table.push(added);
        }
    }
}

impl fmt::Display for PieceTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        for entry in self.table.iter() {
            if entry.length == 0 {
                continue;
            }
            match entry.source {
                BufferType::Original => {
                    let text = substring(&self.original, entry.start_index, entry.length);
                    string.push_str(text);
                }
                BufferType::Added => {
                    let text = substring(&self.added, entry.start_index, entry.length);
                    string.push_str(text);
                }
            }
        }
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete() {
        let original = String::from("the quick brown fox\njumped over the lazy dog");
        let added = String::new();
        let table = vec![Piece {
            source: BufferType::Original,
            start_index: 0,
            length: 44,
        }];

        let mut piece_table = PieceTable {
            original,
            added,
            table,
        };

        println!("{}", piece_table);

        piece_table.delete(16);

        println!("{}", piece_table);

        let expected = vec![
            Piece {
                source: BufferType::Original,
                start_index: 0,
                length: 16,
            },
            Piece {
                source: BufferType::Original,
                start_index: 17,
                length: 27,
            },
        ];

        let result = piece_table.table;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert() {
        let original = String::from("the quick brown fox\njumped over the lazy dog");
        let added = String::new();
        let table = vec![Piece {
            source: BufferType::Original,
            start_index: 0,
            length: 44,
        }];

        let mut piece_table = PieceTable {
            original,
            added,
            table,
        };

        piece_table.insert(20, "went to the park and\n");

        let expected = vec![
            Piece {
                source: BufferType::Original,
                start_index: 0,
                length: 20,
            },
            Piece {
                source: BufferType::Added,
                start_index: 0,
                length: 21,
            },
            Piece {
                source: BufferType::Original,
                start_index: 20,
                length: 24,
            },
        ];

        println!("{}", piece_table);

        let result = piece_table.table;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_index() {
        let original = String::from("ipsum sit amet");
        let added = String::from("Lorem deletedtext dolor");
        let table = vec![
            Piece {
                source: BufferType::Added,
                start_index: 0,
                length: 6,
            },
            Piece {
                source: BufferType::Original,
                start_index: 0,
                length: 5,
            },
            Piece {
                source: BufferType::Added,
                start_index: 17,
                length: 6,
            },
            Piece {
                source: BufferType::Original,
                start_index: 5,
                length: 9,
            },
        ];
        let table = PieceTable {
            original,
            added,
            table,
        };

        let result = table.index(15);
        assert_eq!(result, Some('o'));
    }
}
