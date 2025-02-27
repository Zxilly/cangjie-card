use git2::build::RepoBuilder;
use redis::{Client, Commands};
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::{collections::HashMap, io::Cursor};
use tar::Archive;
use tokio::fs;
use url::Url;
use zstd::stream::decode_all;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

static CJLINT_TAR_ZST: &'static [u8] = include!(env!("CJLINT_DATA_FILE"));

async fn ensure_cjlint_extracted() -> Result<(), std::io::Error> {
    let target_dir = Path::new("/tmp/cj");
    // /tmp/cj/tools/bin/cjlint
    let cjlint_path = target_dir.join("tools/bin/cjlint");

    if !target_dir.exists() || !cjlint_path.exists() {

        let cjlint_tar = decode_all(CJLINT_TAR_ZST.as_ref())?;

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

async fn clone_repository(repo_url: &str) -> Result<(), Error> {
    let target_dir = Path::new("/tmp/cjrepo");

    if target_dir.exists() {
        fs::remove_dir_all(target_dir).await?;
    }

    fs::create_dir_all(target_dir).await?;

    let mut option = git2::FetchOptions::default();
    option.depth(1);
    RepoBuilder::new()
        .fetch_options(option)
        .clone(repo_url, target_dir)?;

    Ok(())
}

async fn run_cjlint() -> Result<String, Error> {
    let output_path = "/tmp/cjlint.json";

    let status = Command::new("/tmp/cj/tools/bin/cjlint")
        .args(&[
            "-f",
            "/tmp/cjrepo",
            "-r",
            "json",
            "-o",
            output_path,
        ])
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
    let redis_url =
        env::var("KV_URL").map_err(|_| Error::from("KV_URL not set"))?;

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
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{"message":"repo query parameter is required"}"#,
                ))?)
        }
    };

    // 克隆仓库
    match clone_repository(repo).await {
        Ok(_) => (),
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(format!(
                    r#"{{"message":"Failed to clone repository: {}"}}"#,
                    e
                )))?)
        }
    }

    // 使用 cjlint 检查代码
    let content = match run_cjlint().await {
        Ok(result) => result,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(format!(
                    r#"{{"message":"Failed to run cjlint: {}"}}"#,
                    e
                )))?)
        }
    };

    // 将结果保存到Redis
    if let Err(e) = save_to_redis(repo, &content).await {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::from(format!(
                r#"{{"message":"Failed to save to Redis: {}"}}"#,
                e
            )))?);
    }

    return Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(content))?);
}
