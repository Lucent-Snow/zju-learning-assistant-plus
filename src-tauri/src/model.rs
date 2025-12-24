use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub save_path: String,
    pub to_pdf: bool,
    pub auto_download: bool,
    pub ding_url: String,
    pub auto_open_download_list: bool,
    pub tray: bool,
    pub max_concurrent_tasks: u32,
    pub auto_start: bool,
    pub enable_image_dedup: bool,  // 是否启用图片去重
    pub dedup_threshold: u32,      // 相似度阈值（汉明距离）
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Upload {
    pub id: i64,
    pub reference_id: i64,
    pub file_name: String,
    pub course_name: String,
    pub path: String, // actual save path is path + file_name
    pub size: u64,
}

#[derive(Clone, Serialize, Default)]
pub struct Progress {
    pub id: String,
    pub status: String,
    pub file_name: String,
    pub downloaded_size: u64,
    pub total_size: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Subject {
    pub course_id: i64,
    pub sub_id: i64,
    pub course_name: String,
    pub sub_name: String,
    pub lecturer_name: String,
    pub path: String, // actual save path is path + sub_name
    pub ppt_image_urls: Vec<String>,
}

/// 字幕下载请求
#[derive(Clone, Serialize, Deserialize)]
pub struct SubtitleRequest {
    pub sub_id: i64,
    pub course_name: String,
    pub sub_name: String,
    pub path: String,  // 保存路径，与 PPT 下载保持一致
}

/// 字幕下载结果
#[derive(Clone, Serialize, Deserialize)]
pub struct SubtitleDownloadResult {
    pub success: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}
