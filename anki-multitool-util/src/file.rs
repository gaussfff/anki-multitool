use anyhow::{Result, anyhow};
use std::path::Path;

pub fn to_file_name<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(path
        .as_ref()
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("invalid file name: {}", path.as_ref().display()))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_to_file_name() {
        assert_eq!(
            to_file_name(Path::new("example.md")).expect("can't convert to file name"),
            "example"
        );
        assert_eq!(
            to_file_name(Path::new("example.json")).expect("can't convert to file name"),
            "example"
        );
        assert_eq!(
            to_file_name(Path::new("example.json.md")).expect("can't convert to file name"),
            "example.json"
        );
        assert_eq!(
            to_file_name(Path::new("example.md.json")).expect("can't convert to file name"),
            "example.md"
        );
        assert_eq!(
            to_file_name(Path::new("a/b/c/example.md")).expect("can't convert to file name"),
            "example"
        );
        assert_eq!(
            to_file_name(Path::new("a/b/c/example.zip.json")).expect("can't convert to file name"),
            "example.zip"
        );
        assert_eq!(
            to_file_name(Path::new("a")).expect("can't convert to file name"),
            "a"
        );
    }
}
