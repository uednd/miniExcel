//! App 状态管理与事件循环。
//!
//! 包含主事件循环、键盘事件分发及各 UI 组件的渲染调度。

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    style::{Color, Style},
    widgets::Block,
    widgets::Paragraph,
};

use crate::{exit_handler::ExitHandler, footer::Footer, logo::LOGO_HEIGHT, logo::Logo, menu::Menu};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// logo 到菜单的间距。
const LOGO_MENU_GAP: u16 = 2;

/// 应用全局状态，持有各 UI 组件。
pub struct App {
    logo: Logo,
    menu: Menu,
    footer: Footer,
    exit_handler: ExitHandler,
}

impl App {
    /// 创建应用实例，初始化各 UI 组件并获取当前工作目录路径。
    pub fn new() -> Self {
        let cwd_path = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("."));
        let cwd = replace_homedir::replace_homedir(&cwd_path, "~");
        Self {
            logo: Logo::new(),
            menu: Menu::new(),
            footer: Footer::new(cwd, APP_VERSION.to_string()),
            exit_handler: ExitHandler::new(Duration::from_secs(1)),
        }
    }

    /// 运行主事件循环。
    ///
    /// 每次迭代渲染界面并监听键盘事件。`←/→` 切换菜单选中项，1 秒内连按两次 `Ctrl+C` 退出程序。
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            if self.exit_handler.should_exit() {
                return Ok(());
            }

            self.exit_handler.tick();

            terminal.draw(|frame| self.render(frame))?;

            if !event::poll(self.exit_handler.poll_timeout())? {
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
                self.exit_handler.reset();
            }
            KeyCode::Right => {
                self.menu.move_right();
                self.exit_handler.reset();
            }
            KeyCode::Enter => {
                todo!();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit_handler.press_ctrl_c();
            }
            KeyCode::Char(_) => {
                self.exit_handler.reset();
            }
            _ => {}
        }
    }

    /// 渲染主界面。
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // 黑色背景
        frame.render_widget(
            Block::new().style(Style::default().bg(Color::Rgb(10, 10, 10))),
            area,
        );

        let [body, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

        let [_, logo_area, _, menu_area, hint_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(LOGO_HEIGHT),
            Constraint::Length(LOGO_MENU_GAP),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(body);

        self.logo.render(frame, logo_area);
        self.menu.render(frame, menu_area);
        self.footer.render(frame, footer_area);

        if let Some(hint) = self.exit_handler.hint_text() {
            frame.render_widget(
                Paragraph::new(hint).centered().yellow().bold(),
                hint_area,
            );
        }
    }
}
