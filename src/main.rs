//! mini-excel —— 基于 Ratatui 的 TUI 迷你 Excel 应用。
//!
//! 按两次 Ctrl+C 退出。

mod app;
mod exit_handler;
mod footer;
mod logo;
mod menu;
mod menu_screen;
mod screen;
mod theme;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut app = app::App::new();
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
