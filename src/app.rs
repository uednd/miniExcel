//! App 状态管理与事件循环。
//!
//! 包含主事件循环、键盘事件分发、全局元素渲染。

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Block,
};

use crate::{
    exit_handler::ExitHandler,
    footer::Footer,
    menu_screen::MenuScreen,
    screen::{Screen, ScreenCommand},
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用全局状态。
pub struct App {
    active_screen: Box<dyn Screen>,
    exit_handler: ExitHandler,
    footer: Footer,
}

impl App {
    pub fn new() -> Self {
        let full_cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("."));
        let cwd = replace_homedir::replace_homedir(&full_cwd, "~");

        Self {
            active_screen: Box::new(MenuScreen::new()),
            exit_handler: ExitHandler::new(Duration::from_secs(1)),
            footer: Footer::new(cwd, APP_VERSION.to_string()),
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

            terminal.draw(|frame| {
                let area = frame.area();

                // 黑色背景
                frame.render_widget(
                    Block::new().style(Style::default().bg(Color::Rgb(10, 10, 10))),
                    area,
                );

                let [body, footer_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

                self.active_screen.render(frame, body, hint);
                self.footer.render(frame, footer_area);
            })?;

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
