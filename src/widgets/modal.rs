use crate::widgets::cursor::CursorSupport;
use ratatui::prelude::Position;
use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// A beautiful modal widget for commands and dialogs
pub struct Modal<'a> {
    title: &'a str,
    content: Vec<Line<'a>>,
    width: u16,
    height: u16,
    focused: bool,
}

impl<'a> Modal<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            content: Vec::new(),
            width: 60,
            height: 20,
            focused: true,
        }
    }

    pub fn content(mut self, content: Vec<Line<'a>>) -> Self {
        self.content = content;
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Calculate the centered area for the modal
    fn centered_rect(&self, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(self.height)) / 2),
                Constraint::Length(self.height),
                Constraint::Min(0),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width.saturating_sub(self.width)) / 2),
                Constraint::Length(self.width),
                Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Widget for Modal<'_> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let modal_area = self.centered_rect(area);

        // Clear the background
        Clear.render(modal_area, buf);

        // Create the modal style based on focus
        let border_style = if self.focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title_style = if self.focused {
            Style::default()
                .fg(Color::White)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray).bg(Color::DarkGray)
        };

        // Create the block with beautiful borders
        let block = Block::default()
            .title(Span::styled(format!(" {} ", self.title), title_style))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(Color::Black));

        let inner_area = block.inner(modal_area);
        block.render(modal_area, buf);

        // Render content
        let paragraph = Paragraph::new(self.content)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);

        paragraph.render(inner_area, buf);
    }
}

/// Command palette modal with input and suggestions
pub struct CommandPalette<'a> {
    input: &'a str,
    suggestions: Vec<&'a str>,
    selected: usize,
    focused: bool,
}

impl<'a> CommandPalette<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            suggestions: Vec::new(),
            selected: 0,
            focused: true,
        }
    }

    pub fn suggestions(mut self, suggestions: Vec<&'a str>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected.min(self.suggestions.len().saturating_sub(1));
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Calculate the centered area for the command palette
    fn centered_rect(&self, area: Rect) -> Rect {
        let height = (self.suggestions.len() as u16 + 3).min(15); // +3 for input and borders
        let width = 80.min(area.width.saturating_sub(4));

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(height)) / 3), // Position in upper third
                Constraint::Length(height),
                Constraint::Min(0),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width.saturating_sub(width)) / 2),
                Constraint::Length(width),
                Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Widget for CommandPalette<'_> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        let modal_area = self.centered_rect(area);

        // Clear the background with a subtle shadow effect
        Clear.render(modal_area, buf);

        // Create gradient-like border style
        let border_style = if self.focused {
            Style::default()
                .fg(Color::Rgb(0, 150, 255)) // Blue gradient
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(Span::styled(
                " Command Palette ",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(0, 100, 200))
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(Color::Rgb(20, 20, 30))); // Dark blue background

        let inner_area = block.inner(modal_area);
        block.render(modal_area, buf);

        // Split into input area and suggestions area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner_area);

        // Render input line with prompt (cursor handled by cursor manager)
        let input_line = Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(self.input, Style::default().fg(Color::White)),
            // Cursor is now handled by the global cursor manager
        ]);

        let input_paragraph =
            Paragraph::new(input_line).style(Style::default().bg(Color::Rgb(30, 30, 50)));

        input_paragraph.render(chunks[0], buf);

        // Render suggestions
        if !self.suggestions.is_empty() && chunks.len() > 1 {
            let suggestion_lines: Vec<Line> = self
                .suggestions
                .iter()
                .enumerate()
                .map(|(i, suggestion)| {
                    if i == self.selected {
                        Line::from(Span::styled(
                            format!("  {} ", suggestion),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        Line::from(Span::styled(
                            format!("  {} ", suggestion),
                            Style::default().fg(Color::LightBlue),
                        ))
                    }
                })
                .collect();

            let suggestions_paragraph = Paragraph::new(suggestion_lines);
            suggestions_paragraph.render(chunks[1], buf);
        }
    }
}

impl CursorSupport for CommandPalette<'_> {
    /// Calculate the cursor position within the command palette input field
    fn calculate_cursor_position(&self, logical_pos: (usize, usize), area: Rect) -> Position {
        let modal_area = self.centered_rect(area);

        // Calculate the inner area of the modal (inside borders)
        let inner_area = Block::default().borders(Borders::ALL).inner(modal_area);

        // The input is on the first line of the inner area
        // Cursor position is: modal_x + border(1) + prompt(2) + input_position
        let cursor_x = inner_area.x + 2 + logical_pos.0 as u16; // 2 for "> " prompt
        let cursor_y = inner_area.y; // First line of inner area

        Position::new(cursor_x, cursor_y)
    }

    /// Get the cursor context identifier for this widget
    fn get_cursor_context(&self) -> &str {
        "command_palette"
    }
}
