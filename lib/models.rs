use serde::{Deserialize, Serialize};

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

// 定义一个结构体来存储克隆结果
#[derive(Debug, Clone)]
pub struct CloneResult {
    pub repo_path: String,
    pub commit_hash: String,
} 