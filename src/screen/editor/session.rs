use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect, style::Style, text::Line};

use crate::{
    model::{document::WorkbookDocument, workbook::Workbook},
    screen::{EventResult, FrameState, ScreenCommand},
    theme::Theme,
};

use super::{
    delete::DeleteMode,
    menu::MenuMode,
    mode::{EditorIntent, EditorReadModel, EditorView, Mode, ModeKind, Selection},
    navigation::NavigationMode,
    state::EditorState,
    viewport::Viewport,
    workbook_controller::WorkbookController,
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
    theme: Theme,
    state: EditorState,
    workbook: WorkbookController,
    mode: Box<dyn Mode>,
}

impl EditorSession {
    pub fn new(theme: Theme, document: WorkbookDocument) -> Self {
        Self {
            theme,
            state: EditorState::new(),
            workbook: WorkbookController::new(document),
            mode: Box::new(NavigationMode),
        }
    }

    pub fn pre_render(&mut self, state: FrameState) {
        self.state.set_blink_visible(state.blink_visible);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EventResult<ScreenCommand> {
        if let Some(result) = self.intercept_shortcut(key) {
            return result;
        }

        match self
            .mode
            .handle_key(EditorView::new(self.state.selection().copied()), key)
        {
            EventResult::Handled => EventResult::Handled,
            EventResult::Ignored => EventResult::Ignored,
            EventResult::Command(intent) => self.apply_intent(intent),
        }
    }

    pub fn handle_scroll(&mut self, event: MouseEvent) -> EventResult<ScreenCommand> {
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.state.scroll_up(3);
                EventResult::Handled
            }
            MouseEventKind::ScrollDown => {
                self.state.scroll_down(3, self.workbook.row_count());
                EventResult::Handled
            }
            MouseEventKind::ScrollLeft => {
                self.state.scroll_left(1);
                EventResult::Handled
            }
            MouseEventKind::ScrollRight => {
                self.state.scroll_right(1, self.workbook.column_count());
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    pub fn render_mode(&self, frame: &mut Frame, area: Rect) -> Rect {
        self.mode.render(frame, area, self.read_model())
    }

    pub fn table_parts(&self) -> EditorTableParts<'_> {
        EditorTableParts {
            workbook: self.workbook.workbook(),
            viewport: self.state.viewport(),
            theme: self.theme,
            blink_visible: self.state.blink_visible(),
            edit_buffer: self.mode.edit_buffer(),
            selection: self.state.selection(),
            copied_region: self.state.copied_region(),
        }
    }

    pub fn update_visible_capacity(&mut self, rows: usize, cols: usize) {
        self.state.update_visible_capacity(rows, cols);
    }

    pub fn workbook_name(&self) -> &str {
        self.workbook.workbook_name()
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn footer_hint(&self) -> Option<Line<'static>> {
        self.mode.footer(self.read_model()).hint
    }

    pub fn footer_status(&self) -> Option<Line<'static>> {
        if let Some(message) = self.state.status_message() {
            return Some(Line::styled(
                message.to_string(),
                Style::default().fg(self.theme.accent),
            ));
        }
        self.mode.footer(self.read_model()).status
    }

