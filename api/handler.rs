use std::io::Cursor;
use std::path::Path;
use tar::Archive;
use tokio::fs;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

// pub static CJLINT_TAR
include!(env!("CJLINT_DATA_FILE"));

async fn ensure_cjlint_extracted() -> Result<(), std::io::Error> {
    let target_dir = Path::new("/tmp/cjbind");
    

    if !target_dir.exists() {
        fs::create_dir_all(target_dir).await?;
        
        let cursor = Cursor::new(CJLINT_TAR);
        let mut archive = Archive::new(cursor);
        
        archive.unpack(target_dir)?;
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

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"message":"Hello from Rust!"}"#))?)
}