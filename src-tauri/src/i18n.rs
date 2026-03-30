/// 后端 i18n 辅助模块
/// 根据 locale 返回对应的模板文字

/// 判断是否为英文 locale
pub fn is_en(locale: &str) -> bool {
    locale.starts_with("en")
}

/// 日报标题
pub fn daily_report_title(locale: &str, date: &str) -> String {
    if is_en(locale) {
        format!("# Daily Report - {date}")
    } else {
        format!("# 工作日报 - {date}")
    }
}

/// 日报标题（带 emoji，云端模式）
pub fn daily_report_title_emoji(locale: &str, date: &str) -> String {
    if is_en(locale) {
        format!("# 📈 Daily Report - {date}")
    } else {
        format!("# 📈 工作日报 - {date}")
    }
}

/// 日报标题（无日期，混合模式）
pub fn daily_report_heading(locale: &str) -> &'static str {
    if is_en(locale) {
        "# Daily Report"
    } else {
        "# 工作日报"
    }
}

/// AI 系统提示词
pub fn ai_system_prompt(locale: &str) -> &'static str {
    if is_en(locale) {
        "You are a warm and professional daily report assistant who helps workers summarize their day. Your style is professional yet approachable, delivering valuable insights in an engaging way."
    } else {
        "你是一个充满人情味的工作日报助手，专门帮助打工人总结工作。你的风格是专业但不死板，能用轻松有趣的方式传递有价值的信息。"
    }
}

/// AI 生成日报的用户提示词后缀
pub fn ai_report_instruction(locale: &str) -> &'static str {
    if is_en(locale) {
        "Based on the data above, generate a valuable daily work report in your own style."
    } else {
        "请根据以上数据，用你自己的风格生成一份有价值的工作日报。"
    }
}

/// 桌宠气泡：开始生成日报
pub fn avatar_report_start(locale: &str) -> &'static str {
    if is_en(locale) {
        "Working on your daily report, one moment..."
    } else {
        "开始整理日报，稍等我一下。"
    }
}

/// 桌宠气泡：日报完成
pub fn avatar_report_done(locale: &str) -> &'static str {
    if is_en(locale) {
        "Your daily report is ready, come take a look."
    } else {
        "日报整理好了，可以回来看看。"
    }
}

/// 桌宠气泡：日报失败
pub fn avatar_report_failed(locale: &str) -> &'static str {
    if is_en(locale) {
        "Report generation failed this time, you can try again later."
    } else {
        "这次日报整理失败了，稍后可以再试。"
    }
}

/// 日报导出错误：未配置导出目录
pub fn export_no_dir(locale: &str) -> &'static str {
    if is_en(locale) {
        "Please select an export directory, or configure the daily report Markdown export directory in Settings"
    } else {
        "请先选择导出目录，或在设置中配置日报 Markdown 导出目录"
    }
}

/// 日报导出错误：未找到可导出的日报
pub fn export_no_report(locale: &str) -> &'static str {
    if is_en(locale) {
        "No report found to export"
    } else {
        "未找到可导出的日报"
    }
}

/// Ask 模块：日报来源标签
pub fn source_daily_report(locale: &str) -> &'static str {
    if is_en(locale) {
        "Daily Report"
    } else {
        "日报"
    }
}

/// 数据库：日报标题格式
pub fn db_report_title(locale: &str, date: &str) -> String {
    if is_en(locale) {
        format!("{date} Daily Report")
    } else {
        format!("{date} 日报")
    }
}
