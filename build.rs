use std::env;
use std::path::Path;
use std::fs;
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let output_file = Path::new(&out_dir).join("cjlint.tar");
    let include_file = Path::new(&out_dir).join("cjlint_data.rs");

    if output_file.exists() && include_file.exists() {
        return Ok(());
    }
    
    let token = env::var("GH_TOKEN").expect("GH_TOKEN environment variable not set");
    
    let url = "https://github.com/ZxillyLib/cangjie-card-bin/releases/download/0.58.3/cjlint.tar";
    
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("token {}", token))
        .send()
        .await?;
        
    if !response.status().is_success() {
        panic!("Failed to download file: {}", response.status());
    }
    
    let bytes = response.bytes().await?;
    fs::write(&output_file, &bytes)?;
    
    let include_code = format!(
        "pub static CJLINT_TAR: &[u8] = include_bytes_zstd::include_bytes_zstd!({:?}, 9);",
        output_file.to_str().unwrap()
    );
    fs::write(&include_file, include_code)?;
    
    println!("cargo:rustc-env=CJLINT_DATA_FILE={}", include_file.display());
    
    Ok(())
}
