use cangjie_card::analysis::{process_analysis_result, run_cjlint};
use cangjie_card::models::{AnalysisResult, AnalysisResultItem, ApiResponse};
use cangjie_card::repository::{clone_repository, find_package_name, RepoCleanup};
use cangjie_card::storage::save_to_redis;
use cangjie_card::utils::ensure_cjlint_extracted;
use serde::Serialize;
use std::collections::HashMap;
use std::time::SystemTime;
use url::Url;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

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
