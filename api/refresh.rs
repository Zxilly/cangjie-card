use std::io::Cursor;
use std::path::Path;
use tar::Archive;
use tokio::fs;
use std::os::unix::fs::PermissionsExt;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

async fn ensure_cjlint_extracted() -> Result<(), std::io::Error> {
    let target_dir = Path::new("/tmp/cj");
    

    if !target_dir.exists() {
        let CJLINT_TAR = include!(env!("CJLINT_DATA_FILE"));
        
        fs::create_dir_all(target_dir).await?;
        
        let cursor = Cursor::new(CJLINT_TAR);
        let mut archive = Archive::new(cursor);
        
        archive.unpack(target_dir)?;

        let cjlint_path = target_dir.join("cjlint");
        let mut perms = fs::metadata(&cjlint_path).await?.permissions();
        perms.set_mode(0o755);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    if let Err(e) = ensure_cjlint_extracted().await {
        eprintln!("Failed to extract cjlint: {}", e);
        return Err(Error::from(e));
    }
    
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"message":"Hello from Rust!"}"#))?)
}