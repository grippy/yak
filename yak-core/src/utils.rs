use anyhow::{Context, Result};
use std::path::{Component, Path, PathBuf};
use std::{fs, fs::File, io::copy};

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

pub fn download_file(url: &str, file_dir: &str, file_path: &str) -> Result<()> {
    // Send an HTTP GET request to the URL
    let mut response = reqwest::blocking::get(url)?;

    // Create file path...
    if !Path::new(file_dir).exists() {
        let _ = fs::create_dir_all(file_dir)
            .with_context(|| format!("failed to create file directory: {}", file_dir))?;
    }

    // Create a new file to write the downloaded image to
    let fd = PathBuf::from(&file_path);
    let mut file = File::create(fd)?;

    // Copy the contents of the response to the file
    copy(&mut response, &mut file)?;

    Ok(())
}
