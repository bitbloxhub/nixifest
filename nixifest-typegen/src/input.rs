use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub fn collect_inputs(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            collect_dir(path, &mut files)?;
        } else {
            bail!("input does not exist: {}", path.display());
        }
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        bail!("no input files found");
    }
    Ok(files)
}

fn collect_dir(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(path).with_context(|| format!("reading {}", path.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir(&path, files)?;
        } else if path.is_file() && is_input_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_input_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("json" | "yaml" | "yml")
    )
}
