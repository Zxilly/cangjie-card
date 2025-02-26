use futures_util::StreamExt;
use octocrab::Octocrab;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

/// 从GitHub下载指定的资源文件
async fn download_github_asset(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    tag: &str,
    asset_name: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let release = octocrab
        .repos(owner, repo)
        .releases()
        .get_by_tag(tag)
        .await?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or(format!("Could not find {} in release assets", asset_name))?;

    let mut asset_stream = octocrab
        .repos(owner, repo)
        .release_assets()
        .stream(asset.id.into_inner())
        .await?;

    let mut file = fs::File::create(output_path)?;

    while let Some(chunk_result) = asset_stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk)?;
    }

    file.flush()?;

    Ok(())
}

fn generate_include_code(
    file_path: &Path,
    include_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let path_str = file_path.to_string_lossy().to_string();

    let include_code = format!("include_bytes!({:?})", path_str);

    fs::write(include_file, include_code)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let output_file = Path::new(&out_dir).join("cjlint.tar.zst");
    let include_file = Path::new(&out_dir).join("cjlint_data.rs");

    if let Ok(existing_data_path) = env::var("CJLINT_DATA_FILE") {
        let existing_path = Path::new(&existing_data_path);
        if existing_path.exists() {
            // 使用已有文件生成包含代码
            generate_include_code(existing_path, &include_file)?;
            println!(
                "cargo:rustc-env=CJLINT_DATA_FILE={}",
                include_file.display()
            );
            return Ok(());
        }
    }

    if output_file.exists() && include_file.exists() {
        println!(
            "cargo:rustc-env=CJLINT_DATA_FILE={}",
            include_file.display()
        );
        return Ok(());
    }

    let owner = "ZxillyLib";
    let repo = "cangjie-card-bin";
    let tag = "0.58.3";
    let asset_name = "cjlint.tar.zst";

    let token = env::var("GH_TOKEN").expect("GH_TOKEN environment variable not set");

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    download_github_asset(&octocrab, owner, repo, tag, asset_name, &output_file).await?;

    generate_include_code(&output_file, &include_file)?;

    println!(
        "cargo:rustc-env=CJLINT_DATA_FILE={}",
        include_file.display()
    );

    Ok(())
}
