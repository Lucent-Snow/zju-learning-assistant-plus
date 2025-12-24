use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 单条字幕条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    /// 开始时间（秒）
    pub start_time: f64,
    /// 结束时间（秒）
    pub end_time: f64,
    /// 字幕文本（中文）
    pub text: String,
    /// 翻译文本（英文，可选）
    pub translation: Option<String>,
}

/// 字幕集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtitle {
    pub entries: Vec<SubtitleEntry>,
}

impl Subtitle {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 从智云课堂 API 返回的 JSON 解析字幕
    ///
    /// JSON 结构:
    /// {
    ///   "code": 0,
    ///   "msg": "获取同步文本成功",
    ///   "total": 1,
    ///   "list": [{
    ///     "all_content": [
    ///       { "BeginSec": 23, "EndSec": 30, "Text": "中文", "TransText": "English" },
    ///       ...
    ///     ]
    ///   }]
    /// }
    pub fn from_json(json: &serde_json::Value) -> Result<Self> {
        let mut entries = Vec::new();

        // 检查 API 返回状态
        if let Some(code) = json.get("code").and_then(|v| v.as_i64()) {
            if code != 0 {
                let msg = json.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                return Err(anyhow::anyhow!("API error: {}", msg));
            }
        }

        // 智云课堂结构: list[0].all_content[]
        if let Some(list) = json.get("list").and_then(|v| v.as_array()) {
            if let Some(first) = list.first() {
                if let Some(all_content) = first.get("all_content").and_then(|v| v.as_array()) {
                    for item in all_content {
                        let start = item.get("BeginSec").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let end = item.get("EndSec").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let text = item.get("Text").and_then(|v| v.as_str()).unwrap_or("");
                        let trans = item.get("TransText").and_then(|v| v.as_str()).unwrap_or("");

                        if !text.is_empty() {
                            entries.push(SubtitleEntry {
                                start_time: start,
                                end_time: end,
                                text: text.to_string(),
                                translation: if trans.is_empty() { None } else { Some(trans.to_string()) },
                            });
                        }
                    }
                }
            }
        }

        Ok(Self { entries })
    }

    /// 输出密集纯文本（无时间戳）
    pub fn to_plain_text(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.text.as_str())
            .collect::<Vec<_>>()
            .join("")
    }

    /// 输出带时间戳的纯文本
    pub fn to_timestamped_text(&self) -> String {
        self.entries
            .iter()
            .map(|e| {
                format!(
                    "[{} - {}] {}",
                    format_time_simple(e.start_time),
                    format_time_simple(e.end_time),
                    e.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 输出 SRT 格式字幕
    pub fn to_srt(&self) -> String {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                format!(
                    "{}\n{} --> {}\n{}\n",
                    i + 1,
                    format_time_srt(e.start_time),
                    format_time_srt(e.end_time),
                    e.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 输出 VTT 格式字幕
    pub fn to_vtt(&self) -> String {
        let mut result = String::from("WEBVTT\n\n");
        for (i, e) in self.entries.iter().enumerate() {
            result.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                format_time_vtt(e.start_time),
                format_time_vtt(e.end_time),
                e.text
            ));
        }
        result
    }

    /// 输出双语 SRT 格式字幕（中英对照）
    pub fn to_srt_bilingual(&self) -> String {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let trans = e.translation.as_deref().unwrap_or("");
                if trans.is_empty() {
                    format!(
                        "{}\n{} --> {}\n{}\n",
                        i + 1,
                        format_time_srt(e.start_time),
                        format_time_srt(e.end_time),
                        e.text
                    )
                } else {
                    format!(
                        "{}\n{} --> {}\n{}\n{}\n",
                        i + 1,
                        format_time_srt(e.start_time),
                        format_time_srt(e.end_time),
                        e.text,
                        trans
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 输出纯英文文本
    pub fn to_plain_text_english(&self) -> String {
        self.entries
            .iter()
            .filter_map(|e| e.translation.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("")
    }
}

/// 格式化时间为简单格式 (MM:SS)
fn format_time_simple(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// 格式化时间为 SRT 格式 (HH:MM:SS,mmm)
fn format_time_srt(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let secs = total_secs % 60;
    let total_mins = total_secs / 60;
    let mins = total_mins % 60;
    let hours = total_mins / 60;
    format!("{:02}:{:02}:{:02},{:03}", hours, mins, secs, ms)
}

/// 格式化时间为 VTT 格式 (HH:MM:SS.mmm)
fn format_time_vtt(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let secs = total_secs % 60;
    let total_mins = total_secs / 60;
    let mins = total_mins % 60;
    let hours = total_mins / 60;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time_simple(65.5), "01:05");
        assert_eq!(format_time_srt(3661.5), "01:01:01,500");
        assert_eq!(format_time_vtt(3661.5), "01:01:01.500");
    }

    #[test]
    fn test_subtitle_output() {
        let subtitle = Subtitle {
            entries: vec![
                SubtitleEntry {
                    start_time: 0.0,
                    end_time: 2.5,
                    text: "你好".to_string(),
                    translation: Some("Hello".to_string()),
                },
                SubtitleEntry {
                    start_time: 2.5,
                    end_time: 5.0,
                    text: "世界".to_string(),
                    translation: Some("World".to_string()),
                },
            ],
        };

        assert_eq!(subtitle.to_plain_text(), "你好世界");
        assert_eq!(subtitle.to_plain_text_english(), "HelloWorld");
        assert!(subtitle.to_srt().contains("1\n00:00:00,000 --> 00:00:02,500\n你好"));
        assert!(subtitle.to_srt_bilingual().contains("Hello"));
    }
}
