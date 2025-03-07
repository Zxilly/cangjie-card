use std::process::Command;
use tokio::fs;
use vercel_runtime::Error;
use crate::models::AnalysisResultItem;
use crate::utils::{generate_random_string, get_memory_usage};

/// 运行cjlint工具分析代码
pub async fn run_cjlint(repo_path: String) -> Result<String, Error> {
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

/// 处理分析结果，移除文件路径中的仓库路径前缀
pub fn process_analysis_result(
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