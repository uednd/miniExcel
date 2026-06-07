use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct Theme {
    /// 全局背景色。
    pub bg: Color,
    /// 内容面板背景色。
    pub surface: Color,
    /// 标签栏背景色。
    pub surface_alt: Color,
    /// 表格网格线颜色。
    pub grid: Color,
    /// 表格选中行/列头背景色。
    pub table_header_highlight_bg: Color,
    /// 主题色（强调色）。
    pub accent: Color,
    /// 主题色上的文字色（选中态）。
    pub accent_text: Color,
    /// 高亮文本。
    pub text: Color,
    /// 普通/次要文本。
    pub text_dim: Color,
    /// Logo 浅色部分。
    pub logo_light: Color,
    /// Logo 亮色部分。
    pub logo_bright: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(10, 10, 10),
            surface: Color::Rgb(28, 28, 28),
            surface_alt: Color::Rgb(18, 18, 18),
            grid: Color::Rgb(80, 80, 80),
            table_header_highlight_bg: Color::Rgb(56, 56, 56),
            accent: Color::Rgb(80, 160, 100),
            accent_text: Color::Rgb(16, 32, 22),
            text: Color::Rgb(240, 240, 240),
            text_dim: Color::Rgb(150, 150, 150),
            logo_light: Color::Rgb(135, 142, 142),
            logo_bright: Color::Rgb(220, 224, 224),
        }
    }
}
