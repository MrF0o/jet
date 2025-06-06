//! # Text Buffer Management
//!
//! Core text buffer that represents a single document/file.
//!
//! ## What it does
//!
//! - Stores text as lines in memory
//! - Tracks cursor position and text selections  
//! - Handles file loading/saving
//! - Manages undo/redo history
//! - Supports search & replace
//!
//! ## Structure
//!
//! Each buffer keeps track of:
//! - File content (vector of lines)
//! - File path and whether it's been modified
//! - Cursor position and any selected text
//! - Undo history for changes
//!
//! ## Performance
//!
//! Designed to handle large files efficiently while keeping
//! cursor movement and editing operations fast.

use std::path::PathBuf;

#[derive(Clone)]
pub struct Buffer {
    pub content: Vec<String>,
    pub path: Option<PathBuf>,
    pub name: String,
    pub modified: bool,
    pub cursor_pos: (usize, usize),              // (row, column)
    pub selection_start: Option<(usize, usize)>, // Start position of selection (row, column), if any
    pub visual_mode: bool,                       // Whether we're in visual (selection) mode
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: vec![String::new()],
            path: None,
            name: String::from("untitled"),
            modified: false,
            cursor_pos: (0, 0),
            selection_start: None,
            visual_mode: false,
        }
    }

    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        use std::fs;
        use std::io::{BufRead, BufReader};

        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);

        let content: Vec<String> = reader.lines().collect::<Result<Vec<String>, _>>()?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();

        Ok(Self {
            content: if content.is_empty() {
                vec![String::new()]
            } else {
                content
            },
            path: Some(path),
            name,
            modified: false,
            cursor_pos: (0, 0),
            selection_start: None,
            visual_mode: false,
        })
    }

    pub async fn from_path_async(path: PathBuf) -> std::io::Result<Self> {
        use tokio::fs;
        use tokio::io::{AsyncBufReadExt, BufReader};

        let file = fs::File::open(&path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut content = Vec::new();
        while let Some(line) = lines.next_line().await? {
            content.push(line);
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| "untitled".to_owned());

        Ok(Self {
            content: if content.is_empty() {
                vec![String::new()]
            } else {
                content
            },
            path: Some(path),
            name,
            modified: false,
            cursor_pos: (0, 0),
            selection_start: None,
            visual_mode: false,
        })
    }

    /// Load a large file with chunked reading for better performance
    pub async fn from_large_file_async(path: PathBuf, chunk_size: usize) -> std::io::Result<Self> {
        use tokio::fs;
        use tokio::io::{AsyncBufReadExt, BufReader};

        let file = fs::File::open(&path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut content = Vec::new();
        let mut lines_read = 0;

        // Read in chunks to avoid blocking the UI
        while let Some(line) = lines.next_line().await? {
            content.push(line);
            lines_read += 1;

            // Yield control every chunk_size lines
            if lines_read % chunk_size == 0 {
                tokio::task::yield_now().await;
            }
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();

        Ok(Self {
            content: if content.is_empty() {
                vec![String::new()]
            } else {
                content
            },
            path: Some(path),
            name,
            modified: false,
            cursor_pos: (0, 0),
            selection_start: None,
            visual_mode: false,
        })
    }

    /// Get buffer content as a string efficiently without allocating intermediate strings
    /// This is optimized to avoid the expensive `join()` operation on every call
    pub fn content_as_string(&self) -> String {
        // Pre-calculate the total capacity needed
        let total_chars: usize = self.content.iter().map(|line| line.len() + 1).sum(); // +1 for newlines
        let mut result = String::with_capacity(total_chars.saturating_sub(1)); // -1 because last line doesn't need newline

        for (i, line) in self.content.iter().enumerate() {
            result.push_str(line);
            if i < self.content.len() - 1 {
                result.push('\n');
            }
        }

        result
    }

    pub fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor_pos;
        if row >= self.content.len() {
            self.content.push(String::new());
        }

        let line = &mut self.content[row];
        if col > line.len() {
            line.push_str(&" ".repeat(col - line.len()));
        }

        line.insert(col, c);
        self.cursor_pos.1 += 1;
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        let (row, col) = self.cursor_pos;
        if row >= self.content.len() {
            self.content.push(String::new());
            self.cursor_pos = (row + 1, 0);
            return;
        }

        if col < self.content[row].len() {
            // Split the line at cursor position without creating intermediate strings
            let mut new_line = String::new();
            new_line.push_str(&self.content[row][col..]);
            self.content[row].truncate(col);
            self.content.insert(row + 1, new_line);
        } else {
            // Cursor is at end of line, just insert empty line
            self.content.insert(row + 1, String::new());
        }

        self.cursor_pos = (row + 1, 0);
        self.modified = true;
    }

    pub fn backspace(&mut self) {
        let (row, col) = self.cursor_pos;
        if col > 0 {
            // Delete character before cursor
            let line = &mut self.content[row];
            line.remove(col - 1);
            self.cursor_pos.1 -= 1;
        } else if row > 0 {
            // Join with previous line
            let current_line = self.content.remove(row);
            let prev_line = &mut self.content[row - 1];
            let new_cursor_col = prev_line.len();
            prev_line.push_str(&current_line);
            self.cursor_pos = (row - 1, new_cursor_col);
        }
        self.modified = true;
    }

    pub fn delete(&mut self) {
        let (row, col) = self.cursor_pos;
        if row < self.content.len() {
            let line = &mut self.content[row];
            if col < line.len() {
                // Delete character at cursor
                line.remove(col);
            } else if row + 1 < self.content.len() {
                // Join with next line
                let next_line = self.content.remove(row + 1);
                self.content[row].push_str(&next_line);
            }
            self.modified = true;
        }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            use std::fs;
            use std::io::Write;

            let mut file = fs::File::create(path)?;
            for line in &self.content {
                writeln!(file, "{}", line)?;
            }
            self.modified = false;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No file path specified",
            ))
        }
    }

    /// Save buffer content to its associated file path asynchronously
    pub async fn save_async(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            self.save_to_path_async(path.clone()).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No file path associated with buffer",
            ))
        }
    }

    /// Save buffer content to a specific path asynchronously
    pub async fn save_to_path_async(&mut self, path: PathBuf) -> std::io::Result<()> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        let content = self.content_as_string();
        let mut file = fs::File::create(&path).await?;
        file.write_all(content.as_bytes()).await?;
        file.sync_all().await?;

        self.modified = false;
        self.path = Some(path.clone());
        self.name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();

        Ok(())
    }

    /// Toggle visual (selection) mode
    pub fn toggle_visual_mode(&mut self) {
        self.visual_mode = !self.visual_mode;

        if self.visual_mode {
            // Start selection at current cursor position
            self.selection_start = Some(self.cursor_pos);
        } else {
            // Clear selection when exiting visual mode
            self.selection_start = None;
        }
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.visual_mode = false;
    }

    /// Check if the buffer has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.modified
    }

    /// Get the current selection range as (start_pos, end_pos)
    /// Returns None if there is no selection
    pub fn get_selection_range(&self) -> Option<(Position, Position)> {
        self.selection_start.map(|start| {
            let end = self.cursor_pos;

            // Convert to Position objects
            let start_pos = Position::from_tuple(start);
            let end_pos = Position::from_tuple(end);

            // Ensure start is before end for consistent ordering
            if start_pos <= end_pos {
                (start_pos, end_pos)
            } else {
                (end_pos, start_pos)
            }
        })
    }

    /// Get the text content of the current selection
    pub fn get_selected_text(&self) -> Option<String> {
        self.get_selection_range().map(|(start, end)| {
            // If selection is within a single line
            if start.row == end.row {
                let line = &self.content[start.row];
                return line[start.col..end.col].to_string();
            }

            // Pre-calculate capacity for multi-line selection to reduce allocations
            let mut estimated_capacity = 0;
            for row in start.row..=end.row {
                if row < self.content.len() {
                    if row == start.row {
                        // First line: from start.col to end
                        estimated_capacity += self.content[row].len().saturating_sub(start.col) + 1;
                    // +1 for newline
                    } else if row == end.row {
                        // Last line: from start to end.col
                        estimated_capacity += end.col.min(self.content[row].len());
                    } else {
                        // Middle lines: whole line + newline
                        estimated_capacity += self.content[row].len() + 1;
                    }
                }
            }

            let mut selected_text = String::with_capacity(estimated_capacity);

            // Selection spans multiple lines
            // First line (from start to end of line)
            if start.row < self.content.len() {
                let line = &self.content[start.row];
                if start.col < line.len() {
                    selected_text.push_str(&line[start.col..]);
                }
                selected_text.push('\n');
            }

            // Middle lines (whole lines)
            for row in (start.row + 1)..end.row {
                if row < self.content.len() {
                    selected_text.push_str(&self.content[row]);
                    selected_text.push('\n');
                }
            }

            // Last line (from start of line to end)
            if end.row < self.content.len() {
                let line = &self.content[end.row];
                let end_col = end.col.min(line.len());
                selected_text.push_str(&line[..end_col]);
            }

            selected_text
        })
    }

    /// Delete the selected text
    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.get_selection_range() {
            // Handle single-line selection
            if start.row == end.row {
                if start.row < self.content.len() {
                    let line = &mut self.content[start.row];
                    if start.col < line.len() {
                        line.replace_range(start.col..end.col.min(line.len()), "");
                    }
                }
            } else {
                // Handle multi-line selection
                if start.row < self.content.len() && end.row < self.content.len() {
                    let first_line = &self.content[start.row];
                    let last_line = &self.content[end.row];

                    // Calculate new line capacity
                    let prefix_len = start.col.min(first_line.len());
                    let suffix_start = end.col.min(last_line.len());
                    let suffix_len = last_line.len() - suffix_start;

                    // Create combined line efficiently
                    let mut new_line = String::with_capacity(prefix_len + suffix_len);
                    new_line.push_str(&first_line[..prefix_len]);
                    new_line.push_str(&last_line[suffix_start..]);

                    // Remove lines between start and end
                    self.content
                        .splice(start.row..(end.row + 1), vec![new_line]);
                } else {
                    // Fallback for edge cases
                    let first_line_prefix = if start.row < self.content.len() {
                        let line = &self.content[start.row];
                        line[..start.col.min(line.len())].to_string()
                    } else {
                        String::new()
                    };

                    let last_line_suffix = if end.row < self.content.len() {
                        let line = &self.content[end.row];
                        line[end.col.min(line.len())..].to_string()
                    } else {
                        String::new()
                    };

                    // Combine first line prefix with last line suffix
                    let new_line = first_line_prefix + &last_line_suffix;

                    // Remove lines between start and end
                    self.content
                        .splice(start.row..(end.row + 1), vec![new_line]);
                }
            }

            // Set cursor to the start of the deleted selection
            self.cursor_pos = start.to_tuple();
            self.clear_selection();
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn move_cursor(&mut self, direction: CursorMovement) {
        let (mut row, mut col) = self.cursor_pos;

        match direction {
            CursorMovement::Up => {
                if row > 0 {
                    row -= 1;
                    // Adjust column if the line is shorter
                    let line = &self.content[row];
                    col = col.min(line.len());
                }
            }
            CursorMovement::Down => {
                if row + 1 < self.content.len() {
                    row += 1;
                    // Adjust column if the line is shorter
                    let line = &self.content[row];
                    col = col.min(line.len());
                }
            }
            CursorMovement::Left => {
                if col > 0 {
                    col -= 1;
                } else if row > 0 {
                    row -= 1;
                    col = self.content[row].len();
                }
            }
            CursorMovement::Right => {
                let line = &self.content[row];
                if col < line.len() {
                    col += 1;
                } else if row + 1 < self.content.len() {
                    row += 1;
                    col = 0;
                }
            }
            CursorMovement::LineStart => {
                col = 0;
            }
            CursorMovement::LineEnd => {
                if row < self.content.len() {
                    col = self.content[row].len();
                }
            }
            CursorMovement::PageUp => {
                // Use a larger number for page scrolling (default to 8 but will be overridden by actual area height)
                // This is just a fallback if the terminal size isn't available
                let page_size = 8;
                if row >= page_size {
                    row -= page_size;
                } else {
                    row = 0;
                }
                // Adjust column if needed
                let line = &self.content[row];
                col = col.min(line.len());
            }
            CursorMovement::PageDown => {
                // Use a larger number for page scrolling (default to 8 but will be overridden by actual area height)
                let page_size = 8;
                if row + page_size < self.content.len() {
                    row += page_size;
                } else {
                    row = self.content.len() - 1;
                }
                // Adjust column if needed
                let line = &self.content[row];
                col = col.min(line.len());
            }
            CursorMovement::BufferStart => {
                row = 0;
                col = 0;
            }
            CursorMovement::BufferEnd => {
                if self.content.is_empty() {
                    row = 0;
                    col = 0;
                } else {
                    row = self.content.len() - 1;
                    col = self.content[row].len();
                }
            }
        }

        // Update cursor position
        self.cursor_pos = (row, col);

        // Update selection if in visual mode
        if self.visual_mode && self.selection_start.is_none() {
            // Start selection from the original position if none exists
            self.selection_start = Some(self.cursor_pos);
        }
    }

    /// Count the number of digits in a number
    pub fn count_digits(mut n: usize) -> usize {
        if n == 0 {
            return 1;
        }
        let mut digits = 0;
        while n > 0 {
            digits += 1;
            n /= 10;
        }
        digits
    }

    /// Get the width needed for line numbers display
    /// Always reserves space for at least 4 digits to prevent UI shifts
    pub fn line_number_width(&self) -> usize {
        let total_lines = self.content.len().max(1);
        let calculated_width = Self::count_digits(total_lines);
        // Reserve space for at least 4 digits (up to 9999 lines) to prevent UI shifts
        let min_width = 4;
        calculated_width.max(min_width) + 1 // +1 for spacing
    }
}

pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    LineStart,
    LineEnd,
    PageUp,
    PageDown,
    BufferStart,
    BufferEnd,
}

/// Represents a text position (row, column)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    /// Create a new position
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Convert from tuple
    pub fn from_tuple(pos: (usize, usize)) -> Self {
        Self {
            row: pos.0,
            col: pos.1,
        }
    }

    /// Convert to tuple
    pub fn to_tuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}
