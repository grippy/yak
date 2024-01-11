use anyhow::{Context, Result};
use log::info;
use std::path::{Component, Path, PathBuf};
use std::{fs, fs::File, io::copy};

// Strip quotes from start/end of AST strings
pub fn clean_quotes(mut s: String) -> String {
    if s.starts_with("\"") {
        s = s.strip_prefix("\"").unwrap().to_string();
    }
    if s.ends_with("\"") {
        s = s.strip_suffix("\"").unwrap().to_string();
    }
    s
}

// Normalize path
pub fn normalize_path(path: &Path) -> PathBuf {
    // info!("normalize {:?}", &path);
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    // info!("ret {:?}", &ret);
    ret
}

// Download file
pub fn download_file(url: &str, file_path: &str) -> Result<()> {
    // Send an HTTP GET request to the URL
    let mut response = reqwest::blocking::get(url)?;

    // Create file directory?
    let file_dir = Path::new(&file_path)
        .components()
        .as_path()
        .parent()
        .unwrap();

    if !Path::new(file_dir).exists() {
        info!("creating file directory {}", &file_dir.display());
        let _ = fs::create_dir_all(file_dir)
            .with_context(|| format!("failed to create file directory: {}", file_dir.display()))?;
    }

    // Create a new file to write the downloaded file to
    let fd = PathBuf::from(&file_path);
    let mut file = File::create(fd)?;

    // Copy the contents of the response to the file
    copy(&mut response, &mut file)?;

    Ok(())
}
