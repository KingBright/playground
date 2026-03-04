# **核心架构文档一：外脑系统 (External Brain)**

**核心定位**：平台的“感知器官”与“长期记忆”。它不仅是被动的数据存储，更是一个主动获取、清洗、结构化现实世界信息的**知识供应链 (Knowledge Supply Chain)**。

## **1\. 设计理念**

External Brain 的设计目标是解耦“知识获取”与“仿真模拟”。它独立于任何一次具体的模拟 Session 运行。

**关键演进**：外脑的维护工作（如清洗、整理、摘要）不再仅仅是硬编码的脚本（Pipeline），而是由**通用型 Agent (Universal Agents)** 驱动的。这意味着外脑本身就是一个“Agent 协作工场”。

## **2\. 系统架构层级**

### **2.1 数据采集层 (The Collector)**

负责连接外部数据源。

| 组件 | 说明 |
| :---- | :---- |
| **Connectors** | API, Crawler, RSS, File Upload |

### **2.2 认知加工层 (The Processor) \- Powered by Universal Agents**

这是将原始数据转化为知识的核心层。我们将特定任务委托给通用 Agent。

* **Dispatch Mechanism**: 当采集层有新数据时，任务被分发给特定的通用 Agent。  
* **Agent 角色**：  
  * **Cleaner Agent (清洗者)**：擅长 Regex 和格式化，负责去噪。  
  * **Extractor Agent (抽取者)**：擅长 NER（实体识别），负责从文本提取 {Entity, Relation}。  
  * **Summarizer Agent (摘要者)**：擅长长文本压缩。  
  * **Tagging Agent (打标者)**：负责分类和打标签，方便向量检索。

*注：这些 Universal Agents 与 Playground 中使用的是同一套代码库，只是配置不同。*

### **2.3 记忆存储层 (The Memory)**

采用多模态分层存储策略。

| 存储类型 | 技术选型 | 用途 |
| :---- | :---- | :---- |
| **Hot Memory** | Redis | 24小时热点数据。 |
| **Vector Memory** | Milvus/Qdrant | 语义向量索引。 |
| **Graph Memory** | Neo4j | 实体关系图谱。 |
| **Raw Archive** | S3/MinIO | 原始数据备份。 |

## **3\. 核心接口定义 (Memory API)**

External Brain 通过标准接口向 Playground 暴露能力。

### **3.1 供 Playground 调用的接口**

* GET /agent/invoke/{agent\_name}  
  * **新特性**：Playground 可以直接借用外脑的“算力”。例如，Playground 内部没有复杂的摘要能力，可以把文本发给外脑的 Summarizer Agent 处理。  
* GET /knowledge/search  
* GET /graph/explore

## **4\. 知识挂载机制 (Mounting)**

支持 **Knowledge Slice (知识切片)**。

* **Slice Definition**: {"time": "2025", "tag": "AI"}  
* **Mounting**: Session 启动时，Slice 被加载到通用 Agent（如 Librarian）的上下文中，使其回答更精准。

// 示例：外脑维护任务配置 (使用通用 Agent)  
{  
  "job": "daily\_arxiv\_cleanup",  
  "pipeline": \[  
    { "agent": "CleanerAgent", "input": "raw\_pdf\_text" },  
    { "agent": "SummarizerAgent", "input": "clean\_text" },  
    { "agent": "ExtractorAgent", "input": "summary" }  
  \]  
}  
