use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use fetch_paper_lib::Root;
use log::{error, info, warn};

#[derive(Parser)]
#[command(about)]
struct Cli {
    /// project_id
    project: String,

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
        return Err(anyhow!("file already exists"));
    }

    let root = Root::new().await?;
    if !root.projects.contains(&cli.project) {
        info!("available projects:");
        for p in root.projects {
            info!("\t{}", p)
        }
        return Err(anyhow!(format!("project <{}> not found", cli.project)));
    }

    let project = root.get_project(&cli.project).await?;
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

    warn!("start download >> {}", build.download_link());
    warn!("download path >> {}", cli.path.to_string_lossy());
    warn!("remote sha256 >> {}", build.download_digest_sha256());
    build.download(&cli.path).await?;
    info!("download done");

    if !cli.skip_checksum {
        info!("checking sha256 >> ");
        if build.checksum(&cli.path).await? {
            info!("pass");
        } else {
            error!("fail");
            return Err(anyhow!("sha256 check fail"));
        }
    }

    Ok(())
}
