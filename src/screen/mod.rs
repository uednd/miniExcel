pub mod editor;
pub mod home;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, text::Line};

/// 事件处理结果。
pub enum EventResult<T> {
    /// 当前模块未处理事件。
    Ignored,
    /// 当前模块已处理事件，但没有产生命令。
    Handled,
    /// 当前模块已处理事件，并产生一个命令。
    Command(T),
}

/// 屏幕返回给应用主循环的命令。
pub enum ScreenCommand {
    /// 打开指定路径的表格编辑器。
    OpenEditor { path: String },
    /// 返回首页。
    GoHome,
}

/// 每帧渲染前的状态快照。
#[derive(Debug, Clone, Copy)]
pub struct FrameState {
    pub blink_visible: bool,
}

/// 可被应用主循环驱动的屏幕。
pub trait Screen {
    /// 渲染前的回调，用于推送帧状态（如闪烁相位）到屏幕内部。
    fn pre_render(&mut self, _state: FrameState) {}

    /// 渲染屏幕内容。
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// 处理键盘事件。
    ///
    /// 返回 `Ignored` 表示该屏幕未处理事件；返回 `Handled` 表示已处理但不切换屏幕。
    fn handle_key(&mut self, key: KeyEvent) -> EventResult<ScreenCommand>;

    /// 处理鼠标滚动事件。
    fn handle_scroll(&mut self, _event: MouseEvent) -> EventResult<ScreenCommand> {
        EventResult::Ignored
    }

    /// 页脚中的快捷键提示。
    fn footer_hint(&self) -> Option<Line<'static>> {
        None
    }

    /// 页脚中的当前位置或模式状态。
    fn footer_status(&self) -> Option<Line<'static>> {
        None
    }
}
