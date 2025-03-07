use rand::Rng;
use rand::distr::Alphanumeric;
use sysinfo::{System, MemoryRefreshKind};
use vercel_runtime::Error;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::{io::Cursor};
use tar::Archive;
use tokio::fs;
use zstd::stream::decode_all;

// 包含cjlint的二进制数据
static CJLINT_TAR_ZST: &'static [u8] = include!(env!("CJLINT_DATA_FILE"));

/// 生成一个指定长度的随机字符串
pub fn generate_random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// 获取当前内存使用情况
pub fn get_memory_usage() -> Result<String, Error> {
    let mut system = System::new();
    system.refresh_memory_specifics(MemoryRefreshKind::everything());
    
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let total_swap = system.total_swap();
    let used_swap = system.used_swap();
    
    let result = format!(
        "Memory: {:.2} GB / {:.2} GB (Used/Total)\n\
         Swap: {:.2} GB / {:.2} GB (Used/Total)\n\
         Memory Usage: {:.2}%",
        used_memory as f64 / 1024.0 / 1024.0,
        total_memory as f64 / 1024.0 / 1024.0,
        used_swap as f64 / 1024.0 / 1024.0,
        total_swap as f64 / 1024.0 / 1024.0,
        (used_memory as f64 / total_memory as f64) * 100.0
    );
    
    Ok(result)
}

/// 确保cjlint已经解压到指定目录
pub async fn ensure_cjlint_extracted() -> Result<(), std::io::Error> {
    let target_dir = Path::new("/tmp/cj");
    // /tmp/cj/tools/bin/cjlint
    let cjlint_path = target_dir.join("tools/bin/cjlint");

    if !target_dir.exists() || !cjlint_path.exists() {
        let cjlint_tar = decode_all(CJLINT_TAR_ZST.as_ref() as &[u8])?;

        fs::create_dir_all(target_dir).await?;

        let cursor = Cursor::new(cjlint_tar);
        let mut archive = Archive::new(cursor);
        archive.unpack(target_dir)?;

        eprintln!("cjlint_path: {:?}", cjlint_path);

        let mut perms = fs::metadata(&cjlint_path).await?.permissions();
        perms.set_mode(0o755);
    }

    Ok(())
} 