use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
};

use super::primitives::{BorderStroke, CellRect, RegionRect};

pub fn draw_cell_border(buffer: &mut Buffer, cell_rect: CellRect, style: Style) {
    let CellRect {
        left_x: left,
        right_x: right,
        top_y: top,
        content_y: content,
        bottom_y: bottom,
    } = cell_rect;

    buffer[(left, top)].set_symbol("╆").set_style(style);
    buffer[(right, top)].set_symbol("╅").set_style(style);
    buffer[(left, bottom)].set_symbol("╄").set_style(style);
    buffer[(right, bottom)].set_symbol("╃").set_style(style);

    for x in left + 1..right {
        buffer[(x, top)].set_symbol("━").set_style(style);
        buffer[(x, bottom)].set_symbol("━").set_style(style);
    }
    buffer[(left, content)].set_symbol("┃").set_style(style);
    buffer[(right, content)].set_symbol("┃").set_style(style);
}

pub fn draw_region_border(
    buffer: &mut Buffer,
    region: RegionRect,
    style: Style,
    stroke: BorderStroke,
) {
    let glyphs = stroke.glyphs();
    let RegionRect {
        left_x: left,
        right_x: right,
        top_y: top,
        bottom_y: bottom,
    } = region;

    buffer[(left, top)].set_symbol(glyphs.top_left).set_style(style);
    buffer[(right, top)]
        .set_symbol(glyphs.top_right)
        .set_style(style);
    buffer[(left, bottom)]
        .set_symbol(glyphs.bottom_left)
        .set_style(style);
    buffer[(right, bottom)]
        .set_symbol(glyphs.bottom_right)
        .set_style(style);

    for x in left + 1..right {
        buffer[(x, top)].set_symbol(glyphs.horizontal).set_style(style);
        buffer[(x, bottom)]
            .set_symbol(glyphs.horizontal)
            .set_style(style);
    }
    for y in top + 1..bottom {
        buffer[(left, y)].set_symbol(glyphs.vertical).set_style(style);
        buffer[(right, y)]
            .set_symbol(glyphs.vertical)
            .set_style(style);
    }
}

pub fn draw_grid(
    buffer: &mut Buffer,
    area: Rect,
    horizontal_ys: &[u16],
    vertical_xs: &[u16],
    style: Style,
) {
    for &y in horizontal_ys {
        for x in area.x..area.right() {
            buffer[(x, y)].set_symbol("─").set_style(style);
        }
    }

    for &x in vertical_xs {
        for y in area.y..area.bottom() {
            let is_grid_y = horizontal_ys.binary_search(&y).is_ok();
            let sym = if y == area.y {
                "│"
            } else if is_grid_y {
                "┼"
            } else {
                "│"
            };
            buffer[(x, y)].set_symbol(sym).set_style(style);
        }
    }
}
