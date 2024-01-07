use futures::io;
use futures::StreamExt;
use manifest::config::Config;
use manifest::dependencies::Dependency;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Default, Clone)]
pub struct Client;

#[derive(Clone, Eq, PartialEq)]
pub enum DownloadResult {
    Downloaded,
    Exist,
    Failed(String),
}

impl DownloadResult {
    pub fn is_downloaded(&self) -> bool {
        self == &DownloadResult::Downloaded
    }

    pub fn is_cached(&self) -> bool {
        self == &DownloadResult::Exist
    }

    pub fn is_failed(&self) -> bool {
        !self.is_cached() && !self.is_downloaded()
    }
}

impl Client {
    pub fn download(&self, dep: &Dependency, config: &Config) -> DownloadResult {
        if let Err(err) = create_dir_all(&dep.target_dir) {
            return DownloadResult::Failed(err.to_string());
        }

        let repos = &config.manifest.repositories;

        let downloaded =
            repos
                .iter()
                .fold(DownloadResult::Failed("initialized".into()), |acc, repo| {
                    // success or exists from previous iteration
                    if acc.is_downloaded() || acc.is_cached() {
                        acc
                    } else if check_target_file(&dep.target_dir, &dep.jar) {
                        DownloadResult::Exist
                    } else {
                        let url = format!("{}/{}", repo.url, dep.path);
                        let jar = download_file(&url, &dep.target_dir, &dep.jar);
                        let pom = download_file(&url, &dep.target_dir, &dep.pom);
                        let _src = download_file(&url, &dep.target_dir, &dep.sources);
                        let _mod = download_file(&url, &dep.target_dir, &dep.module);

                        match (jar, pom) {
                            (DownloadResult::Downloaded, DownloadResult::Downloaded) => {
                                DownloadResult::Downloaded
                            }
                            (DownloadResult::Exist, DownloadResult::Exist) => DownloadResult::Exist,
                            (DownloadResult::Failed(e), _) => DownloadResult::Failed(e),
                            (_, DownloadResult::Failed(e)) => DownloadResult::Failed(e),
                            _ => DownloadResult::Failed(format!(
                                "Failed when downloading dep {}",
                                dep
                            )),
                        }
                    }
                });

        downloaded
    }
}

fn check_target_file(target_dir: &Path, filename: &String) -> bool {
    target_dir.join(filename).exists()
}

fn create_target_file(target_dir: &Path, filename: &String) -> anyhow::Result<File> {
    let file_path = target_dir.join(filename);
    let file = File::create(&file_path)?;
    Ok(file)
}

fn delete_target_file(target_dir: &Path, filename: &String) -> anyhow::Result<()> {
    let file_path = target_dir.join(filename);
    std::fs::remove_file(file_path)?;
    Ok(())
}

/*
fn download_target_file(mut file: &File, url: &String) -> anyhow::Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    io::copy(&mut response, &mut file)?;
    Ok(())
}
*/

fn download_file(url: &String, target_dir: &Path, filename: &String) -> DownloadResult {
    let target_file = match create_target_file(target_dir, filename) {
        Ok(file) => file,
        Err(e) => return DownloadResult::Failed(format!("Failed to create target file: {}", e)),
    };

    let url = format!("{url}{filename}");
    match download_target_file(&target_file, &url) {
        Ok(_) => DownloadResult::Downloaded,
        Err(e) => {
            delete_target_file(target_dir, filename).unwrap();
            DownloadResult::Failed(format!("Failed to download file: {}", e))
        }
    }
}

async fn download_target_file(mut file: &File, url: &String) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    //let mut res = reqwest::get(url).await?;

    let size = res.content_length().unwrap_or(0);

    let spinner = [
        "▰▱▱▱▱▱▱▱▱▱",
        "▰▰▱▱▱▱▱▱▱▱",
        "▰▰▰▱▱▱▱▱▱▱",
        "▰▰▰▰▱▱▱▱▱▱",
        "▰▰▰▰▰▱▱▱▱▱",
        "▰▰▰▰▰▰▱▱▱▱",
        "▰▰▰▰▰▰▰▱▱▱",
        "▰▰▰▰▰▰▰▰▱▱",
        "▰▰▰▰▰▰▰▰▰▱",
        "▰▰▰▰▰▰▰▰▰▰",
    ];

    let mut downloaded = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk);
        downloaded += chunk.len();
        let progress = (downloaded as f64 / size as f64) * 10.0;
        let progress = progress.floor() as usize;

        print!("\r{}", spinner[progress]);
    }

    print!("\r");
    Ok(())
}
