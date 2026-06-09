use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, text::Line};

use super::context::TableContext;
use crate::model::cell::CellAddress;

/// 模式处理按键后的结果。
pub enum ModeAction {
    Handled,
    #[allow(dead_code)]
    Unhandled,
    SwitchMode(Box<dyn Mode>),
}

/// 选区类型：行选中 / 列选中 / 矩形区域（预留 Shift+方向键）。
pub enum Selection {
    Row(usize),
    Column(usize),
    Range {
        anchor: CellAddress,
        cursor: CellAddress,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    Navigation,
    Edit,
    Menu,
    Delete,
}

/// 页脚信息
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
    fn kind(&self) -> ModeKind;

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction;

    /// 渲染模式专属内容（侧面板、编辑栏等），返回留给表格区域的 Rect。
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect;

    fn footer(&self, ctx: &TableContext) -> FooterLine {
        let _ = ctx;
        FooterLine::none()
    }

    fn edit_buffer(&self) -> Option<&str> {
        None
    }
}