    fn intercept_shortcut(&mut self, key: KeyEvent) -> Option<EventResult<ScreenCommand>> {
        if Self::is_ctrl_s(key)
            && self.mode.kind() != ModeKind::Menu
            && self.mode.kind() != ModeKind::Delete
        {
            self.commit_visible_edit_buffer();
            self.save_workbook();
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
                self.state.move_cursor(
                    direction,
                    self.workbook.row_count(),
                    self.workbook.column_count(),
                );
                EventResult::Handled
            }
            EditorIntent::MoveCursorAndClearSelection(direction) => {
                self.state.move_cursor_and_clear_selection(
                    direction,
                    self.workbook.row_count(),
                    self.workbook.column_count(),
                );
                EventResult::Handled
            }
            EditorIntent::StartEdit { initial_char } => {
                let existing = self.workbook.current_cell_raw(self.state.cursor());
                self.mode = Box::new(super::edit::EditMode::new(existing, initial_char));
                EventResult::Handled
            }
            EditorIntent::CommitEdit(raw) => {
                self.workbook.commit_cell(self.state.cursor(), raw);
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
            EditorIntent::ClearCurrentCell => {
                self.workbook.clear_cell(self.state.cursor());
                EventResult::Handled
            }
            EditorIntent::ClearSelectionCells => {
                if let Some(selection) = self.state.selection().copied() {
                    self.workbook.clear_selection(selection);
                    self.state.clear_selection();
                }
                EventResult::Handled
            }
            EditorIntent::ClearSelection => {
                self.state.clear_selection();
                EventResult::Handled
            }
            EditorIntent::StartRangeSelection(direction) => {
                self.state.start_range_selection(
                    direction,
                    self.workbook.row_count(),
                    self.workbook.column_count(),
                );
                EventResult::Handled
            }
            EditorIntent::ExtendRangeSelection(direction) => {
                self.state.extend_range_selection(
                    direction,
                    self.workbook.row_count(),
                    self.workbook.column_count(),
                );
                EventResult::Handled
            }
            EditorIntent::SelectCurrentRow => {
                self.state.select_current_row();
                EventResult::Handled
            }
            EditorIntent::SelectCurrentColumn => {
                self.state.select_current_column();
                EventResult::Handled
            }
            EditorIntent::Copy => {
                let selection = self.state.selection().copied();
                let cells = self
                    .workbook
                    .collect_selection(selection, self.state.cursor());
                let tsv = crate::clipboard::to_tsv(&cells);
                if let Err(err) = crate::clipboard::copy_to_clipboard(&tsv) {
                    self.state.set_status_message(err);
                } else if let Some(selection) = selection {
                    self.state.set_copied_region(selection);
                } else {
                    let cursor = self.state.cursor();
                    self.state.set_copied_region(Selection::Range {
                        anchor: cursor,
                        cursor,
                    });
                }
                EventResult::Handled
            }
            EditorIntent::Paste => {
                match crate::clipboard::read_from_clipboard() {
                    Ok(text) => {
                        let rows = crate::clipboard::from_tsv(&text);
                        if let Some(selection) =
                            self.workbook.paste_range(self.state.cursor(), &rows)
                        {
                            self.state.clear_copied_region();
                            self.state.set_selection(selection);
                        }
                    }
                    Err(err) => self.state.set_status_message(err),
                }
                EventResult::Handled
            }
            EditorIntent::Save => {
                self.commit_visible_edit_buffer();
                self.save_workbook();
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
            EditorIntent::SaveAndGoHome => {
                self.commit_visible_edit_buffer();
                if self.save_workbook() {
                    EventResult::Command(ScreenCommand::GoHome)
                } else {
                    EventResult::Handled
                }
            }
            EditorIntent::GoHome => EventResult::Command(ScreenCommand::GoHome),
            EditorIntent::DeleteCurrentRow => {
                self.workbook.delete_row(self.state.cursor().row);
                self.state.clamp_cursor_row(self.workbook.row_count());
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
            EditorIntent::DeleteCurrentColumn => {
                self.workbook.delete_column(self.state.cursor().col);
                self.state.clamp_cursor_col(self.workbook.column_count());
                self.mode = Box::new(NavigationMode);
                EventResult::Handled
            }
        }
    }

    fn commit_visible_edit_buffer(&mut self) {
        if let Some(raw) = self.mode.edit_buffer().map(str::to_owned) {
            self.workbook.commit_cell(self.state.cursor(), raw);
        }
    }

    fn save_workbook(&mut self) -> bool {
        match self.workbook.save() {
            Ok(()) => {
                self.state.set_status_message("已保存");
                true
            }
            Err(err) => {
                self.state.set_status_message(err.message());
                false
            }
        }
    }

    fn read_model(&self) -> EditorReadModel<'_> {
        let selection = self.state.selection().copied();
        EditorReadModel {
            theme: self.theme,
            viewport: self.state.viewport(),
            blink_visible: self.state.blink_visible(),
            selection_stats: selection
                .and_then(|selection| self.workbook.selection_stats(selection)),
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
