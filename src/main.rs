//! mini-excel —— 基于 Ratatui 的 TUI 迷你 Excel 应用。
//!
//! 按两次 Ctrl+Q 退出。

mod app;
mod clipboard;
mod exit;
mod model;
mod screen;
mod theme;
mod util;
mod widget;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut app = app::App::new();
    ratatui::run(|terminal| {
        crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;
        let result = app.run(terminal);
        crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
        result
    })?;
    Ok(())
}
