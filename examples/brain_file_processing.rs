//! Brain系统示例 - 文件处理与Agent协作
//!
//! 本示例展示Brain系统的文件处理能力和Agent协作：
//! 1. 处理多种文件格式
//! 2. 使用不同的Agent处理器
//! 3. Agent组合与协作
//! 4. 批量处理

use brain::collectors::file_handler::{FileHandler, FileHandlerConfig};
use brain::processors::{
    cleaner::{CleanerAgent, CleanerConfig},
    extractor::{ExtractorAgent, ExtractorConfig},
    tagger::{TaggerAgent, TaggerConfig},
    summarizer::{SummarizerAgent, SummarizerConfig, SummaryStyle},
};
use common::Agent;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("📁 Brain系统文件处理示例");
    info!("===========================\n");

    let temp_dir = TempDir::new()?;

    // ========== 步骤1: 创建测试文件 ==========
    info!("📝 步骤1: 创建测试文件");

    let files = vec![
        (
            "article.txt",
            "这是一篇关于人工智能的文章。

人工智能（AI）是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统。

联系方式: contact@ai-research.org
访问: https://ai-research.org",
            "text/plain",
        ),
        (
            "news.md",
            "# 技术新闻

## Rust语言获得年度最佳编程语言

Rust因其**内存安全**和*高性能*特性，荣获2024年度最佳编程语言。

更多信息请访问: https://www.rust-lang.org",
            "text/markdown",
        ),
        (
            "data.json",
            r#"{
  "title": "量子计算进展",
  "content": "IBM发布新型量子处理器，拥有1000个量子比特。",
  "author": "tech@quantum.com",
  "url": "https://quantum.ibm.com"
}"#,
            "application/json",
        ),
        (
            "report.html",
            "<html>
<head><title>研究报告</title></head>
<body>
    <h1>气候变化影响评估</h1>
    <p>全球平均气温在过去100年中上升了1.1°C。</p>
    <p>联系人: research@climate.org</p>
</body>
</html>",
            "text/html",
        ),
    ];

    for (filename, content, _content_type) in &files {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content)?;
        info!("  ✓ 创建: {}", filename);
    }

    info!();

    // ========== 步骤2: 使用FileHandler处理文件 ==========
    info!("🔄 步骤2: 使用FileHandler处理文件");

    let file_handler = FileHandler::new(FileHandlerConfig {
        max_file_size: 1024 * 1024, // 1MB
        allowed_extensions: vec![
            "txt".to_string(),
            "json".to_string(),
            "html".to_string(),
            "md".to_string(),
        ],
        enable_extraction: true,
    });

    let mut processed_results = Vec::new();

    for (filename, _, _) in &files {
        let file_path = temp_dir.path().join(filename);

        match file_handler.process_file(&file_path).await {
            Ok(result) => {
                info!("\n文件: {}", filename);
                info!("  大小: {} 字节", result.size);
                info!("  类型: {}", result.content_type);

                if let Some(content) = &result.content {
                    info!("  内容预览: {}", truncate(content, 60));
                }

                processed_results.push(result);
            }
            Err(e) => {
                info!("\n文件: {} - 处理失败: {}", filename, e);
            }
        }
    }

    info!("\n✓ 处理了 {} 个文件", processed_results.len());
    info!();

    // ========== 步骤3: 使用Cleaner Agent清洗数据 ==========
    info!("🧹 步骤3: 使用Cleaner Agent清洗数据");

    let cleaner = CleanerAgent::new(CleanerConfig {
        remove_html: true,
        normalize_whitespace: true,
        remove_special_chars: false,
        min_length: 10,
    });

    info!("\n清洗HTML内容:");
    let html_content = &processed_results[3].content.as_ref().unwrap();
    info!("原始: {}", truncate(html_content, 80));

    let input = common::AgentInput::new(serde_json::json!({
        "text": html_content
    }));

    match cleaner.invoke(input).await {
        Ok(output) => {
            let cleaned = output.data["text"].as_str().unwrap();
            info!("清洗: {}", truncate(cleaned, 80));
        }
        Err(e) => {
            info!("清洗失败: {}", e);
        }
    }

    info!();

    // ========== 步骤4: 使用Extractor Agent提取实体 ==========
    info!("⚙️  步骤4: 使用Extractor Agent提取信息");

    let extractor = ExtractorAgent::new(ExtractorConfig {
        extract_urls: true,
        extract_emails: true,
        extract_dates: true,
        extract_numbers: true,
    });

    let sample_text = "请联系support@example.com或admin@test.org。
        访问https://example.com了解详情。日期是2024-01-15。";

    let input = common::AgentInput::new(serde_json::json!({
        "text": sample_text
    }));

    match extractor.invoke(input).await {
        Ok(output) => {
            info!("\n文本: {}", sample_text);

            if let Some(entities) = output.data["entities"].as_array() {
                info!("\n提取的实体:");
                for entity in entities {
                    let text = entity["text"].as_str().unwrap();
                    let entity_type = entity["entity_type"].as_str().unwrap();
                    let confidence = entity["confidence"].as_f64().unwrap();

                    info!("  - {}: {} (置信度: {:.2})", entity_type, text, confidence);
                }
            }

            if let Some(relationships) = output.data["relationships"].as_array() {
                info!("\n提取的关系:");
                for rel in relationships.iter().take(3) {
                    let subject = rel["subject"].as_str().unwrap_or("");
                    let predicate = rel["predicate"].as_str().unwrap_or("");
                    let object = rel["object"].as_str().unwrap_or("");
                    info!("  - {} {} {}", subject, predicate, object);
                }
            }
        }
        Err(e) => {
            info!("提取失败: {}", e);
        }
    }

    info!();

    // ========== 步骤5: 使用Tagger Agent生成标签 ==========
    info!("🏷️  步骤5: 使用Tagger Agent生成标签");

    let tagger = TaggerAgent::new(TaggerConfig {
        max_tags: 5,
        min_confidence: 0.5,
        enable_classification: true,
    });

    let sample_text = "Rust是一种系统编程语言，注重安全、并发和性能";

    let input = common::AgentInput::new(serde_json::json!({
        "text": sample_text
    }));

    match tagger.invoke(input).await {
        Ok(output) => {
            info!("\n文本: {}", sample_text);

            if let Some(tags) = output.data["tags"].as_array() {
                info!("\n生成的标签:");
                for tag in tags {
                    info!("  - {}", tag.as_str().unwrap());
                }
            }
        }
        Err(e) => {
            info!("标签生成失败: {}", e);
        }
    }

    info!();

    // ========== 步骤6: 使用Summarizer Agent生成摘要 ==========
    info!("📋 步骤6: 使用Summarizer Agent生成摘要");

    let long_text = "人工智能是计算机科学的一个分支。它致力于创建能够执行通常需要人类智能的任务的系统。
        AI领域包括机器学习、深度学习、自然语言处理、计算机视觉等子领域。
        近年来，AI技术在图像识别、语音识别、自然语言理解等方面取得了重大突破。
        AI正在被应用于医疗、金融、交通、教育等多个行业。
        然而，AI也带来了伦理和隐私方面的挑战，需要谨慎对待。";

    info!("\n原文 ({} 字符):", long_text.len());
    info!("{}", long_text.replace("\n", " "));

    // 测试不同的摘要风格
    let styles = vec![
        (SummaryStyle::Paragraph, "段落式"),
        (SummaryStyle::BulletPoints, "要点式"),
        (SummaryStyle::Concise, "简洁式"),
    ];

    for (style, style_name) in styles {
        let summarizer = SummarizerAgent::new(SummarizerConfig {
            max_length: 100,
            style,
            num_bullets: 3,
        });

        let input = common::AgentInput::new(serde_json::json!({
            "text": long_text
        }));

        match summarizer.invoke(input).await {
            Ok(output) => {
                let summary = output.data["summary"].as_str().unwrap();
                info!("\n{}摘要:", style_name);
                info!("{}", summary);
            }
            Err(e) => {
                info!("摘要生成失败: {}", e);
            }
        }
    }

    info!();

    // ========== 步骤7: Agent组合示例 ==========
    info!("🤝 步骤7: Agent协作示例");

    let raw_html = "<article>
        <h1>Rust 1.75发布</h1>
        <p>Rust 1.75版本带来了许多新特性。访问 https://www.rust-lang.org 了解更多。</p>
        <p>联系邮箱: info@rust-lang.org</p>
    </article>";

    info!("\n原始HTML:");
    info!("{}", truncate(raw_html, 100));

    // 步骤7.1: Cleaner清洗
    let clean_input = common::AgentInput::new(serde_json::json!({
        "text": raw_html
    }));

    let cleaned_text = match cleaner.invoke(clean_input).await {
        Ok(output) => output.data["text"].as_str().unwrap().to_string(),
        Err(_) => raw_html.to_string(),
    };

    info!("\n清洗后:");
    info!("{}", truncate(&cleaned_text, 100));

    // 步骤7.2: Extractor提取实体
    let extract_input = common::AgentInput::new(serde_json::json!({
        "text": cleaned_text
    }));

    let entities = match extractor.invoke(extract_input).await {
        Ok(output) => {
            output.data["entities"].as_array()
                .map(|arr| arr.len())
                .unwrap_or(0)
        }
        Err(_) => 0,
    };

    info!("\n提取了 {} 个实体", entities);

    // 步骤7.3: Tagger生成标签
    let tag_input = common::AgentInput::new(serde_json::json!({
        "text": cleaned_text
    }));

    let tags = match tagger.invoke(tag_input).await {
        Ok(output) => {
            output.data["tags"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|t| t.as_str())
                    .collect::<Vec<_>>()
                )
                .unwrap_or_default()
        }
        Err(_) => vec![],
    };

    info!("生成标签: {:?}", tags);

    // 步骤7.4: Summarizer生成摘要
    let summarize_input = common::AgentInput::new(serde_json::json!({
        "text": cleaned_text
    }));

    match summarizer.invoke(summarize_input).await {
        Ok(output) => {
            let summary = output.data["summary"].as_str().unwrap();
            info!("生成摘要: {}", summary);
        }
        Err(e) => {
            info!("摘要生成失败: {}", e);
        }
    }

    info!();

    info!("🎉 文件处理示例完成!");
    info!("\n展示的功能:");
    info!("  ✓ 多格式文件处理");
    info!("  ✓ HTML内容清洗");
    info!("  ✓ 实体提取 (URL, 邮箱, 日期等)");
    info!("  ✓ 智能标签生成");
    info!("  ✓ 多种摘要风格");
    info!("  ✓ Agent链式协作");

    Ok(())
}

/// 截断文本
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}
