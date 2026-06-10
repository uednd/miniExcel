use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect, style::Style, text::Line};

use crate::{
    model::{document::WorkbookDocument, workbook::Workbook},
    screen::{EventResult, FrameState, ScreenCommand},
    theme::Theme,
};

use super::{
    context::TableContext,
    delete::DeleteMode,
    menu::MenuMode,
    mode::{EditorIntent, EditorView, Mode, ModeKind, Selection},
    navigation::NavigationMode,
    viewport::Viewport,
};

/// 表格渲染需要的只读状态。
pub struct EditorTableParts<'a> {
    pub workbook: &'a Workbook,
    pub viewport: &'a Viewport,
    pub theme: Theme,
    pub blink_visible: bool,
    pub edit_buffer: Option<&'a str>,
    pub selection: Option<&'a Selection>,
    pub copied_region: Option<&'a Selection>,
}

/// 编辑器会话。
///
/// 会话是编辑器唯一的交互入口：模式只返回意图，保存、提交编辑、
/// 屏幕命令和跨模式快捷键都在这里统一应用。
pub struct EditorSession {
    ctx: TableContext,
    mode: Box<dyn Mode>,
}

impl EditorSession {
    pub fn new(theme: Theme, document: WorkbookDocument) -> Self {
        Self {
            ctx: TableContext::new(theme, document),
            mode: Box::new(NavigationMode),
        }
    }

    pub fn pre_render(&mut self, state: FrameState) {
        self.ctx.set_blink_visible(state.blink_visible);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EventResult<ScreenCommand> {
        if let Some(result) = self.intercept_shortcut(key) {
            return result;
        }

        match self.mode.handle_key(EditorView::new(&self.ctx), key) {
            EventResult::Handled => EventResult::Handled,
            EventResult::Ignored => EventResult::Ignored,
            EventResult::Command(intent) => self.apply_intent(intent),
        }
    }

    pub fn handle_scroll(&mut self, event: MouseEvent) -> EventResult<ScreenCommand> {
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.ctx.scroll_up(3);
                EventResult::Handled
            }
            MouseEventKind::ScrollDown => {
                self.ctx.scroll_down(3);
                EventResult::Handled
            }
            MouseEventKind::ScrollLeft => {
                self.ctx.scroll_left(1);
                EventResult::Handled
            }
            MouseEventKind::ScrollRight => {
                self.ctx.scroll_right(1);
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    pub fn render_mode(&self, frame: &mut Frame, area: Rect) -> Rect {
        self.mode.render(frame, area, &self.ctx)
    }

    pub fn table_parts(&self) -> EditorTableParts<'_> {
        EditorTableParts {
            workbook: self.ctx.workbook(),
            viewport: self.ctx.viewport(),
            theme: self.ctx.theme,
            blink_visible: self.ctx.blink_visible(),
            edit_buffer: self.mode.edit_buffer(),
            selection: self.ctx.selection(),
            copied_region: self.ctx.copied_region(),
        }
    }

    pub fn update_visible_capacity(&mut self, rows: usize, cols: usize) {
        self.ctx.update_visible_capacity(rows, cols);
    }

    pub fn workbook_name(&self) -> &str {
        self.ctx.workbook_name()
    }

    pub fn theme(&self) -> Theme {
        self.ctx.theme
    }

    pub fn footer_hint(&self) -> Option<Line<'static>> {
        self.mode.footer(&self.ctx).hint
    }

