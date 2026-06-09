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

/// 编辑器模式宿主。
///
/// 宿主先处理跨模式快捷键，再把按键交给当前模式。
/// 当前模式返回 `Handled` 时，宿主会向外层返回 `ScreenCommand::Stay`，
/// 让应用主循环知道该事件已经被消费。
pub struct ModeHost {
    mode: Box<dyn Mode>,
}

impl ModeHost {
    /// 使用初始模式创建宿主。
    pub fn new(mode: Box<dyn Mode>) -> Self {
        Self { mode }
    }

    /// 返回当前模式种类。
    #[allow(dead_code)]
    pub fn mode_kind(&self) -> ModeKind {
        self.mode.kind()
    }

    /// 处理按键并转换为屏幕命令。
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

    /// 渲染当前模式专属区域，并返回剩余表格区域。
    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        self.mode.render(frame, area, ctx)
    }

    /// 返回当前编辑缓冲；只有编辑模式会返回 `Some`。
    pub fn edit_buffer(&self) -> Option<&str> {
        self.mode.edit_buffer()
    }

    /// 返回当前模式页脚。
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
