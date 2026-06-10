//! 剪贴板操作与 TSV 编解码。
//!
//! 复制时：单元格数据 → [`to_tsv`] → [`copy_to_clipboard`]
//! 粘贴时：[`read_from_clipboard`] → [`from_tsv`] → 单元格数据

/// 复制文本到系统剪贴板。
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut last_err = String::new();
    for _ in 0..3 {
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = e.to_string(),
        }
    }
    Err(last_err)
}

/// 从系统剪贴板读取文本。
pub fn read_from_clipboard() -> Result<String, String> {
    let mut last_err = String::new();
    for _ in 0..3 {
        match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(text) => return Ok(text),
            Err(e) => last_err = e.to_string(),
        }
    }
    Err(last_err)
}

/// 将二维单元格网格编码为 TSV 文本。
///
/// 行内用 `\t` 分隔列，行间用 `\n` 分隔。
/// 空字符串代表空单元格。
pub fn to_tsv(rows: &[Vec<String>]) -> String {
    rows.iter()
        .map(|row| row.join("\t"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 将 TSV 文本解析为二维字符串网格。
///
/// - 按 `\n` 分行（同时处理 `\r\n`）
/// - 去除末尾空行
/// - 每行按 `\t` 分列
pub fn from_tsv(text: &str) -> Vec<Vec<String>> {
    let trimmed = text.trim_end_matches('\n').trim_end_matches('\r');
    if trimmed.is_empty() {
        return vec![];
    }
    trimmed
        .lines()
        .map(|line| line.split('\t').map(|s| s.to_string()).collect())
        .collect()
}
