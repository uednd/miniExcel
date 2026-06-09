pub mod editor;
pub mod home;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, text::Line};

/// 屏幕返回给应用主循环的命令。
pub enum ScreenCommand {
    /// 事件已处理，继续停留在当前屏幕。
    Stay,
    /// 打开指定路径的表格编辑器。
    OpenEditor { path: String },
    /// 返回首页。
    GoHome,
}

/// 可被应用主循环驱动的屏幕。
pub trait Screen {
    /// 渲染屏幕内容。
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// 处理键盘事件。
    ///
    /// 返回 `None` 表示该屏幕未处理事件；返回 `Some(Stay)` 表示已处理但不切换屏幕。
    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand>;

    /// 处理鼠标滚动事件。
    fn handle_scroll(&mut self, _event: MouseEvent) -> Option<ScreenCommand> {
        None
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
