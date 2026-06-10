use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, text::Line};

use super::context::TableContext;
use crate::model::cell::CellAddress;
use crate::screen::EventResult;

/// 编辑器模式产生的命令。
pub enum ModeCommand {
    /// 切换到另一个编辑器模式。
    SwitchMode(Box<dyn Mode>),
}

/// 编辑器模式处理按键后的结果。
pub type ModeResult = EventResult<ModeCommand>;

/// 表格选区。
///
/// 行、列索引均从 0 开始。矩形选区保存锚点和当前光标。
#[derive(Clone, Copy)]
pub enum Selection {
    Row(usize),
    Column(usize),
    Range {
        anchor: CellAddress,
        cursor: CellAddress,
    },
}

impl Selection {
    /// 判断给定地址是否在选区内。
    pub fn contains(&self, addr: CellAddress) -> bool {
        match *self {
            Selection::Row(r) => r == addr.row,
            Selection::Column(c) => c == addr.col,
            Selection::Range { anchor, cursor } => {
                let (r1, r2, c1, c2) = Self::normalized(anchor, cursor);
                addr.row >= r1 && addr.row <= r2 && addr.col >= c1 && addr.col <= c2
            }
        }
    }

    /// 返回选区的归一化边界 `(min_row, max_row, min_col, max_col)`。
    ///
    /// Row / Column 选区返回该行/列在所有行/列上的范围。
    pub fn normalized_bounds(&self, rows: usize, cols: usize) -> (usize, usize, usize, usize) {
        match *self {
            Selection::Row(r) => (r, r, 0, cols.saturating_sub(1)),
            Selection::Column(c) => (0, rows.saturating_sub(1), c, c),
            Selection::Range { anchor, cursor } => Self::normalized(anchor, cursor),
        }
    }

    pub(crate) fn normalized(
        anchor: CellAddress,
        cursor: CellAddress,
    ) -> (usize, usize, usize, usize) {
        (
            anchor.row.min(cursor.row),
            anchor.row.max(cursor.row),
            anchor.col.min(cursor.col),
            anchor.col.max(cursor.col),
        )
    }
}

/// 编辑器模式种类。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    Navigation,
    Edit,
    Menu,
    Delete,
}

/// 当前模式想显示在页脚中的提示和状态文本。
pub struct FooterLine {
    pub hint: Option<Line<'static>>,
    pub status: Option<Line<'static>>,
}

impl FooterLine {
    pub fn none() -> Self {
        Self {
            hint: None,
            status: None,
        }
    }
}

pub trait Mode {
    /// 返回当前模式种类，供宿主判断快捷键策略。
    fn kind(&self) -> ModeKind;

    /// 处理一个按键事件。
    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeResult;

    /// 渲染模式专属内容，并返回留给表格区域的区域。
    ///
    /// 例如菜单模式会占用右侧面板，编辑模式会占用底部输入行。
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect;

    /// 返回当前模式的页脚文本。
    fn footer(&self, ctx: &TableContext) -> FooterLine {
        let _ = ctx;
        FooterLine::none()
    }

    /// 返回正在编辑的文本；非编辑模式返回 `None`。
    fn edit_buffer(&self) -> Option<&str> {
        None
    }
}
