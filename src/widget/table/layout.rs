use std::ops::RangeInclusive;

use ratatui::layout::Rect;

use super::primitives::{CellRect, RegionRect, VisibleCapacity};

#[derive(Debug, Clone, Copy)]
pub struct GridMetrics {
    pub col_width: u16,
    pub row_num_width: u16,
    pub rows_per_cell: u16,
}

impl GridMetrics {
    pub fn new(col_width: u16, row_num_width: u16, rows_per_cell: u16) -> Self {
        Self {
            col_width,
            row_num_width,
            rows_per_cell,
        }
    }

    pub fn layout(self, area: Rect) -> GridLayout {
        GridLayout {
            metrics: self,
            area,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridLayout {
    metrics: GridMetrics,
    area: Rect,
}

impl GridLayout {
    pub fn visible_capacity(&self) -> VisibleCapacity {
        let rows =
            (self.area.height.saturating_sub(2) / self.metrics.rows_per_cell) as usize;
        let cols = ((self.area.width.saturating_sub(self.metrics.row_num_width))
            / (self.metrics.col_width + 1)) as usize;
        VisibleCapacity { rows, cols }
    }

    pub fn table_column_widths(&self, rendered_cols: usize) -> Vec<u16> {
        let mut widths = Vec::with_capacity(rendered_cols + 1);
        widths.push(self.metrics.row_num_width);
        for _ in 0..rendered_cols {
            widths.push(self.metrics.col_width);
        }
        widths
    }

    pub fn cell_rect(&self, vis_row: usize, vis_col: usize) -> Option<CellRect> {
        let left = self.col_grid_x(vis_col);
        let right = self.col_grid_x(vis_col + 1);
        let top = self.area.y + 1 + vis_row as u16 * self.metrics.rows_per_cell;
        if left >= right || right > self.area.right() || {
            let bottom = top + self.metrics.rows_per_cell;
            bottom > self.area.bottom()
        } {
            return None;
        }
        let content = top + 1;
        let bottom = top + self.metrics.rows_per_cell;
        Some(CellRect {
            left_x: left,
            right_x: right,
            top_y: top,
            content_y: content,
            bottom_y: bottom,
        })
    }

    pub fn region_rect(
        &self,
        vis_rows: RangeInclusive<usize>,
        vis_cols: RangeInclusive<usize>,
    ) -> Option<RegionRect> {
        let vis_r1 = *vis_rows.start();
        let vis_r2 = *vis_rows.end();
        let vis_c1 = *vis_cols.start();
        let vis_c2 = *vis_cols.end();

        let left = self.col_grid_x(vis_c1);
        let right = self.col_grid_x(vis_c2 + 1);
        let top = self.area.y + 1 + vis_r1 as u16 * self.metrics.rows_per_cell;
        let bottom =
            self.area.y + 1 + vis_r2 as u16 * self.metrics.rows_per_cell + self.metrics.rows_per_cell;

        if left >= right || right > self.area.right() || bottom > self.area.bottom() {
            return None;
        }
        Some(RegionRect {
            left_x: left,
            right_x: right,
            top_y: top,
            bottom_y: bottom,
        })
    }

    pub fn row_region_rect(&self, vis_row: usize, visible_cols: usize) -> Option<RegionRect> {
        let left = self.col_grid_x(0);
        let right = self.col_grid_x(visible_cols).min(self.area.right());
        let top = self.area.y + 1 + vis_row as u16 * self.metrics.rows_per_cell;
        let bottom = top + self.metrics.rows_per_cell;

        if bottom > self.area.bottom() {
            return None;
        }
        Some(RegionRect {
            left_x: left,
            right_x: right,
            top_y: top,
            bottom_y: bottom,
        })
    }

    pub fn col_region_rect(&self, vis_col: usize, visible_rows: usize) -> Option<RegionRect> {
        let left = self.col_grid_x(vis_col);
        let right = self.col_grid_x(vis_col + 1);
        let top = self.area.y;
        let bottom = (self.area.y + 1 + visible_rows as u16 * self.metrics.rows_per_cell)
            .min(self.area.bottom());

        if left >= right || right > self.area.right() {
            return None;
        }
        Some(RegionRect {
            left_x: left,
            right_x: right,
            top_y: top,
            bottom_y: bottom,
        })
    }

    pub fn horizontal_grid_ys(&self, visible_rows: usize) -> Vec<u16> {
        (0..visible_rows)
            .map(|r| self.area.y + 1 + r as u16 * self.metrics.rows_per_cell)
            .collect()
    }

    pub fn vertical_grid_xs(&self, visible_cols: usize) -> Vec<u16> {
        (0..=visible_cols)
            .map(|c| self.col_grid_x(c))
            .filter(|&x| x < self.area.right())
            .collect()
    }

    fn col_grid_x(&self, col_index: usize) -> u16 {
        self.area.x + self.metrics.row_num_width + col_index as u16 * (self.metrics.col_width + 1)
    }
}
