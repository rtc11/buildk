use async_std::{path::{Path, PathBuf}, fs::{File, remove_file, create_dir_all}, io, task};
use manifest::{config::Config, dependencies::Dependency};

#[derive(Default, Clone)]
pub struct Client;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum DownloadResult {
    Downloaded,
    Exist,
    Failed(String),
}

impl Client {
pub async fn download_async<'a>(&'a self, dep: &'a Dependency, config: &'a Config) -> DownloadResult {
        if let Err(err) = create_dir_all(&dep.target_dir).await {
            return DownloadResult::Failed(err.to_string());
        }

        let (jar, pom) = task::block_on(async {
            let repo = config.manifest.repositories[0].clone(); // TODO: support all repos  
            let base_url = format!("{}/{}", &repo.url, &dep.path);
            let target_dir = PathBuf::from(&dep.target_dir);
            
            let jar_res = create_target_and_download(&base_url, &target_dir, &dep.jar).await;
            let pom_res = create_target_and_download(&base_url, &target_dir, &dep.pom).await;
            let _optional = create_target_and_download(&base_url, &target_dir, &dep.sources).await;
            let _optional = create_target_and_download(&base_url, &target_dir, &dep.module).await;
            (jar_res, pom_res)
        });
        
        if let DownloadResult::Failed(_) = jar {
            //println!("failed to download jar: {:?}", &jar);
            return jar
        }

        if let DownloadResult::Failed(_) = pom {
            //println!("failed to download pom: {:?}", &jar);
            return pom
        }

        if jar == DownloadResult::Downloaded || pom == DownloadResult::Downloaded {
            //println!("downloaded either {} or {}", &dep.name, &dep.version);
            return DownloadResult::Downloaded;
        }

        //println!("{} is cached", &dep.name);
        DownloadResult::Exist
    }
}

async fn create_target_and_download(base_url: &String, target_dir: &Path, filename: &String) -> DownloadResult {
    if target_exists(target_dir, filename).await {
        //println!("{} already exists", filename);
        return DownloadResult::Exist;

    }

    let target_file = match create_target_file(target_dir, filename).await {
        Ok(file) => file,
        Err(e) => {
            //println!("failed to create target file: {}", e);
            return DownloadResult::Failed(format!("Failed to create target file: {}", e));
        }
    };

    let url = format!("{base_url}{filename}");

    //println!("whole url: {}", url);

    match download(&target_file, &url).await {
        Ok(_) => DownloadResult::Downloaded,
        Err(e) => {
            //println!("failed to downalod file: {}", e);
            delete_target_file(target_dir, filename).await.unwrap();
            DownloadResult::Failed(format!("Failed to download file: {}", e))
        }
    }
}
async fn target_exists(dir: &Path, filename: &String) -> bool {
    let target = dir.join(filename);
    target.exists().await && target.metadata().await.unwrap().len() > 0
}

async fn create_target_file(target_dir: &Path, filename: &String) -> anyhow::Result<File> {
    let file_path = target_dir.join(filename);
    let file = File::create(file_path).await?;
    Ok(file)
}

async fn delete_target_file(target_dir: &Path, filename: &String) -> anyhow::Result<()> { 
    let file_path = target_dir.join(filename);
    remove_file(file_path).await?;
    //println!("deleted {}", filename);
    Ok(())
}

async fn download(mut file: &File, url: &str) -> anyhow::Result<()> {
    //println!("downloading {}...", url);
    let mut response = surf::get(url).await.map_err(|e| anyhow::anyhow!(e))?;
    io::copy(&mut response, &mut file).await?;
    //println!("downloaded {:?}!", response);
    Ok(())
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

