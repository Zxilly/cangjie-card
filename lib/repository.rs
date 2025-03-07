use git2::build::RepoBuilder;
use glob::glob;
use std::path::Path;
use tokio::fs;
use toml::Value;
use vercel_runtime::Error;
use crate::models::CloneResult;
use crate::utils::generate_random_string;

// 定义一个结构体用于自动清理仓库目录
pub struct RepoCleanup {
    pub repo_path: String,
    cleaned: bool,
}

impl RepoCleanup {
    pub fn new(repo_path: String) -> Self {
        Self {
            repo_path,
            cleaned: false,
        }
    }

    // 手动清理方法，如果需要提前清理
    pub async fn cleanup(&mut self) -> Result<(), Error> {
        if !self.cleaned {
            if let Err(e) = fs::remove_dir_all(&self.repo_path).await {
                eprintln!("Failed to remove repository directory: {}", e);
                return Err(Error::from(format!("Failed to remove repository directory: {}", e)));
            }
            self.cleaned = true;
        }
        Ok(())
    }
}

impl Drop for RepoCleanup {
    fn drop(&mut self) {
        if !self.cleaned {
            if let Err(e) = std::fs::remove_dir_all(&self.repo_path) {
                eprintln!("Failed to remove repository directory in drop: {}", e);
            } else {
                self.cleaned = true;
            }
        }
    }
}

/// 克隆仓库到临时目录
pub async fn clone_repository(repo_url: &str) -> Result<CloneResult, Error> {
    let random_suffix = generate_random_string(10);
    let repo_dir_name = format!("cjrepo_{}", random_suffix);
    let target_dir = Path::new("/tmp").join(&repo_dir_name);
    let target_dir_str = target_dir.to_string_lossy().to_string();

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).await?;
    }

    fs::create_dir_all(&target_dir).await?;

    let mut option = git2::FetchOptions::default();
    option.depth(1);
    let repo = RepoBuilder::new()
        .fetch_options(option)
        .clone(repo_url, &target_dir)?;

    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let hash = commit.id().to_string();

    Ok(CloneResult {
        repo_path: target_dir_str,
        commit_hash: hash,
    })
}

/// 从仓库中查找包名
pub async fn find_package_name(repo_path: String) -> Result<String, Error> {
    let pattern = format!("{}/**/cjpm.toml", repo_path);
    let paths: Vec<_> = glob(&pattern)
        .map_err(|e| Error::from(format!("Failed to read glob pattern: {}", e)))?
        .filter_map(Result::ok)
        .collect();

    if paths.is_empty() {
        return Err(Error::from("No cjpm.toml found"));
    }

    let content = fs::read_to_string(&paths[0])
        .await
        .map_err(|e| Error::from(format!("Failed to read cjpm.toml: {}", e)))?;

    let value: Value = toml::from_str(&content)
        .map_err(|e| Error::from(format!("Failed to parse TOML: {}", e)))?;

    let package_name = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| Error::from("package.name not found in cjpm.toml"))?;

    Ok(package_name.to_string())
} 