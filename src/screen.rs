//! Screen 抽象层。
//!
//! 定义画面切换的接口。每个画面实现自己的渲染与按键处理。

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

/// 按键处理后的下一步动作。
#[allow(dead_code)]
pub enum ScreenCommand {
    Stay,
    Navigate(Box<dyn Screen>),
}

/// 画面接口：渲染与按键处理。
///
/// 每个画面封装自己的布局、子组件、按键响应。
pub trait Screen {
    /// 在 `area` 区域内渲染当前画面。
    fn render(&self, frame: &mut Frame, area: Rect);

    /// 处理按键。
    ///
    /// 返回 `None` 表示该按键未被处理（App 不重置退出确认），
    /// 返回 `Some(ScreenCommand)` 表示已处理（App 重置退出确认并执行命令）。
    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand>;
}
