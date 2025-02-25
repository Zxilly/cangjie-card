use octocrab::Octocrab;
use std::env;
use std::fs;
use std::path::Path;
use std::io::Write;
use std::path::PathBuf;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let output_file = Path::new(&out_dir).join("cjlint.tar");
    let include_file = Path::new(&out_dir).join("cjlint_data.rs");

    if output_file.exists() && include_file.exists() {
        return Ok(());
    }

    let token = env::var("GH_TOKEN").expect("GH_TOKEN environment variable not set");

    // 使用 octocrab 创建客户端
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()?;

    // 获取特定版本的 release
    let release = octocrab
        .repos("ZxillyLib", "cangjie-card-bin")
        .releases()
        .get_by_tag("0.58.3")
        .await?;

    // 查找特定资源
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "cjlint.tar")
        .expect("Could not find cjlint.tar in release assets");

    let mut asset_stream = octocrab
        .repos("ZxillyLib", "cangjie-card-bin")
        .release_assets()
        .stream(asset.id.into_inner())
        .await?;


    let mut file = fs::File::create(&output_file)?;
    let mut bytes = Vec::new();

    while let Some(chunk) = asset_stream.next().await {
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);
    }
    file.write_all(&bytes)?;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let relative = output_file.strip_prefix(&manifest_dir).unwrap();

    let include_code = format!(
        "include_bytes_zstd::include_bytes_zstd!({:?}, 9)",
        relative.to_str().unwrap()
    );
    fs::write(&include_file, include_code)?;

    println!(
        "cargo:rustc-env=CJLINT_DATA_FILE={}",
        include_file.display()
    );

    Ok(())
}
