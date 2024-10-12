use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use fetch_paper_lib::Root;

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
    let cli = Cli::parse();

    if cli.path.exists() {
        return Err(anyhow!("file already exists"));
    }

    let root = Root::new().await?;
    if !root.projects.contains(&cli.project) {
        println!("available projects:");
        for p in root.projects {
            println!("\t{}", p)
        }
        return Err(anyhow!(format!("project <{}> not found", cli.project)));
    }

    let project = root.get_project(&cli.project).await?;
    let version = if let Some(target_version) = cli.version {
        if project.versions.contains(&target_version) {
            project.get_version(&target_version).await?
        } else {
            println!("available versions: {:?}", project.versions);
            return Err(anyhow!(format!("version <{target_version}> not found")));
        }
    } else {
        project.get_latest_version().await?
    };

    let build = if let Some(target_build) = cli.build {
        if version.builds.contains(&target_build) {
            version.get_build(target_build).await?
        } else {
            println!("available builds: {:?}", version.builds);
            return Err(anyhow!(format!("build <{target_build}> not found")));
        }
    } else {
        version.get_latest_build().await?
    };

    println!("start download >> {}", build.download_link());
    println!("download path >> {}", cli.path.to_string_lossy());
    println!("remote sha256 >> {}", build.download_digest_sha256());
    build.download(&cli.path).await?;
    println!("download done");

    if !cli.skip_checksum {
        print!("checking sha256 >> ");
        if build.checksum(&cli.path).await? {
            print!("pass");
            println!();
        } else {
            print!("fail");
            println!();
            return Err(anyhow!("sha256 check fail"));
        }
    }

    Ok(())
}
