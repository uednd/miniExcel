#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleCapacity {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRect {
    pub left_x: u16,
    pub right_x: u16,
    pub top_y: u16,
    pub content_y: u16,
    pub bottom_y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionRect {
    pub left_x: u16,
    pub right_x: u16,
    pub top_y: u16,
    pub bottom_y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStroke {
    Solid,
    Dashed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BorderGlyphs {
    pub horizontal: &'static str,
    pub vertical: &'static str,
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
}

impl BorderStroke {
    pub fn glyphs(self) -> BorderGlyphs {
        match self {
            BorderStroke::Solid => BorderGlyphs {
                horizontal: "━",
                vertical: "┃",
                top_left: "┏",
                top_right: "┓",
                bottom_left: "┗",
                bottom_right: "┛",
            },
            BorderStroke::Dashed => BorderGlyphs {
                horizontal: "╌",
                vertical: "╎",
                top_left: "┏",
                top_right: "┓",
                bottom_left: "┗",
                bottom_right: "┛",
            },
        }
    }
}
