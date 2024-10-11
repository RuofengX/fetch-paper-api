use std::path::Path;

use anyhow::{anyhow, Ok, Result};
use constcat;
use reqwest;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

const BASE: &'static str = "https://papermc.io/api/v2";

#[derive(Debug, Deserialize)]
pub struct Root {
    pub projects: Vec<String>,
}

impl Root {
    pub const fn link() -> &'static str {
        constcat::concat!(BASE, "/projects")
    }
    pub async fn new() -> Result<Self> {
        Ok(reqwest::get(Root::link()).await?.json::<Self>().await?)
    }

    pub async fn get_project(&self, project: &str) -> Result<Project> {
        Project::new(project).await
    }
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub project_name: String,
    pub version_groups: Vec<String>,
    pub versions: Vec<String>,
}

impl Project {
    pub fn link(project: &str) -> String {
        format!("{0}/{1}", Root::link(), project)
    }

    pub async fn new(name: &str) -> Result<Self> {
        Ok(reqwest::get(Project::link(name))
            .await?
            .json::<Self>()
            .await?)
    }

    pub async fn get_version(&self, version: &str) -> Result<Version> {
        Ok(Version::new(&self.project_id, version).await?)
    }

    pub async fn get_latest_version(&self) -> Result<Version> {
        self.get_version(self.versions.last().ok_or(anyhow!("no version found"))?)
            .await
    }
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub project_id: String,
    pub project_name: String,
    pub version: String,
    pub builds: Vec<u16>,
}
impl Version {
    pub fn link(project: &str, version: &str) -> String {
        format!("{0}/versions/{1}", Project::link(project), version)
    }

    pub async fn new(project: &str, version: &str) -> Result<Self> {
        let link = Version::link(project, version);
        Ok(reqwest::get(link).await?.json::<Self>().await?)
    }

    pub async fn get_build(&self, build: u16) -> Result<Build> {
        Ok(Build::new(&self.project_id, &self.version, build).await?)
    }

    pub async fn get_latest_build(&self) -> Result<Build> {
        self.get_build(*self.builds.last().ok_or(anyhow!("no builds found"))?)
            .await
    }
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub project_id: String,
    pub project_name: String,
    pub version: String,
    pub build: u16,
    pub time: String,    //"2016-02-29T01:43:34.279Z"
    pub channel: String, //"default"
    pub promoted: bool,
    pub changes: Vec<BuildChange>,
    pub downloads: Application,
}
impl Build {
    pub fn link(project: &str, version: &str, build: u16) -> String {
        format!("{0}/builds/{1}", Version::link(project, version), build)
    }

    pub async fn new(project: &str, version: &str, build: u16) -> Result<Self> {
        let link = Build::link(project, version, build);
        Ok(reqwest::get(link).await?.json::<Self>().await?)
    }

    pub fn download_link(&self) -> String {
        format!(
            "{0}/downloads/{1}",
            Self::link(&self.project_id, &self.version, self.build),
            self.downloads.application.name
        )
    }

    pub fn download_digest_sha256(&self) -> &str {
        &self.downloads.application.sha256
    }

    pub async fn download(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut file = File::create(path).await?;

        let mut stream = reqwest::get(self.download_link()).await?.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        Ok(())
    }

    pub async fn check_sum(&self, path: impl AsRef<Path>) -> Result<bool> {
        fn sha256(path: impl AsRef<Path>) -> Result<String> {
            let mut file = std::fs::File::open(path)?;
            let mut hasher = Sha256::new();
            let n = std::io::copy(&mut file, &mut hasher)?;
            let hash = hasher.finalize();

            println!("Bytes processed: {}", n);
            println!("Hash value: {:x}", hash);
            Ok(format!("{:x}", hash))
        }

        Ok(self.download_digest_sha256() == sha256(path)?.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct BuildChange {
    pub commit: String,  //"a7b53030d943c8205513e03c2bc888ba2568cf06",
    pub summary: String, //"Add exception reporting events",
    pub message: String, //"Add exception reporting events"
}

#[derive(Debug, Deserialize)]
pub struct Application {
    pub application: Info,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub name: String,   //"paper-1.8.8-443.jar",
    pub sha256: String, //"621649a139ea51a703528eac1eccac40a1c8635bc4d376c05e248043b23cb3c3"
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let root = Root::new().await?;
    let projects = &root.projects;
    for p in projects {
        println!("{p}");
    }

    let paper = root.get_project("paper").await?;
    for v in &paper.versions {
        println!("{}\t{}", paper.project_id, v);
    }

    let latest_version = paper.get_latest_version().await?;
    for b in &latest_version.builds {
        println!(
            "{}\t{}\t{}",
            latest_version.project_id, latest_version.version, b
        );
    }

    let latest_build = latest_version.get_latest_build().await?;
    println!(
        "{}\t{}\t{}\t{}",
        latest_build.project_id,
        latest_build.version,
        latest_build.build,
        latest_build.download_link()
    );

    let path: &'static str = "./target.jar";
    latest_build.download(path).await?;
    assert!(latest_build.check_sum(path).await?);

    Ok(())
}
