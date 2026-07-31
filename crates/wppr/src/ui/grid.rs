use ratatui::{
    layout::{Constraint, Layout, Size},
    style::{Color, Style},
    widgets::{Block, StatefulWidget, Widget},
};
use ratatui_image::{Resize, StatefulImage, protocol::StatefulProtocol};

pub struct Grid<'a> {
    items: &'a mut [&'a mut StatefulProtocol],
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
    pub fn new(items: &'a mut [&'a mut StatefulProtocol], cell_size: Size) -> Self {
        Self {
            items,
            cell_size,
            highlight_style: Style::default().fg(Color::White),
        }
    }
}

impl StatefulWidget for Grid<'_> {
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
                self.items[index],
            );
        }
    }
}

impl GridState {
    pub const fn new() -> Self {
        Self {
            item_count: 0,
            cells_in_row: 0,
            cells_in_column: 0,
            offset: 0,
            selected: Some(0),
        }
    }

    pub const fn select(&mut self, sel: usize) {
        self.selected = Some(sel);
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn set_offset(&mut self, off: usize) {
        self.offset = off;
    }

    pub const fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub const fn item_count(&self) -> usize {
        self.item_count
    }

    pub const fn update_item_count(&mut self, count: usize) {
        self.item_count = count;
    }

    pub const fn update_size(&mut self, area: Size, cell_size: Size) {
        self.cells_in_row = (area.width / cell_size.width) as usize;
        self.cells_in_column = (area.height / cell_size.height) as usize;
    }

    pub const fn cells_in_row(&self) -> usize {
        self.cells_in_row
    }

    pub const fn cells_in_column(&self) -> usize {
        self.cells_in_column
    }

    const fn can_move_up(&self) -> bool {
        let Some(selected) = self.selected else {
            return false;
        };

        selected < self.offset * self.cells_in_row
    }

    const fn can_move_down(&self) -> bool {
        let Some(selected) = self.selected else {
            return false;
        };

        selected
            > (self.offset * self.cells_in_row) + (self.cells_in_row * self.cells_in_column) - 1
    }

    pub const fn move_up(&mut self) {
        if let Some(selected) = self.selected
            && selected >= self.cells_in_row
        {
            self.selected = Some(selected - self.cells_in_row);

            if self.can_move_up() {
                self.offset -= 1;
            }
        }
    }

    pub const fn move_down(&mut self) {
        if let Some(selected) = self.selected
            && selected + self.cells_in_row < self.item_count
        {
            self.selected = Some(selected + self.cells_in_row);

            if self.can_move_down() {
                self.offset += 1;
            }
        }
    }

    pub const fn move_left(&mut self) {
        if let Some(selected) = self.selected
            && selected > 0
        {
            self.selected = Some(selected - 1);

            if self.can_move_up() {
                self.offset -= 1;
            }
        }
    }

    pub const fn move_right(&mut self) {
        if let Some(selected) = self.selected
            && selected + 1 < self.item_count
        {
            self.selected = Some(selected + 1);

            if self.can_move_down() {
                self.offset += 1;
            }
        }
    }
}
