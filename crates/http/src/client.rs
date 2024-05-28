use anyhow::anyhow;
use async_std::{fs::{create_dir_all, File, remove_file}, io, path::{Path, PathBuf}, task};

use dependency::Package;
use manifest::{config::BuildK, repos::Repo};
use util::DEBUG;

#[derive(Default, Clone)]
pub struct Client;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum DownloadResult {
    Downloaded,
    Exist,
    Failed(String),
}

impl Client {
    pub async fn download_async<'a>(&'a self, pkg: &'a Package, buildk: &'a BuildK) -> DownloadResult {
        if let Err(err) = create_dir_all(&pkg.location).await {
            return DownloadResult::Failed(err.to_string());
        }

        let (jar, pom) = task::block_on(async {
            let mut jar = DownloadResult::Failed("".into());
            let mut pom = DownloadResult::Failed("".into());

            // FIXME
            let manifest = <Option<manifest::Manifest> as Clone>::clone(&buildk.manifest)
                .expect("no buildk.toml found.");

            for repo in manifest.repos.repos.iter() {
                let repo = repo.clone();
                (jar, pom) = Self::download_jar_and_pom(&pkg, &repo).await;
                if !jar.is_failed() && !pom.is_failed() {
                    break;
                } else {
                    println!("\rfrom {} failed, trying", &repo.url);
                }
            }
            (jar, pom)
        });

        if let DownloadResult::Failed(_) = jar {
            return jar;
        }

        if let DownloadResult::Failed(_) = pom {
            return pom;
        }

        if jar == DownloadResult::Downloaded || pom == DownloadResult::Downloaded {
            return DownloadResult::Downloaded;
        }

        //println!("{} is cached", &dep.name);
        DownloadResult::Exist
    }

    fn resolve_url(pkg: &&Package, repo: &Repo) -> String {
        let name = &pkg.name;
        let version = &pkg.version;
        let path = pkg.namespace.clone().unwrap().replace('.', "/"); // TODO: support no namespace
        let path = format!("{path}/{name}/{version}");
        let file_prefix = format!("{name}-{version}");
        format!("{}/{}/{}", &repo.url, &path, &file_prefix)
    }

    async fn download_jar_and_pom(pkg: &&Package, repo: &Repo) -> (DownloadResult, DownloadResult) {
        let url = Self::resolve_url(&pkg, &repo);
        let target_dir = PathBuf::from(&pkg.location);
        let maven = target_dir.join("maven.xml");
        let gradle = target_dir.join("gradle.json");
        let sources = target_dir.join("sources.jar");
        let jar = target_dir.join("pkg").with_extension("jar");

        let jar_res = create_target_and_download(&format!{"{}.jar", &url}, &jar).await;
        let pom_res = create_target_and_download(&format!{"{}.pom", &url}, &maven).await;
        let _optional = create_target_and_download(&format!{"{}-sources.jar", &url}, &sources).await;
        let _optional = create_target_and_download(&format!{"{}.module", &url}, &gradle).await;

        if !jar_res.is_failed() && !pom_res.is_failed() {
            return (jar_res, pom_res);
        }

        println!("jar or pom failed to download. Try to resolve download url differently?");

        // trying alternative names
        // let alt_prefix = format!("{}", &pkg.path).replace("/", ".").substr_before_last('.');
        // let pom = format!("{}.pom", &alt_prefix);
        // let module = format!("{}.module", &alt_prefix);
        // let jar = format!("{}.jar", &alt_prefix);
        // let sources = format!("{}-sources.jar", &alt_prefix);
        // if DEBUG {
        //     println!("trying alternative names: {} {} {} {}", &jar, &pom, &sources, &module);
        // }
        // let jar_res = create_target_and_download(&base_url, &target_dir, &jar).await;
        // let pom_res = create_target_and_download(&base_url, &target_dir, &pom).await;
        // let _optional = create_target_and_download(&base_url, &target_dir, &sources).await;
        // let _optional = create_target_and_download(&base_url, &target_dir, &module).await;

        (jar_res, pom_res)
    }
}

async fn create_target_and_download(url: &String, target: &Path) -> DownloadResult {
    if target_exists(&target).await {
        if DEBUG {
            println!("{} already exists", target.display());
        }
        return DownloadResult::Exist;
    }

    let target_file = match create_target_file(target).await {
        Ok(file) => file,
        Err(e) => {
            if DEBUG {
                println!("failed to create target file: {}", e);
            }
            return DownloadResult::Failed(format!("Failed to create target file: {}", e));
        }
    };

    match download(&target_file, &url).await {
        Ok(_) => DownloadResult::Downloaded,
        Err(e) => {
            // if DEBUG {
            //     println!("failed to downalod file: {}", e);
            // }
            delete_target_file(target).await.unwrap();
            DownloadResult::Failed(format!("Failed to download file from {} with err: {}", &url, e))
        }
    }
}

async fn target_exists(file: &Path) -> bool {
    file.exists().await && file.metadata().await.unwrap().len() > 0
}

async fn create_target_file(file: &Path) -> anyhow::Result<File> {
    let file = File::create(file).await?;
    Ok(file)
}

async fn delete_target_file(file: &Path) -> anyhow::Result<()> {
    remove_file(file).await?;
    // if DEBUG {
    //     println!("deleted {}", file.display());
    // }
    Ok(())
}

async fn download(mut file: &File, url: &str) -> anyhow::Result<()> {
    if DEBUG {
        println!("downloading {}", url);
    }
    let mut response = surf::get(url).await.map_err(|e| anyhow::anyhow!(e))?;
    /* if DEBUG {
        println!("downloaded {:?}!", response);
    } */

    if response.status().is_success() {
        io::copy(&mut response, &mut file).await?;
        Ok(())
    } else {
        Err(anyhow!(response.status().to_string()))
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

