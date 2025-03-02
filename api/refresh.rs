use git2::build::RepoBuilder;
use glob::glob;
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use std::{collections::HashMap, io::Cursor};
use tar::Archive;
use tokio::fs;
use toml::Value;
use url::Url;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use zstd::stream::decode_all;
use rand::Rng;
use rand::distr::Alphanumeric;
use sysinfo::{System, MemoryRefreshKind};

static CJLINT_TAR_ZST: &'static [u8] = include!(env!("CJLINT_DATA_FILE"));

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum DefectLevel {
    #[serde(rename = "MANDATORY")]
    Mandatory,
    #[serde(rename = "SUGGESTIONS")]
    Suggestions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResultItem {
    pub file: String,
    pub line: i32,
    pub column: i32,
    #[serde(rename = "endLine")]
    pub end_line: i32,
    #[serde(rename = "endColumn")]
    pub end_column: i32,
    #[serde(rename = "analyzerName")]
    pub analyzer_name: String,
    pub description: String,
    #[serde(rename = "defectLevel")]
    pub defect_level: DefectLevel,
    #[serde(rename = "defectType")]
    pub defect_type: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub cjlint: Vec<AnalysisResultItem>,
    pub created_at: i64,
    pub commit: String,
    pub package_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
    pub error: Option<String>,
}

fn create_response<T: Serialize>(
    status_code: StatusCode,
    success: bool,
    message: Option<&str>,
    data: Option<T>,
    error: Option<&str>,
) -> Result<Response<Body>, Error> {
    let response = ApiResponse {
        success,
        message: message.map(String::from),
        data,
        error: error.map(String::from),
    };

    let body = serde_json::to_string(&response)
        .map_err(|e| Error::from(format!("Failed to serialize response: {}", e)))?;

    Ok(Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))?)
}

/// 生成一个指定长度的随机字符串
fn generate_random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

// 定义一个结构体来存储克隆结果
#[derive(Debug, Clone)]
struct CloneResult {
    repo_path: String,
    commit_hash: String,
}

// 定义一个结构体用于自动清理仓库目录
struct RepoCleanup {
    repo_path: String,
    cleaned: bool,
}

impl RepoCleanup {
    fn new(repo_path: String) -> Self {
        Self {
            repo_path,
            cleaned: false,
        }
    }

