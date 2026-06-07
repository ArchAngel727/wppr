use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

pub struct Grid<'a, T> {
    items: &'a [T],
    columns: usize,
    rows: usize,
    highlight_style: Style,
}

pub struct GridState {
    item_count: usize,
    column_count: usize,
    offset: usize,
    selected: Option<usize>,
}

impl<'a, T> Grid<'a, T> {
    pub fn new(items: &'a [T], rows: usize, columns: usize) -> Self {
        Self {
            items,
            columns,
            rows,
            highlight_style: Style::default().fg(Color::Black),
        }
    }
}

impl<'a, T: std::fmt::Debug> StatefulWidget for Grid<'a, T> {
    type State = GridState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        let col_constraints = (0..self.columns).map(|_| Constraint::Length(9));
        let row_constraints = (0..self.rows).map(|_| Constraint::Length(3));
        let horizontal = Layout::horizontal(col_constraints).spacing(1);
        let vertical = Layout::vertical(row_constraints).spacing(1);

        let rows = vertical.split(area);
        let cells: Vec<_> = rows
            .iter()
            .flat_map(|&row| horizontal.split(row).to_vec())
            .collect();

        for (i, cell) in cells.iter().enumerate() {
            if i >= self.items.len() {
                continue;
            }

            let block = if state.selected == Some(i) {
                Block::bordered().border_style(self.highlight_style)
            } else {
                Block::bordered()
            };

            Paragraph::new(format!("{:?}", self.items.len()))
                .block(block)
                .render(*cell, buf);
        }
    }
}

impl GridState {
    pub fn new() -> Self {
        Self {
            item_count: 0,
            column_count: 1,
            offset: 0,
            selected: Some(0),
        }
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn update_item_count(&mut self, count: usize) {
        self.item_count = count;
    }

    pub fn update_column_count(&mut self, count: usize) {
        self.column_count = count;
    }

    // pub fn move_up(&mut self) {
    //     if let Some(selected) = self.selected
    //         && selected > self.column_count - 1
    //     {
    //         self.selected = Some(selected - 1);
    //     }
    // }
    //
    // pub fn move_down(&mut self) {
    //     if let Some(selected) = self.selected
    //         && selected % self.column_count + 1 < self.column_count
    //         && selected + self.column_count < self.item_count
    //     {
    //         self.selected = Some(selected + self.column_count);
    //     }
    // }

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
