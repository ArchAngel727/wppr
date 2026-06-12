use ratatui::{
    layout::{Constraint, Layout, Size},
    style::{Color, Style},
    widgets::{Block, StatefulWidget, Widget},
};
use ratatui_image::{Resize, StatefulImage, protocol::StatefulProtocol};

pub struct Grid<'a> {
    items: &'a mut [StatefulProtocol],
    cell_size: Size,
    highlight_style: Style,
}

pub struct GridState {
    item_count: usize,
    cells_in_row: usize,
    cells_in_column: usize,
    offset: usize,
    selected: Option<usize>,
}

impl<'a> Grid<'a> {
    pub fn new(items: &'a mut [StatefulProtocol], cell_size: Size) -> Self {
        Self {
            items,
            cell_size,
            highlight_style: Style::default().fg(Color::White),
        }
    }
}

impl<'a> StatefulWidget for Grid<'a> {
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
            let index = i + (state.offset * state.cells_in_row);

            if index >= self.items.len() {
                break;
            }

            let block = if state.selected == Some(index) {
                Block::bordered().border_style(self.highlight_style)
            } else {
                Block::bordered().border_style(Style::new().fg(Color::Black))
            };

            let inner_area = block.inner(*cell);
            block.render(*cell, buf);

            StatefulImage::default().resize(Resize::Fit(None)).render(
                inner_area,
                buf,
                &mut self.items[index],
            );
        }
    }
}

impl GridState {
    pub fn new() -> Self {
        Self {
            item_count: 0,
            cells_in_row: 0,
            cells_in_column: 0,
            offset: 0,
            selected: Some(0),
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn update_item_count(&mut self, count: usize) {
        self.item_count = count;
    }

    pub fn update_size(&mut self, area: &Size, cell_size: &Size) {
        self.cells_in_row = (area.width / cell_size.width) as usize;
        self.cells_in_column = (area.height / cell_size.height) as usize;
    }

    pub fn move_up(&mut self) {
        if let Some(selected) = self.selected
            && selected >= self.cells_in_row
        {
            self.selected = Some(selected - self.cells_in_row);

            if let Some(selected) = self.selected
                && selected < self.offset * self.cells_in_row
            {
                self.offset -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selected) = self.selected
            && selected + self.cells_in_row < self.item_count
        {
            self.selected = Some(selected + self.cells_in_row);

            if let Some(selected) = self.selected
                && selected
                    > (self.offset * self.cells_in_row) + (self.cells_in_row * self.cells_in_column)
                        - 1
            {
                self.offset += 1;
            }
        }
    }

    pub fn move_left(&mut self) {
        if let Some(selected) = self.selected {
            if selected > 0 {
                self.selected = Some(selected - 1);

                // FIX: add scrolling
            } else {
                self.selected = Some(if self.item_count > 0 {
                    self.item_count - 1
                } else {
                    0
                });

                let num_visible_rows = self.cells_in_column;
                let selected_row = selected / self.cells_in_row;

                self.offset = selected_row.saturating_sub(num_visible_rows.saturating_sub(1));
            }
        }
    }

    pub fn move_right(&mut self) {
        if let Some(selected) = self.selected {
            if selected + 1 < self.item_count {
                self.selected = Some(selected + 1);

                // FIX: add scrolling
                let selected_row = selected / self.cells_in_row;
                if selected >= ((selected_row + 1) * self.cells_in_column) - 1 {
                    self.offset += 1;
                }
            } else {
                self.selected = Some(0);
                self.offset = 0;
            }
        }
    }
}
