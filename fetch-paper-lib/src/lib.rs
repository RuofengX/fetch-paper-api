use std::path::Path;

use anyhow::{anyhow, Ok, Result};
use constcat;
use reqwest;
use serde::Deserialize;

use sha2::{Digest, Sha256};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

/// Official api url base.
pub const API_BASE: &'static str = "https://api.papermc.io/v2";

/// Everything start from here.
///
/// A root is a json response from papermc's official ci.
/// It contains different projects, sach as paper, velocity, etc.
///
/// You can easily read these project name(or project_id),
/// then use [`Self::get_project`] to get further.
#[derive(Debug, Deserialize)]
pub struct Root {
    /// a list contains all supported projects' name(or project_id)
    pub projects: Vec<String>,
}

impl Root {
    /// Get raw url link of ci root page
    pub const fn link() -> &'static str {
        constcat::concat!(API_BASE, "/projects")
    }

    /// Request the url then parse the response into [`Root`]
    pub async fn new() -> Result<Self> {
        Ok(reqwest::get(Root::link()).await?.json::<Self>().await?)
    }

    /// Return the given project's info.
    pub async fn get_project(&self, project: &str) -> Result<Project> {
        Project::new(project).await
    }
}

/// A specific project info
#[derive(Debug, Deserialize)]
pub struct Project {
    /// Project id that given by [`Root`], eg. "paper"
    pub project_id: String,
    /// Full name / Display name of this project, eg. "Paper"
    pub project_name: String,
    /// (No desc)
    pub version_groups: Vec<String>,
    /// All downloadable versions, eg. "1.16.5"
    pub versions: Vec<String>,
}

impl Project {
    /// Get raw url link of a project, giving a project_id
    pub fn link(project: &str) -> String {
        format!("{0}/{1}", Root::link(), project)
    }

    /// See [`Self::link`]
    pub async fn new(project: &str) -> Result<Self> {
        Ok(reqwest::get(Project::link(project))
            .await?
            .json::<Self>()
            .await?)
    }

    /// Return the given version's info.
    pub async fn get_version(&self, version: &str) -> Result<Version> {
        Ok(Version::new(&self.project_id, version).await?)
    }

    /// Return the latest version's info.
    ///
    /// It is assumed that latest version is the last item in the list.
    pub async fn get_latest_version(&self) -> Result<Version> {
        self.get_version(self.versions.last().ok_or(anyhow!("no version found"))?)
            .await
    }
}

/// A specific verison info
#[derive(Debug, Deserialize)]
pub struct Version {
    /// Project id that given by [`Root`], eg. "paper"
    pub project_id: String,
    /// Full name / Display name of this project, eg. "Paper"
    pub project_name: String,
    /// Version name, eg. "1.16.5"
    pub version: String,
    /// All downloadable builds, eg. 250
    pub builds: Vec<u16>,
}
impl Version {
    /// Get raw url link of a version, giving a project_id and a version number.
    pub fn link(project: &str, version: &str) -> String {
        format!("{0}/versions/{1}", Project::link(project), version)
    }

    /// See [`Self::link`]
    pub async fn new(project: &str, version: &str) -> Result<Self> {
        let link = Version::link(project, version);
        Ok(reqwest::get(link).await?.json::<Self>().await?)
    }

    /// Return the given build's info.
    pub async fn get_build(&self, build: u16) -> Result<Build> {
        Ok(Build::new(&self.project_id, &self.version, build).await?)
    }

    /// Return the latest build's info.
    pub async fn get_latest_build(&self) -> Result<Build> {
        self.get_build(*self.builds.last().ok_or(anyhow!("no builds found"))?)
            .await
    }
}

/// Same as [`Version`]
#[derive(Debug, Deserialize)]
pub struct Build {
    pub project_id: String,
    pub project_name: String,
    pub version: String,
    pub build: u16,
    pub time: String,    //"2016-02-29T01:43:34.279Z"
    pub channel: String, //"default"
    pub promoted: bool,
    pub changes: Vec<wrapper::BuildChange>,
    pub downloads: wrapper::Application,
}
impl Build {
    pub fn link(project: &str, version: &str, build: u16) -> String {
        format!("{0}/builds/{1}", Version::link(project, version), build)
    }

    pub async fn new(project: &str, version: &str, build: u16) -> Result<Self> {
        let link = Build::link(project, version, build);
        Ok(reqwest::get(link).await?.json::<Self>().await?)
    }

    /// Get the direct download link of this build.
    pub fn download_link(&self) -> String {
        format!(
            "{0}/downloads/{1}",
            Self::link(&self.project_id, &self.version, self.build),
            self.downloads.application.name
        )
    }

    /// Get the remote file sha256 digest.
    pub fn download_digest_sha256(&self) -> &str {
        &self.downloads.application.sha256
    }

    /// Download the file.
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

    /// Check file sum.
    pub async fn checksum(&self, path: impl AsRef<Path>) -> Result<bool> {
        fn sha256(path: impl AsRef<Path>) -> Result<String> {
            let mut file = std::fs::File::open(path)?;
            let mut hasher = Sha256::new();
            let _n = std::io::copy(&mut file, &mut hasher)?;
            let hash = hasher.finalize();

            Ok(format!("{:x}", hash))
        }
        let owned_path = path.as_ref().to_path_buf();
        let rtn = self.download_digest_sha256()
            == tokio::task::spawn_blocking(|| sha256(owned_path)).await??;
        Ok(rtn)
    }
}

/// Structs that help json parse.
pub mod wrapper {
    use super::*;
    /// (Json response wrapper component)
    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    pub struct BuildChange {
        commit: String,  //"a7b53030d943c8205513e03c2bc888ba2568cf06",
        summary: String, //"Add exception reporting events",
        message: String, //"Add exception reporting events"
    }

    /// (Json response wrapper component)
    #[derive(Debug, Deserialize)]
    pub struct Application {
        pub application: FileInfo,
    }

    /// (Json response wrapper component)
    #[derive(Debug, Deserialize)]
    pub struct FileInfo {
        pub name: String,   //"paper-1.8.8-443.jar",
        pub sha256: String, //"621649a139ea51a703528eac1eccac40a1c8635bc4d376c05e248043b23cb3c3"
    }
}

/// Download the file.
///
/// `download("/tmp/target.jar", "paper", Some("1.16.5"), None, true).await?;`
/// this will download papermc, version 1.16.5, with latest build (None means latest), and check download file's hash.
pub async fn download(
    path: impl AsRef<Path>,
    project: &str,
    version: Option<&str>,
    build: Option<u16>,
    checksum: bool,
) -> Result<()> {
    let root = Root::new().await?;
    let project = root.get_project(project).await?;
    let version = if let Some(version) = version {
        project.get_version(version).await?
    } else {
        project.get_latest_version().await?
    };
    let build = if let Some(build) = build {
        version.get_build(build).await?
    } else {
        version.get_latest_build().await?
    };
    build.download(&path).await?;
    if checksum {
        build.checksum(&path).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn test() -> Result<()> {
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
    assert!(latest_build.checksum(path).await?);

    Ok(())
}
