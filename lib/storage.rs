use redis::{Client, Commands};
use std::env;
use vercel_runtime::Error;

/// 将分析结果保存到Redis
pub async fn save_to_redis(repo: &str, content: &str) -> Result<(), Error> {
    let redis_url = env::var("KV_URL").map_err(|_| Error::from("KV_URL not set"))?;

    let client = Client::open(redis_url)
        .map_err(|e| Error::from(format!("Failed to create Redis client: {}", e)))?;

    let mut con = client.get_connection()?;

    let key = format!("cjlint_{}", repo);
    let _: () = con.set(key, content.to_string())?;

    Ok(())
} 