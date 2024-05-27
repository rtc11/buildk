use std::path::PathBuf;

pub fn kotlin_home(manifest: &toml_edit::DocumentMut) -> Option<PathBuf> {
    manifest
        .as_table()
        .get("kotlin")
        .and_then(|it| it.as_str())
        .map(PathBuf::from)
}
