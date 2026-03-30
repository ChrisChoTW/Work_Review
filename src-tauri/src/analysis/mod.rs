pub mod cloud;
pub mod hourly;
pub mod local;
pub mod summary;

use crate::config::{AiMode, AiProvider};
use crate::database::{Activity, DailyStats};
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GeneratedReport {
    pub content: String,
    pub used_ai: bool,
}

/// AI分析器 trait
/// 使用 async_trait 宏使 trait 支持 dyn 兼容
#[async_trait]
pub trait Analyzer: Send + Sync {
    /// 生成日报
    async fn generate_report(
        &self,
        date: &str,
        stats: &DailyStats,
        activities: &[Activity],
        screenshots_dir: &Path,
    ) -> Result<GeneratedReport>;
}

pub fn normalize_custom_prompt(custom_prompt: &str) -> Option<String> {
    let trimmed = custom_prompt.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn append_custom_prompt(base_prompt: String, custom_prompt: &str) -> String {
    append_custom_prompt_locale(base_prompt, custom_prompt, "zh-CN")
}

/// 支持 locale 的附加提示词拼接
pub fn append_custom_prompt_locale(base_prompt: String, custom_prompt: &str, locale: &str) -> String {
    if let Some(custom_prompt) = normalize_custom_prompt(custom_prompt) {
        if crate::i18n::is_en(locale) {
            format!(
                "{base_prompt}\n\n## Additional Requirements\nThe following are the user's supplementary preferences for the daily report. Please accommodate them without violating the structure and constraints above:\n{custom_prompt}"
            )
        } else {
            format!(
                "{base_prompt}\n\n## 额外要求\n以下是用户补充的日报偏好，请在不违背前述结构和约束的前提下尽量满足：\n{custom_prompt}"
            )
        }
    } else {
        base_prompt
    }
}

/// 创建分析器
pub fn create_analyzer(
    mode: AiMode,
    provider: AiProvider,
    endpoint: &str,
    model: &str,
    api_key: Option<&str>,
    custom_prompt: &str,
    locale: &str,
) -> Box<dyn Analyzer + Send + Sync> {
    match mode {
        AiMode::Local => Box::new(local::LocalAnalyzer::new(endpoint, model, custom_prompt, locale)),
        AiMode::Summary => Box::new(summary::SummaryAnalyzer::new(
            provider,
            endpoint,
            model,
            api_key,
            custom_prompt,
            locale,
        )),
        AiMode::Cloud => Box::new(cloud::CloudAnalyzer::new(
            api_key.unwrap_or(""),
            model,
            custom_prompt,
            locale,
        )),
    }
}

/// 格式化时长（秒 -> 可读字符串，精确到秒）
pub fn format_duration(seconds: i64) -> String {
    format_duration_locale(seconds, "zh-CN")
}

/// 格式化时长（支持 locale）
pub fn format_duration_locale(seconds: i64, locale: &str) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if crate::i18n::is_en(locale) {
        if hours > 0 {
            format!("{hours}h {minutes}m {secs}s")
        } else if minutes > 0 {
            format!("{minutes}m {secs}s")
        } else {
            format!("{secs}s")
        }
    } else {
        if hours > 0 {
            format!("{hours}小时{minutes}分{secs}秒")
        } else if minutes > 0 {
            format!("{minutes}分{secs}秒")
        } else {
            format!("{secs}秒")
        }
    }
}

/// 生成统计摘要
pub fn generate_stats_summary(stats: &DailyStats) -> String {
    generate_stats_summary_locale(stats, "zh-CN")
}

/// 生成统计摘要（支持 locale）
pub fn generate_stats_summary_locale(stats: &DailyStats, locale: &str) -> String {
    let mut summary = String::new();
    let en = crate::i18n::is_en(locale);

    summary.push_str(if en { "## Today's Work Statistics\n\n" } else { "## 今日工作统计\n\n" });
    summary.push_str(&format!(
        "- {}: {}\n",
        if en { "Total work duration" } else { "总工作时长" },
        format_duration_locale(stats.total_duration, locale)
    ));
    summary.push_str(&format!(
        "- {}: {}\n\n",
        if en { "Screenshots" } else { "截图数量" },
        stats.screenshot_count
    ));

    summary.push_str(if en { "### Application Usage Duration\n\n" } else { "### 应用使用时长\n\n" });
    for app in &stats.app_usage {
        summary.push_str(&format!(
            "- {}: {}\n",
            app.app_name,
            format_duration_locale(app.duration, locale)
        ));
    }

    summary.push_str(if en { "\n### Category Time Distribution\n\n" } else { "\n### 分类时间分布\n\n" });
    for cat in &stats.category_usage {
        let percentage = if stats.total_duration > 0 {
            (cat.duration as f64 / stats.total_duration as f64 * 100.0) as i32
        } else {
            0
        };
        summary.push_str(&format!(
            "- {}: {} ({}%)\n",
            crate::monitor::get_category_name_locale(&cat.category, locale),
            format_duration_locale(cat.duration, locale),
            percentage
        ));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::{append_custom_prompt, normalize_custom_prompt};

    #[test]
    fn 空白附加提示词应被忽略() {
        assert_eq!(normalize_custom_prompt("   "), None);
    }

    #[test]
    fn 应将附加提示词追加到基础提示词末尾() {
        let prompt = append_custom_prompt("基础提示".to_string(), "输出偏正式一些");

        assert!(prompt.contains("基础提示"));
        assert!(prompt.contains("额外要求"));
        assert!(prompt.contains("输出偏正式一些"));
    }
}
