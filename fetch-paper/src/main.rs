use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use fetch_paper_lib::Root;
use log::{error, info, warn};

#[derive(Parser)]
#[command(about)]
struct Cli {
    /// project's name, leave blank to show all projects
    project: Option<String>,

    /// path to download file, default will use "./target.jar"
    #[arg(short, long, value_name = "VER", default_value = "./target.jar", value_hint = clap::ValueHint::FilePath)]
    path: PathBuf,

    /// version id, default will use latest
    #[arg(short, long, value_name = "VER")]
    version: Option<String>,

    /// build id, default will use latest
    #[arg(short, long, value_name = "BUILD")]
    build: Option<u16>,

    #[arg(long, action=ArgAction::SetTrue)]
    skip_checksum: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();

    let cli = Cli::parse();

    if cli.path.exists() {
        warn!("file already exists and would be truncated");
    }

    let root = Root::new().await?;
    info!("available projects:");
    for p in &root.projects {
        info!("\t{}", p)
    }

    if cli.project.is_none() {
        return Err(anyhow!("please choose one project"));
    }
    let project = cli.project.unwrap();

    if !root.projects.contains(&project) {
        return Err(anyhow!(format!("project <{}> not found", &project)));
    }

    let project = root.get_project(&project).await?;
    let version = if let Some(target_version) = cli.version {
        if project.versions.contains(&target_version) {
            project.get_version(&target_version).await?
        } else {
            info!("available versions: {:?}", project.versions);
            return Err(anyhow!(format!("version <{target_version}> not found")));
        }
    } else {
        project.get_latest_version().await?
    };

    let build = if let Some(target_build) = cli.build {
        if version.builds.contains(&target_build) {
            version.get_build(target_build).await?
        } else {
            info!("available builds: {:?}", version.builds);
            return Err(anyhow!(format!("build <{target_build}> not found")));
        }
    } else {
        version.get_latest_build().await?
    };

    warn!("download started>> {}", build.download_link());
    warn!("download path >> {}", cli.path.to_string_lossy());
    info!("remote sha256 >> {}", build.download_digest_sha256());
    build.download(&cli.path).await?;
    warn!("download done");

    if !cli.skip_checksum {
        info!("checking sha256");
        if build.checksum(&cli.path).await? {
            info!("check sha256 pass");
        } else {
            error!("check sha256 fail");
            return Err(anyhow!("sha256 check fail"));
        }
    }

    Ok(())
}
