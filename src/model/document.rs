use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{
    limits::{MAX_COLUMNS, MAX_ROWS},
    workbook::Workbook,
};

const TABLE_EXTENSION: &str = "mxlsx";

/// 工作簿文档：工作簿数据和它所在的文件路径。
///
/// 打开、创建、保存和文件格式错误都集中在这里，调用方不需要直接组合
/// `PathBuf + Workbook + JSON IO`。
pub struct WorkbookDocument {
    path: PathBuf,
    workbook: Workbook,
}

#[derive(Debug)]
pub enum DocumentError {
    Read { path: PathBuf },
    Parse,
    Write { path: PathBuf },
}

impl DocumentError {
    pub fn message(&self) -> String {
        match self {
            DocumentError::Read { path } => {
                format!("无法打开：{}", display_path(path))
            }
            DocumentError::Parse => String::from("无法打开：文件格式无效"),
            DocumentError::Write { path } => {
                format!("无法保存：{}", display_path(path))
            }
        }
    }
}

impl WorkbookDocument {
    pub fn open_or_create(path: PathBuf) -> Result<Self, DocumentError> {
        if path.exists() {
            let raw = fs::read_to_string(&path)
                .map_err(|_| DocumentError::Read { path: path.clone() })?;
            let mut workbook: Workbook =
                serde_json::from_str(&raw).map_err(|_| DocumentError::Parse)?;
            workbook.recalc();
            return Ok(Self { path, workbook });
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();
        Ok(Self {
            path,
            workbook: Workbook::new(name, MAX_COLUMNS, MAX_ROWS),
        })
    }

    pub fn save(&self) -> Result<(), DocumentError> {
        let json =
            serde_json::to_string_pretty(&self.workbook).map_err(|_| DocumentError::Write {
                path: self.path.clone(),
            })?;
        fs::write(&self.path, json).map_err(|_| DocumentError::Write {
            path: self.path.clone(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn workbook(&self) -> &Workbook {
        &self.workbook
    }

    pub fn workbook_mut(&mut self) -> &mut Workbook {
        &mut self.workbook
    }
}

pub fn resolve_table_path(input: &str, cwd: &Path) -> PathBuf {
    let raw = PathBuf::from(input);
    let with_extension = if raw.extension().is_some() {
        raw
    } else {
        raw.with_extension(TABLE_EXTENSION)
    };

    if with_extension.is_absolute() {
        with_extension
    } else {
        cwd.join(with_extension)
    }
}

fn display_path(path: &Path) -> String {
    replace_homedir::replace_homedir(&path.display().to_string(), "~")
}
