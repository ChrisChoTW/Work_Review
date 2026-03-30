use crate::analysis::{append_custom_prompt_locale, generate_stats_summary_locale, Analyzer, GeneratedReport};
use crate::i18n;
use crate::database::{Activity, DailyStats};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use reqwest::Client;
use serde_json::json;
use std::path::Path;
use std::time::Duration;

/// 云端视觉分析器
/// 使用 GPT-4o 等云端视觉模型直接分析截图
pub struct CloudAnalyzer {
    api_key: String,
    model: String,
    custom_prompt: String,
    locale: String,
    client: Client,
}

impl CloudAnalyzer {
    pub fn new(api_key: &str, model: &str, custom_prompt: &str, locale: &str) -> Self {
        // 创建带超时配置的 HTTP 客户端
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // OpenAI API 超时设置为60秒
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            custom_prompt: custom_prompt.to_string(),
            locale: locale.to_string(),
            client,
        }
    }

    /// 分析单张截图
    async fn analyze_screenshot(&self, screenshot_path: &Path) -> Result<String> {
        // 读取截图并转换为 Base64
        let image_data = tokio::fs::read(screenshot_path).await?;
        let image_base64 = BASE64_STANDARD.encode(&image_data);

        // 调用 OpenAI Vision API
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "text",
                                "text": "请简要描述这张截图中的内容，用户正在做什么工作？请用中文回答，限制在50字以内。"
                            },
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": format!("data:image/png;base64,{}", image_base64),
                                    "detail": "low"
                                }
                            }
                        ]
                    }
                ],
                "max_tokens": 100,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Analysis(format!(
                "OpenAI Vision API 错误: {error_text}"
            )));
        }

        let result: serde_json::Value = response.json().await?;
        let insight = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("无法分析")
            .to_string();

        Ok(insight)
    }

    /// 生成最终报告
    async fn generate_final_report(
        &self,
        date: &str,
        stats: &DailyStats,
        insights: &[String],
    ) -> Result<String> {
        let stats_summary = generate_stats_summary_locale(stats, &self.locale);

        let insights_text = insights
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n");

        // 让 AI 自由发挥，不设置固定模板格式
        let prompt = append_custom_prompt_locale(
            format!(
            r#"以下是一位打工人今天的工作数据：

{stats_summary}

### 从屏幕截图中识别到的工作内容
{insights_text}

{}"#,
            i18n::ai_report_instruction(&self.locale)
        ),
            &self.custom_prompt,
            &self.locale,
        );

        let response = self.client
            .post(format!("https://api.openai.com/v1/chat/completions"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "system",
                        "content": i18n::ai_system_prompt(&self.locale)
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "max_tokens": 2000,
                "temperature": 0.8,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Analysis(format!("OpenAI API 错误: {error_text}")));
        }

        let result: serde_json::Value = response.json().await?;
        let report = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(format!("{}\n\n{report}", i18n::daily_report_title_emoji(&self.locale, date)))
    }
}

#[async_trait]
impl Analyzer for CloudAnalyzer {
    async fn generate_report(
        &self,
        date: &str,
        stats: &DailyStats,
        activities: &[Activity],
        screenshots_dir: &Path,
    ) -> Result<GeneratedReport> {
        if self.api_key.is_empty() {
            return Err(AppError::Analysis("OpenAI API Key 未配置".to_string()));
        }

        log::info!("开始云端视觉分析，共 {} 条活动记录", activities.len());

        // 采样分析截图（最多分析5张，因为云端API成本较高）
        let sample_size = std::cmp::min(activities.len(), 5);
        let step = if activities.len() > sample_size {
            activities.len() / sample_size
        } else {
            1
        };

        let mut insights = Vec::new();

        for (i, activity) in activities.iter().enumerate() {
            if i % step != 0 && insights.len() >= sample_size {
                continue;
            }

            let screenshot_path = screenshots_dir.join(&activity.screenshot_path);
            if screenshot_path.exists() {
                match self.analyze_screenshot(&screenshot_path).await {
                    Ok(insight) => {
                        insights.push(format!("[{}] {}", activity.app_name, insight));
                        log::debug!("分析截图成功: {}", activity.screenshot_path);
                    }
                    Err(e) => {
                        log::warn!("分析截图失败: {screenshot_path:?}, 错误: {e}");
                    }
                }
            }
        }

        // 生成最终报告
        Ok(GeneratedReport {
            content: self.generate_final_report(date, stats, &insights).await?,
            used_ai: true,
        })
    }
}
