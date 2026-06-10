use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, text::Line};

use super::{viewport::Viewport, workbook_controller::SelectionStats};
use crate::{model::cell::CellAddress, screen::EventResult, theme::Theme};

/// 编辑器方向键方向。
#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// 模式处理按键时可读取的编辑器快照。
///
/// `EditorView` 只暴露只读信息，避免模式直接修改编辑器状态。
pub struct EditorView<'a> {
    selection: Option<Selection>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> EditorView<'a> {
    pub fn new(selection: Option<Selection>) -> Self {
        Self {
            selection,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn selection(&self) -> Option<Selection> {
        self.selection
    }
}

/// 模式渲染和 footer 需要的只读模型。
pub struct EditorReadModel<'a> {
    pub theme: Theme,
    pub viewport: &'a Viewport,
    pub blink_visible: bool,
    pub selection_stats: Option<SelectionStats>,
}

/// 编辑器模式产生的意图。
///
/// 模式只描述“用户想做什么”，具体如何修改工作簿、保存或切换屏幕，
/// 由 `EditorSession` 统一应用。
pub enum EditorIntent {
    /// 切换到另一个编辑器模式。
    SwitchMode(Box<dyn Mode>),
    /// 移动光标。
    MoveCursor(Direction),
    /// 清除选区后移动光标。
    MoveCursorAndClearSelection(Direction),
    /// 进入编辑模式。
    StartEdit { initial_char: Option<char> },
    /// 提交当前单元格的编辑文本。
    CommitEdit(String),
    /// 清除当前单元格。
    ClearCurrentCell,
    /// 清除当前选区内的单元格。
    ClearSelectionCells,
    /// 清除当前选区。
    ClearSelection,
    /// 从当前光标开始创建范围选区。
    StartRangeSelection(Direction),
    /// 扩展现有范围选区。
    ExtendRangeSelection(Direction),
    /// 选中当前行。
    SelectCurrentRow,
    /// 选中当前列。
    SelectCurrentColumn,
    /// 复制当前选区或当前单元格。
    Copy,
    /// 从剪贴板粘贴。
    Paste,
    /// 保存当前工作簿。
    Save,
    /// 保存成功后返回首页。
    SaveAndGoHome,
    /// 不保存，直接返回首页。
    GoHome,
    /// 删除当前行。
    DeleteCurrentRow,
    /// 删除当前列。
    DeleteCurrentColumn,
}

/// 编辑器模式处理按键后的结果。
pub type ModeResult = EventResult<EditorIntent>;

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
    fn handle_key(&mut self, view: EditorView<'_>, key: KeyEvent) -> ModeResult;

    /// 渲染模式专属内容，并返回留给表格区域的区域。
    ///
    /// 例如菜单模式会占用右侧面板，编辑模式会占用底部输入行。
    fn render(&self, frame: &mut Frame, area: Rect, read: EditorReadModel<'_>) -> Rect;

    /// 返回当前模式的页脚文本。
    fn footer(&self, read: EditorReadModel<'_>) -> FooterLine {
        let _ = read;
        FooterLine::none()
    }

    /// 返回正在编辑的文本；非编辑模式返回 `None`。
    fn edit_buffer(&self) -> Option<&str> {
        None
    }
}
