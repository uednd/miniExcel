use ratatui::style::Style;
use ratatui::text::Span;

/// 光标闪烁判断
pub fn blink_visible() -> bool {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        % 1000
        < 500
}

/// 根据闪烁状态返回光标字符（█ 或空格）
pub fn cursor_span(style: Style) -> Span<'static> {
    let c = if blink_visible() { "█" } else { " " };
    Span::styled(c, style)
}
