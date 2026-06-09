use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use crate::screen::ScreenCommand;

use super::{
    context::TableContext,
    delete::DeleteMode,
    menu::MenuMode,
    mode::{FooterLine, Mode, ModeAction, ModeKind},
    navigation::NavigationMode,
};

/// 模式生命周期协调器：快捷键拦截、模式切换、委托渲染。
pub struct ModeHost {
    mode: Box<dyn Mode>,
}

impl ModeHost {
    pub fn new(mode: Box<dyn Mode>) -> Self {
        Self { mode }
    }

    #[allow(dead_code)]
    pub fn mode_kind(&self) -> ModeKind {
        self.mode.kind()
    }

    pub fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> Option<ScreenCommand> {
        if let Some(cmd) = self.intercept_shortcut(ctx, key) {
            return Some(cmd);
        }
        match self.mode.handle_key(ctx, key) {
            ModeAction::Handled => ctx.take_pending_command().or(Some(ScreenCommand::Stay)),
            ModeAction::Unhandled => None,
            ModeAction::SwitchMode(new_mode) => {
                self.mode = new_mode;
                Some(ScreenCommand::Stay)
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        self.mode.render(frame, area, ctx)
    }

    pub fn edit_buffer(&self) -> Option<&str> {
        self.mode.edit_buffer()
    }

    pub fn footer(&self, ctx: &TableContext) -> FooterLine {
        self.mode.footer(ctx)
    }

    fn intercept_shortcut(
        &mut self,
        ctx: &mut TableContext,
        key: KeyEvent,
    ) -> Option<ScreenCommand> {
        if Self::is_ctrl_s(key)
            && self.mode.kind() != ModeKind::Menu
            && self.mode.kind() != ModeKind::Delete
        {
            ctx.save();
            return Some(ScreenCommand::Stay);
        }
        if Self::is_ctrl_p(key) {
            self.mode = match self.mode.kind() {
                ModeKind::Menu | ModeKind::Delete => Box::new(NavigationMode),
                _ => Box::new(MenuMode::new()),
            };
            return Some(ScreenCommand::Stay);
        }
        if Self::is_ctrl_d(key)
            && self.mode.kind() != ModeKind::Menu
            && self.mode.kind() != ModeKind::Delete
        {
            self.mode = Box::new(DeleteMode::new());
            return Some(ScreenCommand::Stay);
        }
        None
    }

    fn is_ctrl_s(key: KeyEvent) -> bool {
        key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_ctrl_p(key: KeyEvent) -> bool {
        key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_ctrl_d(key: KeyEvent) -> bool {
        key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL)
    }
}
