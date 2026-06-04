//! App 状态管理与事件循环。
//!
//! 包含主事件循环、键盘事件分发。渲染与画面逻辑委托给 Screen trait 实现。

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use crate::{
    exit_handler::ExitHandler,
    menu_screen::MenuScreen,
    screen::{Screen, ScreenCommand},
};

/// 应用全局状态。
pub struct App {
    active_screen: Box<dyn Screen>,
    exit_handler: ExitHandler,
}

impl App {
    /// 创建应用实例。
    pub fn new() -> Self {
        Self {
            active_screen: Box::new(MenuScreen::new()),
            exit_handler: ExitHandler::new(Duration::from_secs(1)),
        }
    }

    /// 运行主事件循环。
    ///
    /// 每次迭代渲染当前画面并监听键盘事件。1 秒内连按两次 `Ctrl+C` 退出程序。
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            if self.exit_handler.should_exit() {
                return Ok(());
            }

            self.exit_handler.tick();

            let hint = self.exit_handler.hint_text();
            
            terminal.draw(|frame| self.active_screen.render(frame, hint))?;

            if !event::poll(self.exit_handler.poll_timeout())? {
                continue;
            }

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.dispatch_key(key);
            }
        }
    }

    /// 分发键盘事件。
    ///
    /// Ctrl+C 由 App 拦截以保证全局行为一致，其余按键委托给当前画面。
    fn dispatch_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit_handler.press_ctrl_c();
            return;
        }

        if let Some(cmd) = self.active_screen.handle_key(key) {
            self.exit_handler.reset();
            match cmd {
                ScreenCommand::Stay => {}
                ScreenCommand::Navigate(screen) => self.active_screen = screen,
            }
        }
    }
}
