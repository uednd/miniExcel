use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, text::Line};

use super::context::TableContext;
use crate::{model::cell::CellAddress, screen::ScreenCommand};

/// 模式处理按键后的动作。
pub enum ModeAction {
    Nothing,
    SwitchToEdit { initial_char: Option<char> },
    SwitchToNavigation,
    ScreenCommand(ScreenCommand),
}

/// 选区类型：行选中 / 列选中 / 矩形区域（预留 Shift+方向键）。
pub enum Selection {
    Row(usize),
    Column(usize),
    Range { anchor: CellAddress, cursor: CellAddress },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    Navigation,
    Edit,
    Menu,
    Delete,
}

pub trait Mode {
    fn kind(&self) -> ModeKind;

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction;

    /// 渲染模式专属内容（侧面板、编辑栏等），返回留给表格区域的 Rect。
    fn render_frame(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect;

    fn footer_hint(&self, ctx: &TableContext) -> Option<Line<'static>> {
        let _ = ctx;
        None
    }

    fn footer_status(&self, ctx: &TableContext) -> Option<Line<'static>> {
        let _ = ctx;
        None
    }

    fn edit_buffer(&self) -> Option<&str> {
        None
    }
}
