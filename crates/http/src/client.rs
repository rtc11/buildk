use futures::StreamExt;
use futures::lock::Mutex;
use manifest::config::Config;
use manifest::dependencies::Dependency;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::thread;

const ENABLE_PROGRESSBAR: bool = false;

#[derive(Default, Clone)]
pub struct Client;

#[derive(Clone, Eq, PartialEq)]
pub enum DownloadResult {
    Downloaded,
    Exist,
    Failed(String),
}

impl Client {
    pub fn download_blocking(
        &self,
        dep: &Dependency,
        config: &Config,
    ) -> DownloadResult {
        if let Err(err) = create_dir_all(&dep.target_dir) {
            return DownloadResult::Failed(err.to_string());
        }

        let repo = &config.manifest.repositories[0]; // todo: support multiple repos
        let files = [&dep.jar, &dep.pom, &dep.sources, &dep.module];
        let downloads = files.into_iter().map(|file| {
            let repo = repo.clone();
            let file = file.clone();
            let dep = dep.clone();
            thread::spawn(move || {
                download_file_blocking(
                    &format!("{}/{}", &repo.url, &dep.path),
                    &dep.target_dir,
                    &file,
                )
            })
        }).collect::<Vec<_>>();
        for d in downloads {
            d.join().unwrap();
        }
        return DownloadResult::Downloaded;
    }

    #[allow(dead_code)]
    pub async fn download(
        &self, 
        dep: Dependency, 
        config: Arc<Mutex<Config>>
    ) -> DownloadResult {
        if let Err(err) = create_dir_all(&dep.target_dir) {
            return DownloadResult::Failed(err.to_string());
        }

        let config = config.lock().await;
        let repos = &config.manifest.repositories;
        let files = [&dep.jar, &dep.pom, &dep.sources, &dep.module];

        let handles = repos
            .iter()
            .flat_map(|repo| {
                files.into_iter().map(|file| {
                    let repo = repo.clone();
                    let file = file.clone();
                    let dep = dep.clone();
                    
                    tokio::spawn(async move {
                        download_file(
                            &format!("{}/{}", &repo.url, &dep.path),
                            &dep.target_dir,
                            &file
                        ).await
                    })
                }).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let downloads = futures::future::join_all(handles).await;

        let download_result = downloads
            .iter()
            .fold(DownloadResult::Failed("".to_string()), |acc, d| 
                match d {
                    Ok(DownloadResult::Downloaded) => DownloadResult::Downloaded,
                    Ok(DownloadResult::Exist) => DownloadResult::Exist,
                    Ok(DownloadResult::Failed(err)) => {
                        match acc {
                            DownloadResult::Failed(acc_err) => DownloadResult::Failed(format!("{}:::{}", acc_err, err)),
                            _ => DownloadResult::Failed(err.to_string()),
                        }

                    },
                    Err(err) => DownloadResult::Failed(err.to_string()), 
                    /*
                    Err(err) => {
                        match acc {
                            DownloadResult::Failed(acc_err) => DownloadResult::Failed(format!("error joining threads: {}\n{}", acc_err, err)),
                            _ => DownloadResult::Failed(err.to_string()),
                        }
                    }
                    */
                },
            );

        download_result
    }
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

#[allow(dead_code)]
fn check_target_file(target_dir: &Path, filename: &String) -> bool {
    target_dir.join(filename).exists()
}

fn create_target_file(target_dir: &Path, filename: &String) -> anyhow::Result<File> {
    let file_path = target_dir.join(filename);
    let file = File::create(file_path)?;
    Ok(file)
}

fn delete_target_file(target_dir: &Path, filename: &String) -> anyhow::Result<()> {
    let file_path = target_dir.join(filename);
    std::fs::remove_file(file_path)?;
    Ok(())
}

fn download_file_blocking(url: &String, target_dir: &Path, filename: &String) -> DownloadResult {
    if check_target_file(target_dir, filename) {
        return DownloadResult::Exist;
    }
    let target_file = match create_target_file(target_dir, filename) {
        Ok(file) => file,
        Err(e) => return DownloadResult::Failed(format!("Failed to create target file: {}", e)),
    };

    let url = format!("{url}{filename}");
    match download_target_file_blocking(&target_file, &url) {
        Ok(_) => DownloadResult::Downloaded,
        Err(e) => {
            delete_target_file(target_dir, filename).unwrap();
            DownloadResult::Failed(format!("Failed to download file: {}", e))
        }
    }
}

async fn download_file(url: &String, target_dir: &Path, filename: &String) -> DownloadResult {
    if check_target_file(target_dir, filename) {
        return DownloadResult::Exist;
    }
    let target_file = match create_target_file(target_dir, filename) {
        Ok(file) => file,
        Err(e) => return DownloadResult::Failed(format!("Failed to create target file: {}", e)),
    };

    let url = format!("{url}{filename}");
    match download_target_file(&target_file, &url).await {
        Ok(_) => DownloadResult::Downloaded,
        Err(e) => {
            delete_target_file(target_dir, filename).unwrap();
            DownloadResult::Failed(format!("Failed to download file: {}", e))
        }
    }
}

fn download_target_file_blocking(mut file: &File, url: &str) -> anyhow::Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    std::io::copy(&mut response, &mut file)?;
    Ok(())
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
        file.write_all(&chunk)?;
        if ENABLE_PROGRESSBAR {
            downloaded += chunk.len();
            let progress = (downloaded as f64 / size as f64) * 10.0;
            let progress = progress.floor() as usize;
            print!("\r{}", spinner[progress]);
        }
    }

    print!("\r");
    Ok(())
}
