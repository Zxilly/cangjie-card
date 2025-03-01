use git2::build::RepoBuilder;
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
use url::Url;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use zstd::stream::decode_all;
use glob::glob;
use toml::Value;

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

async fn clone_repository(repo_url: &str) -> Result<String, Error> {
    let target_dir = Path::new("/tmp/cjrepo");

    if target_dir.exists() {
        fs::remove_dir_all(target_dir).await?;
    }

    fs::create_dir_all(target_dir).await?;

    let mut option = git2::FetchOptions::default();
    option.depth(1);
    let repo = RepoBuilder::new()
        .fetch_options(option)
        .clone(repo_url, target_dir)?;

    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let hash = commit.id();

    Ok(hash.to_string())
}

async fn find_package_name() -> Result<String, Error> {
    let pattern = "/tmp/cjrepo/**/cjpm.toml";
    let paths: Vec<_> = glob(pattern)
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

async fn run_cjlint() -> Result<String, Error> {
    let output_path = "/tmp/cjlint.json";

    let status = Command::new("/tmp/cj/tools/bin/cjlint")
        .args(&["-f", "/tmp/cjrepo", "-r", "json", "-o", output_path])
        .env("LD_LIBRARY_PATH", "/tmp/cj")
        .env("CANGJIE_HOME", "/tmp/cj")
        .status()
        .map_err(|e| Error::from(format!("Failed to execute cjlint: {}", e)))?;

    if !status.success() {
        return Err(Error::from(format!(
            "cjlint command failed with exit code: {}",
            status.code().unwrap_or(-1)
        )));
    }

    let json_content = fs::read_to_string(output_path)
        .await
        .map_err(|e| Error::from(format!("Failed to read cjlint output: {}", e)))?;

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

    // 克隆仓库
    let commit = match clone_repository(repo).await {
        Ok(commit) => commit,
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

    let package_name = match find_package_name().await {
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
    let content = match run_cjlint().await {
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
    
    let analysis_result = AnalysisResult {
        cjlint: analysis_result,
        created_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        commit,
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

    return create_response(
        StatusCode::OK,
        true,
        Some("Analysis completed successfully"),
        Some(analysis_result),
        None,
    );
}
