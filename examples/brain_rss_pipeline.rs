//! Brain系统示例 - RSS新闻采集与处理流程
//!
//! 本示例展示如何使用Brain系统进行完整的知识处理流程：
//! 1. 从RSS源采集新闻数据
//! 2. 清洗HTML标签
//! 3. 提取实体和关系
//! 4. 生成标签和摘要
//! 5. 存储到多后端存储系统
//! 6. 进行语义搜索

use brain::collectors::rss_collector::{RssCollector, RssCollectorConfig};
use brain::processors::pipeline::{ProcessingPipeline, PipelineConfig};
use brain::storage::{
    hot_memory::InMemoryHotMemory,
    raw_archive::FileSystemRawArchive,
    unified_memory::UnifiedMemory,
    vector_memory::{InMemoryVectorStore, InMemoryVectorConfig},
    graph_memory::InMemoryGraphStore,
    RawData, DataSource, VectorDocument,
    HotMemoryBackend, VectorMemoryBackend, GraphMemoryBackend, RawArchiveBackend,
};
use common::{Agent, AgentInput};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::ExampleError>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Brain系统 RSS新闻处理流程示例");
    info!("=====================================\n");

    // 步骤1: 初始化存储后端
    info!("📦 步骤1: 初始化多后端存储系统");

    let temp_dir = TempDir::new()?;
    let storage_dir = temp_dir.path().join("brain_data");

    let hot_memory = Arc::new(InMemoryHotMemory::default()) as Arc<dyn HotMemoryBackend>;
    let vector_memory = Arc::new(InMemoryVectorStore::new(InMemoryVectorConfig::default()))
        as Arc<dyn VectorMemoryBackend>;
    let graph_memory = Arc::new(InMemoryGraphStore::new()) as Arc<dyn GraphMemoryBackend>;
    let raw_archive = Arc::new(FileSystemRawArchive::new(
        brain::storage::raw_archive::RawArchiveConfig {
            storage_dir: storage_dir.clone(),
            max_file_size: 10 * 1024 * 1024,
            compression: false,
        },
    ).await?) as Arc<dyn RawArchiveBackend>;

    let unified_memory = UnifiedMemory::new(hot_memory, vector_memory, graph_memory, raw_archive);

    info!("✓ 存储系统初始化完成\n");

    // 步骤2: 模拟RSS数据采集
    info!("📡 步骤2: 模拟RSS新闻采集");

    let sample_articles = vec![
        RawData::new(
            DataSource::RSS { url: "https://example.com/feed".to_string() },
            "<article>
                <h1>AI大模型技术突破：GPT-5即将发布</h1>
                <p>OpenAI宣布即将发布GPT-5模型，预计在推理能力和多模态理解方面有重大突破。
                该模型将支持更长的上下文窗口，达到100万token。</p>
                <p>更多信息请访问 https://openai.com/blog</p>
            </article>".to_string(),
            "text/html".to_string(),
        ),
        RawData::new(
            DataSource::RSS { url: "https://example.com/feed".to_string() },
            "<article>
                <h1>Rust编程语言进入TIOBE前十</h1>
                <p>Rust语言在最新的TIOBE编程语言排行榜中首次进入前十。
                这得益于其内存安全特性和高性能表现。</p>
                <p>联系邮箱: info@rust-lang.org</p>
            </article>".to_string(),
            "text/html".to_string(),
        ),
        RawData::new(
            DataSource::RSS { url: "https://example.com/feed".to_string() },
            "<article>
                <h1>量子计算新里程碑</h1>
                <p>IBM发布新型量子处理器，拥有1000个量子比特。
                这标志着量子计算向实用化迈出重要一步。</p>
            </article>".to_string(),
            "text/html".to_string(),
        ),
    ];

    info!("✓ 采集了 {} 篇文章\n", sample_articles.len());

    // 步骤3: 存储原始数据
    info!("💾 步骤3: 存储原始数据到归档");

    let mut raw_ids = Vec::new();
    for article in &sample_articles {
        let id = unified_memory.store_raw(article.clone()).await?;
        raw_ids.push(id);
        info!("  - 存储文章: {} (ID: {})",
            article.content.split('<').nth(1).unwrap_or("Unknown").split('>').nth(1).unwrap_or("Unknown"),
            id
        );
    }
    info!("✓ 原始数据存储完成\n");

    // 步骤4: 初始化处理流水线
    info!("🔄 步骤4: 初始化数据处理流水线");

    let pipeline_config = PipelineConfig {
        enable_cleaner: true,
        enable_tagger: true,
        enable_extractor: true,
        enable_summarizer: true,
        max_concurrent: 3,
    };

    let pipeline = ProcessingPipeline::new(pipeline_config);

    info!("✓ 流水线配置:");
    info!("  - 清洗器: ✓");
    info!("  - 标签器: ✓");
    info!("  - 提取器: ✓");
    info!("  - 摘要器: ✓");
    info!("  - 并发数: {}\n", 3);

    // 步骤5: 处理文章
    info!("⚙️  步骤5: 处理文章内容");

    for (i, article) in sample_articles.iter().enumerate() {
        info!("处理文章 {}/{}...", i + 1, sample_articles.len());

        match pipeline.process(&article.content).await {
            Ok(processed) => {
                info!("  ✓ 内容清洗: {} 字符", processed.content.len());
                info!("  ✓ 提取实体: {} 个", processed.entities.len());
                info!("  ✓ 生成标签: {:?}", processed.tags);

                if let Some(summary) = &processed.summary {
                    info!("  ✓ 生成摘要: {}", summary);
                }

                // 展示提取的实体
                for entity in processed.entities.iter().take(3) {
                    info!("    - {}: {} (置信度: {:.2})",
                        entity.entity_type, entity.text, entity.confidence);
                }
            }
            Err(e) => {
                info!("  ✗ 处理失败: {}", e);
            }
        }
        info!();
    }

    // 步骤6: 创建向量索引
    info!("🔍 步骤6: 创建语义搜索索引");

    let search_texts = vec![
        "人工智能和机器学习正在改变世界",
        "Rust是系统编程的现代选择",
        "量子计算将革命性改变计算能力",
    ];

    for (i, text) in search_texts.iter().enumerate() {
        let embedding = generate_mock_embedding(512);
        let doc = VectorDocument {
            id: format!("doc_{}", i),
            content: text.to_string(),
            embedding,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "article".to_string());
                meta
            },
        };

        unified_memory.store_vector(doc).await?;
    }

    info!("✓ 创建了 {} 个向量索引\n", search_texts.len());

    // 步骤7: 语义搜索示例
    info!("🎯 步骤7: 执行语义搜索");

    let query = "AI技术的发展";
    let query_embedding = generate_mock_embedding(512);

    let search_results = unified_memory.search_vector(&query_embedding, 3).await?;

    info!("查询: '{}'", query);
    info!("搜索结果:");
    for (i, result) in search_results.iter().enumerate() {
        info!("  {}. {} (相似度: {:.4})",
            i + 1, result.content, result.score);
    }
    info!();

    // 步骤8: 知识图谱构建
    info!("🕸️  步骤8: 构建知识图谱");

    use common::memory::{GraphNode, GraphEdge};

    // 添加实体节点
    let gpt_node = GraphNode {
        id: "gpt5".to_string(),
        labels: vec!["Model".to_string(), "AI".to_string()],
        properties: {
            let mut props = HashMap::new();
            props.insert("name".to_string(), serde_json::json!("GPT-5"));
            props.insert("developer".to_string(), serde_json::json!("OpenAI"));
            props
        },
    };

    let openai_node = GraphNode {
        id: "openai".to_string(),
        labels: vec!["Organization".to_string()],
        properties: {
            let mut props = HashMap::new();
            props.insert("name".to_string(), serde_json::json!("OpenAI"));
            props.insert("type".to_string(), serde_json::json!("AI Research Lab"));
            props
        },
    };

    unified_memory.add_graph_node(gpt_node).await?;
    unified_memory.add_graph_node(openai_node).await?;

    // 添加关系
    let developed_edge = GraphEdge {
        id: "edge1".to_string(),
        from: "openai".to_string(),
        to: "gpt5".to_string(),
        label: "DEVELOPED".to_string(),
        properties: HashMap::new(),
    };

    unified_memory.add_graph_edge(developed_edge).await?;

    info!("✓ 知识图谱构建完成");
    info!("  - 节点数: 2");
    info!("  - 关系数: 1\n");

    // 步骤9: 系统健康检查
    info!("🏥 步骤9: 系统健康检查");

    let health_status = unified_memory.health_check().await;

    info!("各组件状态:");
    for (component, healthy) in health_status.iter() {
        let status = if *healthy { "✓ 健康" } else { "✗ 异常" };
        info!("  - {}: {}", component, status);
    }
    info!();

    // 步骤10: 性能统计
    info!("📊 步骤10: 处理统计");

    info!("处理完成:");
    info!("  - 采集文章: {} 篇", sample_articles.len());
    info!("  - 处理文章: {} 篇", sample_articles.len());
    info!("  - 向量索引: {} 个", search_texts.len());
    info!("  - 图谱节点: {} 个", 2);
    info!("  - 图谱关系: {} 条", 1);
    info!();

    info!("🎉 Brain系统演示完成!");
    info!();
    info!("主要特性展示:");
    info!("  ✓ 多源数据采集 (RSS)");
    info!("  ✓ 多步骤数据清洗");
    info!("  ✓ 智能实体提取");
    info!("  ✓ 自动标签生成");
    info!("  ✓ 文本摘要生成");
    info!("  ✓ 多后端存储 (热存储/向量/图谱/归档)");
    info!("  ✓ 语义搜索");
    info!("  ✓ 知识图谱管理");

    Ok(())
}

/// 生成模拟的embedding向量
fn generate_mock_embedding(dim: usize) -> Vec<f32> {
    (0..dim)
        .map(|i| (i as f32 * 0.01).sin() as f32)
        .collect()
}
