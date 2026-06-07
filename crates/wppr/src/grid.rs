use std::{fs, path::PathBuf};

use ratatui::{
    layout::{Constraint, Layout, Size},
    style::{Color, Style},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

pub struct Grid<'a, T> {
    items: &'a [T],
    cell_size: Size,
    highlight_style: Style,
}

pub struct GridState {
    item_count: usize,
    row_count: usize,
    column_count: usize,
    offset: usize,
    selected: Option<usize>,
}

impl<'a, T: std::fmt::Display> Grid<'a, T> {
    pub fn new(items: &'a [T], cell_size: Size) -> Self {
        Self {
            items,
            cell_size,
            highlight_style: Style::default().fg(Color::Black),
        }
    }
}

impl<'a, T: std::fmt::Display> StatefulWidget for Grid<'a, T> {
    type State = GridState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        let columns = area.width / self.cell_size.width;
        let rows = area.height / self.cell_size.height;

        let col_constraints = (0..columns).map(|_| Constraint::Length(self.cell_size.width));
        let row_constraints = (0..rows).map(|_| Constraint::Length(self.cell_size.height));
        let horizontal = Layout::horizontal(col_constraints);
        let vertical = Layout::vertical(row_constraints);

        let rows = vertical.split(area);
        let cells: Vec<_> = rows
            .iter()
            .flat_map(|&row| horizontal.split(row).to_vec())
            .collect();

        for (i, cell) in cells.iter().enumerate() {
            let index = i + (state.offset * state.column_count);

            if index >= self.items.len() {
                continue;
            }

            let block = if state.selected == Some(index) {
                Block::bordered().border_style(self.highlight_style)
            } else {
                Block::bordered()
            };

            Paragraph::new(format!(
                "index: {}\noffset: {}\ncol*off: {}\nselected: {}",
                // self.items[i + (state.offset * state.row_count)]
                // i + (state.offset * state.row_count)
                index,
                state.offset,
                state.offset * state.column_count,
                state.selected.unwrap()
            ))
            .block(block)
            .render(*cell, buf);
        }
    }
}

impl GridState {
    pub fn new() -> Self {
        Self {
            item_count: 0,
            row_count: 0,
            column_count: 0,
            offset: 0,
            selected: Some(0),
        }
    }

    pub fn update_item_count(&mut self, count: usize) {
        self.item_count = count;
    }

    pub fn update_size(&mut self, area: &Size, cell_size: &Size) {
        self.column_count = (area.width / cell_size.width) as usize;
        self.row_count = (area.height / cell_size.height) as usize;
    }

    pub fn move_up(&mut self) {
        if let Some(selected) = self.selected
            && selected >= self.column_count
        {
            self.selected = Some(selected - self.column_count);

            if let Some(selected) = self.selected
                && selected < self.offset * self.column_count
            {
                self.offset -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selected) = self.selected
            && selected + self.column_count < self.item_count
        {
            self.selected = Some(selected + self.column_count);

            if let Some(selected) = self.selected
                && selected
                    > (self.offset * self.column_count + self.row_count * self.column_count) - 1
            {
                self.offset += 1;
            }
        }
    }

    pub fn move_left(&mut self) {
        if let Some(selected) = self.selected
            && selected > 0
        {
            self.selected = Some(selected - 1);
        }
    }

    pub fn move_right(&mut self) {
        if let Some(selected) = self.selected
            && selected + 1 < self.item_count
        {
            self.selected = Some(selected + 1);
        }
    }
}