    pub fn footer_status(&self) -> Option<Line<'static>> {
        if let Some(message) = self.ctx.status_message() {
            return Some(Line::styled(
                message.to_string(),
                Style::default().fg(self.ctx.theme.accent),
            ));
        }
        self.mode.footer(&self.ctx).status
    }

    fn intercept_shortcut(&mut self, key: KeyEvent) -> Option<EventResult<ScreenCommand>> {
        if Self::is_ctrl_s(key)
            && self.mode.kind() != ModeKind::Menu
            && self.mode.kind() != ModeKind::Delete
        {
            self.commit_visible_edit_buffer();
            self.ctx.save();
            return Some(EventResult::Handled);
        }
        if Self::is_ctrl_p(key) {
            self.mode = match self.mode.kind() {
                ModeKind::Menu | ModeKind::Delete => Box::new(NavigationMode),
                _ => Box::new(MenuMode::new()),
            };
            return Some(EventResult::Handled);
        }
        if Self::is_ctrl_d(key)
            && self.mode.kind() != ModeKind::Menu
            && self.mode.kind() != ModeKind::Delete
        {
            self.mode = Box::new(DeleteMode::new());
            return Some(EventResult::Handled);
        }
        if Self::is_ctrl_c(key) && self.mode.kind() == ModeKind::Navigation {
            return Some(self.apply_intent(EditorIntent::Copy));
        }
        if Self::is_ctrl_v(key) && self.mode.kind() == ModeKind::Navigation {
            return Some(self.apply_intent(EditorIntent::Paste));
        }
        None
    }

    fn apply_intent(&mut self, intent: EditorIntent) -> EventResult<ScreenCommand> {
        match intent {
            EditorIntent::SwitchMode(new_mode) => {
                self.mode = new_mode;
                EventResult::Handled
            }
            EditorIntent::MoveCursor(direction) => {
                self.ctx.move_cursor(direction);
                EventResult::Handled
            }
            EditorIntent::MoveCursorAndClearSelection(direction) => {
                self.ctx.move_cursor_and_clear_selection(direction);
                EventResult::Handled
            }
            EditorIntent::StartEdit { initial_char } => {
                let existing = self.ctx.current_cell_raw();
                self.mode = Box::new(super::edit::EditMode::new(existing, initial_char));
                EventResult::Handled
            }
            EditorIntent::CommitEdit(raw) => {
                self.ctx.commit_current_cell(raw);
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
            EditorIntent::ClearCurrentCell => {
                self.ctx.clear_current_cell();
                EventResult::Handled
            }
            EditorIntent::ClearSelectionCells => {
                self.ctx.clear_selection_cells();
                EventResult::Handled
            }
            EditorIntent::ClearSelection => {
                self.ctx.clear_selection();
                EventResult::Handled
            }
            EditorIntent::StartRangeSelection(direction) => {
                self.ctx.start_range_selection(direction);
                EventResult::Handled
            }
            EditorIntent::ExtendRangeSelection(direction) => {
                self.ctx.extend_range_selection(direction);
                EventResult::Handled
            }
            EditorIntent::SelectCurrentRow => {
                self.ctx.select_current_row();
                EventResult::Handled
            }
            EditorIntent::SelectCurrentColumn => {
                self.ctx.select_current_column();
                EventResult::Handled
            }
            EditorIntent::Copy => {
                if let Err(err) = self.ctx.copy_selection() {
                    self.ctx.set_status_message(err);
                }
                EventResult::Handled
            }
            EditorIntent::Paste => {
                if let Err(err) = self.ctx.paste_from_clipboard() {
                    self.ctx.set_status_message(err);
                }
                EventResult::Handled
            }
            EditorIntent::Save => {
                self.commit_visible_edit_buffer();
                self.ctx.save();
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
            EditorIntent::SaveAndGoHome => {
                self.commit_visible_edit_buffer();
                if self.ctx.save() {
                    EventResult::Command(ScreenCommand::GoHome)
                } else {
                    EventResult::Handled
                }
            }
            EditorIntent::GoHome => EventResult::Command(ScreenCommand::GoHome),
            EditorIntent::DeleteCurrentRow => {
                self.ctx.delete_current_row();
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
            EditorIntent::DeleteCurrentColumn => {
                self.ctx.delete_current_column();
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
        }
    }

    fn commit_visible_edit_buffer(&mut self) {
        if let Some(raw) = self.mode.edit_buffer().map(str::to_owned) {
            self.ctx.commit_current_cell(raw);
        }
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

    fn is_ctrl_c(key: KeyEvent) -> bool {
        key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
    }

    fn is_ctrl_v(key: KeyEvent) -> bool {
        key.code == KeyCode::Char('v') && key.modifiers.contains(KeyModifiers::CONTROL)
    }
}
