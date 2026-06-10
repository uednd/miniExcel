use std::path::{Path, PathBuf};

const TABLE_EXTENSION: &str = "mxlsx";

pub fn resolve_table_path(input: &str, cwd: &Path) -> PathBuf {
    let raw = PathBuf::from(input);
    let with_extension = if raw.extension().is_some() {
        raw
    } else {
        raw.with_extension(TABLE_EXTENSION)
    };

    let absolute = if with_extension.is_absolute() {
        with_extension
    } else {
        cwd.join(with_extension)
    };

    absolute
}
