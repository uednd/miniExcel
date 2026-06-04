//! Screen 抽象层。
//!
//! 定义画面切换的接口。每个画面实现自己的渲染与按键处理。

use crossterm::event::KeyEvent;
use ratatui::Frame;

/// 按键处理后的下一步动作。
#[allow(dead_code)]
pub enum ScreenCommand {
    /// 停留在当前画面。
    Stay,
    /// 切换到新画面。
    Navigate(Box<dyn Screen>),
}

/// 画面接口：渲染与按键处理。
///
/// 每个画面封装自己的布局、子组件、按键响应。
/// App 只需持有当前 Screen 并委托，不感知内部结构。
pub trait Screen {
    /// 渲染当前画面。
    ///
    /// `hint` 为应用级提示文字（如退出确认文案），
    /// 画面自行决定是否以及何处渲染。
    fn render(&self, frame: &mut Frame, hint: Option<&str>);

    /// 处理按键。
    ///
    /// 返回 `None` 表示该按键未被处理（App 不重置退出确认），
    /// 返回 `Some(ScreenCommand)` 表示已处理（App 重置退出确认并执行命令）。
    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand>;
}
