/*
use futures::StreamExt;

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

impl TokioClient for Client {
    #[allow(dead_code)]
    async fn download(
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
                },
            );

        download_result
    }
}
*/
