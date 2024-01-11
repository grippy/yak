use anyhow::Result;
use clap::{arg, Args};
use log::info;

// use std::path::{Path, PathBuf};
// use std::{fs, fs::File, io::copy};

// fn download_file(url: &str, file_dir: &str, file_name: &str) -> Result<()> {
//     // Send an HTTP GET request to the URL
//     let mut response = reqwest::blocking::get(url)?;

//     // Create file path...
//     if !Path::new(file_dir).exists() {
//         let _ = fs::create_dir_all(file_dir)
//             .with_context(|| format!("failed to create file directory: {}", file_dir))?;
//     }
//     // Append file name to file path
//     let mut fd = PathBuf::from(&file_dir);
//     fd.push(&file_name);

//     // Create a new file to write the downloaded image to
//     let mut file = File::create(fd)?;

//     // Copy the contents of the response to the file
//     copy(&mut response, &mut file)?;

//     Ok(())
// }

#[derive(Args, Debug)]
pub(crate) struct GetArgs {
    /// Download remote package path to the local cache
    #[arg(index = 1)]
    path: String,
}

pub(crate) fn call(args: &GetArgs) -> Result<()> {
    info!("get args: {:?}", args);
    // get package path
    let remote_path = args.path.clone();
    yak_pkg::get(remote_path)?;
    Ok(())
}
