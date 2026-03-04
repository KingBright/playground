//! Brain系统示例 - 语义搜索与知识探索
//!
//! 本示例展示Brain系统的语义搜索能力：
//! 1. 存储不同类型的文档
//! 2. 生成向量embeddings
//! 3. 执行语义搜索
//! 4. 多维度过滤
//! 5. 知识图谱探索

use brain::storage::{
    vector_memory::{InMemoryVectorStore, InMemoryVectorConfig},
    graph_memory::InMemoryGraphStore,
    VectorDocument, SearchFilters,
};
use common::memory::{GraphNode, GraphEdge};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🔍 Brain系统语义搜索示例");
    info!("===========================\n");

    // 创建向量存储
    let vector_store = InMemoryVectorStore::new(InMemoryVectorConfig {
        dimension: 128, // 使用较小的维度用于演示
        similarity_threshold: 0.7,
        ..Default::default()
    });

    // 创建图存储
    let graph_store = InMemoryGraphStore::new();

    // ========== 步骤1: 存储文档 ==========
    info!("📚 步骤1: 存储文档到向量数据库");

    let documents = vec![
        (
            "人工智能正在改变医疗行业，AI辅助诊断准确率已达95%",
            vec!["人工智能", "医疗", "诊断"],
            "tech",
        ),
        (
            "Rust编程语言提供内存安全保证，无GC开销",
            vec!["rust", "编程", "内存安全"],
            "tech",
        ),
        (
            "量子计算机有望解决传统计算机无法处理的复杂问题",
            vec!["量子计算", "计算"],
            "tech",
        ),
        (
            "最新研究表明，地中海饮食有助于预防心血管疾病",
            vec!["健康", "饮食", "医疗"],
            "health",
        ),
        (
            "2024年奥运会将在巴黎举办，预计吸引全球数百万观众",
            vec!["体育", "奥运会"],
            "sports",
        ),
        (
            "气候变化导致极端天气事件频发，需要全球合作应对",
            vec!["气候", "环境"],
            "environment",
        ),
    ];

    for (i, (content, tags, category)) in documents.iter().enumerate() {
        let embedding = generate_embedding(content);

        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), category.to_string());
        metadata.insert("tags".to_string(), tags.join(","));
        metadata.insert("length".to_string(), content.len().to_string());

        let doc = VectorDocument {
            id: format!("doc_{}", i),
            content: content.to_string(),
            embedding,
            metadata,
        };

        vector_store.store(doc).await?;
        info!("  ✓ 存储: {} ({})", truncate(content, 40), category);
    }

    info!();

    // ========== 步骤2: 基础语义搜索 ==========
    info!("🔎 步骤2: 执行语义搜索");

    let queries = vec![
        ("AI技术在医疗领域的应用", "AI相关文档"),
        ("编程语言的安全性比较", "编程语言文档"),
        ("环境保护和可持续发展", "环境相关文档"),
    ];

    for (query, description) in &queries {
        let query_embedding = generate_embedding(query);
        let results = vector_store.search(&query_embedding, 3).await?;

        info!("\n查询: '{}' ({})", query, description);
        info!("结果:");
        for (i, result) in results.iter().enumerate() {
            let category = result.metadata.get("category").unwrap_or(&"unknown".to_string());
            info!("  {}. {} [相似度: {:.4}, 分类: {}]",
                i + 1,
                truncate(&result.content, 50),
                result.score,
                category
            );
        }
    }

    info!();

    // ========== 步骤3: 带过滤的搜索 ==========
    info!("🎯 步骤3: 使用元数据过滤");

    let query_embedding = generate_embedding("技术创新");
    let mut filters = SearchFilters::default();
    filters.metadata = Some(HashMap::from([
        ("category".to_string(), "tech".to_string()),
    ]));

    let results = vector_store
        .search_with_filters(&query_embedding, 10, Some(filters))
        .await?;

    info!("\n查询: '技术创新' (仅限技术类文档)");
    info!("结果:");
    for (i, result) in results.iter().enumerate() {
        info!("  {}. {} [相似度: {:.4}]",
            i + 1,
            truncate(&result.content, 50),
            result.score
        );
    }

    info!();

    // ========== 步骤4: 构建知识图谱 ==========
    info!("🕸️  步骤4: 构建知识图谱");

    // 添加实体节点
    let entities = vec![
        ("ai", "Technology", "人工智能", vec!["Technology", "AI"]),
        ("rust", "Language", "Rust语言", vec!["Language", "Programming"]),
        ("quantum", "Technology", "量子计算", vec!["Technology", "Computing"]),
        ("medical", "Field", "医疗", vec!["Field", "Healthcare"]),
        ("climate", "Issue", "气候变化", vec!["Issue", "Environment"]),
    ];

    for (id, label, name, labels) in entities {
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), serde_json::json!(name));

        let node = GraphNode {
            id: id.to_string(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            properties,
        };

        graph_store.add_node(node).await?;
        info!("  ✓ 添加实体: {}", name);
    }

    // 添加关系
    let relationships = vec![
        ("ai", "medical", "APPLIED_TO"),
        ("rust", "ai", "USED_IN"),
        ("quantum", "ai", "ENHANCES"),
        ("climate", "medical", "IMPACTS"),
    ];

    for (i, (from, to, label)) in relationships.iter().enumerate() {
        let edge = GraphEdge {
            id: format!("edge_{}", i),
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
            properties: HashMap::new(),
        };

        graph_store.add_edge(edge).await?;
        info!("  ✓ 添加关系: {} -[{}]-> {}", from, label, to);
    }

    info!();

    // ========== 步骤5: 图谱探索 ==========
    info!("🔬 步骤5: 知识图谱探索");

    let center_node = "ai";
    let (neighbors, edges) = graph_store.get_neighbors(center_node).await?;

    info!("\n探索 '{}' 的邻居节点:", center_node);
    for node in neighbors {
        let name = node.properties.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        info!("  - {} ({:?})", name, node.labels);
    }

    info!("\n相关关系:");
    for edge in edges {
        info!("  - {}", edge.label);
    }

    info!();

    // ========== 步骤6: 路径查找 ==========
    info!("🛤️  步骤6: 查找实体间路径");

    let path_queries = vec![
        ("rust", "medical"),
        ("quantum", "climate"),
    ];

    for (from, to) in path_queries {
        match graph_store.find_path(from, to).await {
            Ok(path) if !path.is_empty() => {
                info!("\n路径: {} -> {}", from, to);
                for (i, edge) in path.iter().enumerate() {
                    info!("  {}. {} -[{}]-> {}", i + 1, edge.from, edge.label, edge.to);
                }
            }
            _ => {
                info!("\n未找到路径: {} -> {}", from, to);
            }
        }
    }

    info!();

    // ========== 步骤7: 统计信息 ==========
    info!("📊 步骤7: 存储统计");

    let vector_count = vector_store.count().await?;
    let graph_stats = graph_store.stats().await?;

    info!("\n向量存储:");
    info!("  - 文档总数: {}", vector_count);

    info!("\n图谱存储:");
    info!("  - 节点总数: {}", graph_stats.node_count);
    info!("  - 关系总数: {}", graph_stats.edge_count);
    info!("  - 标签数: {}", graph_stats.label_counts.len());

    info!();

    info!("🎉 语义搜索示例完成!");
    info!("\n展示的功能:");
    info!("  ✓ 向量存储与检索");
    info!("  ✓ 语义相似度搜索");
    info!("  ✓ 元数据过滤");
    info!("  ✓ 知识图谱构建");
    info!("  ✓ 图谱邻域探索");
    info!("  ✓ 路径查找");

    Ok(())
}

/// 生成简单的embedding (基于字符的模拟)
fn generate_embedding(text: &str) -> Vec<f32> {
    let dim = 128;
    let bytes = text.as_bytes();

    (0..dim)
        .map(|i| {
            let byte_idx = i % bytes.len();
            let byte = bytes[byte_idx] as f32;
            let scale = (i as f32) / (dim as f32);
            (byte / 255.0) * scale.sin()
        })
        .collect()
}

/// 截断文本到指定长度
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}