    // 手动清理方法，如果需要提前清理
    async fn cleanup(&mut self) -> Result<(), Error> {
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

async fn ensure_cjlint_extracted() -> Result<(), std::io::Error> {
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

async fn clone_repository(repo_url: &str) -> Result<CloneResult, Error> {
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

async fn find_package_name(repo_path: String) -> Result<String, Error> {
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

/// 获取当前内存使用情况
fn get_memory_usage() -> Result<String, Error> {
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

async fn run_cjlint(repo_path: String) -> Result<String, Error> {
    let output_path = format!("/tmp/{}.json", generate_random_string(10));

    // 使用函数获取并打印当前内存占用
    match get_memory_usage() {
        Ok(mem_info) => {
            eprintln!("Current memory usage before running cjlint:\n{}", mem_info);
        },
        Err(e) => {
            eprintln!("Failed to get memory usage: {}", e);
        }
    }

    let output = Command::new("/tmp/cj/tools/bin/cjlint")
        .args(&["-f", &repo_path, "-r", "json", "-o", &output_path])
        .env("LD_LIBRARY_PATH", "/tmp/cj")
        .env("CANGJIE_HOME", "/tmp/cj")
        .output()
        .map_err(|e| Error::from(format!("Failed to execute cjlint: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined_output = format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);

    if !output.status.success() {
        return Err(Error::from(format!(
            "cjlint command failed with exit code: {}\n{}",
            output.status.code().unwrap_or(-1),
            combined_output
        )));
    }

    let json_content = match fs::read_to_string(&output_path).await {
        Ok(content) => content,
        Err(e) => {
            return Err(Error::from(format!(
                "Failed to read cjlint output: {}\n{}",
                e, combined_output
            )));
        }
    };

    if let Err(e) = fs::remove_file(&output_path).await {
        eprintln!("Warning: Failed to delete cjlint output file: {}", e);
    }

    Ok(json_content)
}

async fn save_to_redis(repo: &str, content: &str) -> Result<(), Error> {
    let redis_url = env::var("KV_URL").map_err(|_| Error::from("KV_URL not set"))?;

    let client = Client::open(redis_url)
        .map_err(|e| Error::from(format!("Failed to create Redis client: {}", e)))?;

    let mut con = client.get_connection()?;

    let key = format!("cjlint_{}", repo);
    let _: () = con.set(key, content.to_string())?;

    Ok(())
}

/// 处理分析结果，移除文件路径中的仓库路径前缀
fn process_analysis_result(
    analysis_result: Vec<AnalysisResultItem>,
    repo_path: &str,
) -> Vec<AnalysisResultItem> {
    let repo_path_with_slash = if repo_path.ends_with('/') {
        repo_path.to_string()
    } else {
        format!("{}/", repo_path)
    };
    
    analysis_result
        .into_iter()
        .map(|mut item| {
            if item.file.starts_with(&repo_path_with_slash) {
                item.file = item.file[repo_path_with_slash.len()..].to_string();
            } else if item.file.starts_with(repo_path) {
                item.file = item.file[repo_path.len()..].to_string();
                if item.file.starts_with('/') {
                    item.file = item.file[1..].to_string();
                }
            }
            item
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    eprintln!("Starting...");
    if let Err(e) = ensure_cjlint_extracted().await {
        eprintln!("Failed to extract cjlint: {}", e);
        return Err(Error::from(e));
    }
    eprintln!("cjlint extracted");

    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let url = Url::parse(&req.uri().to_string()).unwrap();
    let hash_query: HashMap<String, String> = url.query_pairs().into_owned().collect();
    let repo = hash_query.get("repo");
    let repo = match repo {
        Some(repo) => repo,
        None => {
            return create_response::<()>(
                StatusCode::BAD_REQUEST,
                false,
                None,
                None,
                Some("repo query parameter is required"),
            );
        }
    };

    let clone_result = match clone_repository(repo).await {
        Ok(result) => result,
        Err(e) => {
            return create_response::<()>(
                StatusCode::INTERNAL_SERVER_ERROR,
                false,
                None,
                None,
                Some(&format!("Failed to clone repository: {}", e)),
            );
        }
    };

    let mut repo_cleanup = RepoCleanup::new(clone_result.repo_path.clone());

    let package_name = match find_package_name(clone_result.repo_path.clone()).await {
        Ok(name) => name,
        Err(e) => {
            return create_response::<()>(
                StatusCode::INTERNAL_SERVER_ERROR,
                false,
                None,
                None,
                Some(&format!("Failed to find package name: {}", e)),
            );
        }
    };

    // 使用 cjlint 检查代码
    let content = match run_cjlint(clone_result.repo_path.clone()).await {
        Ok(result) => result,
        Err(e) => {
            return create_response::<()>(
                StatusCode::INTERNAL_SERVER_ERROR,
                false,
                None,
                None,
                Some(&format!("Failed to run cjlint: {}", e)),
            );
        }
    };

    let analysis_result: Vec<AnalysisResultItem> = match serde_json::from_str(&content) {
        Ok(result) => result,
        Err(e) => {
            return create_response::<()>(
                StatusCode::INTERNAL_SERVER_ERROR,
                false,
                None,
                None,
                Some(&format!("Failed to parse cjlint output: {}", e)),
            );
        }
    };

    // 处理file字段，去除repo_path前缀
    let repo_path = clone_result.repo_path.clone();
    let processed_analysis_result = process_analysis_result(analysis_result, &repo_path);

    let analysis_result = AnalysisResult {
        cjlint: processed_analysis_result,
        created_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        commit: clone_result.commit_hash,
        package_name,
    };

    // 将结果保存到Redis
    if let Err(e) = save_to_redis(repo, &serde_json::to_string(&analysis_result).unwrap()).await {
        return create_response::<()>(
            StatusCode::INTERNAL_SERVER_ERROR,
            false,
            None,
            None,
            Some(&format!("Failed to save to Redis: {}", e)),
        );
    }

    if let Err(e) = repo_cleanup.cleanup().await {
        eprintln!("Warning: Failed to clean up repository: {}", e);
    }

    return create_response(
        StatusCode::OK,
        true,
        Some("Analysis completed successfully"),
        Some(analysis_result),
        None,
    );
}
