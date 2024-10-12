use anyhow::Result;
use tokio;
use fetch_paper_lib::Root;

#[tokio::main]
pub async fn main()-> Result<()>{
    let download_path = "/tmp/target.jar";

    let root = Root::new().await?;

    let paper = root.get_project("paper").await?;

    let latest_version= paper.get_latest_version().await?;
    let app = latest_version.get_latest_build().await?;
    app.download(download_path).await?;
    assert!(app.checksum(download_path).await?);

    let download_path_1165 = "/tmp/target-1165.jar";
    let version_1165= paper.get_version("1.16.5").await?;
    let app_1165= version_1165.get_latest_build().await?;
    app_1165.download(download_path_1165).await?;
    assert!(app_1165.checksum(download_path_1165).await?);

    Ok(())
}
