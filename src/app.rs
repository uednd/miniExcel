//! App 状态管理与事件循环。
//!
//! 包含主事件循环、键盘事件分发及各 UI 组件的渲染调度。

use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    style::Stylize,
    style::{Color, Style},
    widgets::Block,
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use crate::{footer::Footer, logo::LOGO_HEIGHT, logo::Logo, menu::LOGO_MENU_GAP, menu::Menu};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const EXIT_CONFIRM_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug)]
enum ExitState {
    Idle,
    ConfirmOnce(Instant),
    Confirmed,
}

/// 应用全局状态，持有各 UI 组件及退出状态。
pub struct App {
    logo: Logo,
    menu: Menu,
    footer: Footer,
    exit_state: ExitState,
}

impl App {
    /// 创建应用实例，初始化各 UI 组件并获取当前工作目录路径。
    pub fn new() -> Self {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("."));
        Self {
            logo: Logo::new(),
            menu: Menu::new(),
            footer: Footer::new(cwd, APP_VERSION.to_string()),
            exit_state: ExitState::Idle,
        }
    }

    /// 运行主事件循环。
    ///
    /// 每次迭代渲染界面并监听键盘事件。`←/→` 切换菜单选中项，1 秒内连按两次 `Ctrl+C` 退出程序。
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            // 连续按两次 => 退出程序
            if matches!(self.exit_state, ExitState::Confirmed) {
                return Ok(());
            }

            // 按一次 && 超时 => 退出状态改为空闲
            if let ExitState::ConfirmOnce(first) = self.exit_state
                && first.elapsed() >= EXIT_CONFIRM_TIMEOUT
            {
                self.exit_state = ExitState::Idle;
            }

            // 绘制界面
            terminal.draw(|frame| self.render(frame))?;

            // 按一次
            let timeout = match self.exit_state {
                ExitState::Idle | ExitState::Confirmed => Duration::from_millis(100),
                ExitState::ConfirmOnce(first) => {
                    EXIT_CONFIRM_TIMEOUT.saturating_sub(first.elapsed())
                }
            };

            // 按一次 && 超时 => 重新绘制界面
            if !event::poll(timeout)? {
                continue;
            }

            // 捕捉键盘事件
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key);
            }
        }
    }

    /// 处理键盘事件，分发到对应组件或更新退出状态。
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Left => {
                self.menu.move_left();
                self.exit_state = ExitState::Idle;
            }
            KeyCode::Right => {
                self.menu.move_right();
                self.exit_state = ExitState::Idle;
            }
            KeyCode::Enter => {
                todo!();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let now = Instant::now();
                match self.exit_state {
                    ExitState::Idle => {
                        self.exit_state = ExitState::ConfirmOnce(now);
                    }
                    ExitState::ConfirmOnce(first)
                        if now.duration_since(first) < EXIT_CONFIRM_TIMEOUT =>
                    {
                        self.exit_state = ExitState::Confirmed;
                    }
                    ExitState::ConfirmOnce(_) => {
                        self.exit_state = ExitState::ConfirmOnce(now);
                    }
                    ExitState::Confirmed => {}
                }
            }
            KeyCode::Char(_) => {
                self.exit_state = ExitState::Idle;
            }
            _ => {}
        }
    }

    /// 渲染主界面。
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        frame.render_widget(
            Block::new().style(Style::default().bg(Color::Rgb(0, 0, 0))),
            area,
        );

        self.logo.render(frame, area);
        self.menu.render(frame, area);
        self.footer.render(frame, area);

        if let ExitState::ConfirmOnce(_) = self.exit_state {
            let hint = "再次按下 Ctrl+C 以退出";
            let hint_w = UnicodeWidthStr::width(hint) as u16;
            let hint_x = area.x + (area.width.saturating_sub(hint_w)) / 2;
            let logo_bottom = area.y + (area.height.saturating_sub(LOGO_HEIGHT)) / 2 + LOGO_HEIGHT;
            let hint_y = logo_bottom + LOGO_MENU_GAP + 1;
            let hint_rect = Rect::new(hint_x, hint_y, hint_w, 1);
            frame.render_widget(Paragraph::new(hint).yellow().bold(), hint_rect);
        }
    }
}
