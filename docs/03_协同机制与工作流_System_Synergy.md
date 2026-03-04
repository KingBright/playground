# **核心架构文档三：协同机制与业务场景 (System Synergy)**

**核心定位**：平台的“神经中枢”。它定义了 External Brain 和 Agent Playground 如何协同，并管理**通用 Agent (Universal Agents)** 的调度与分发。

## **1\. 任务调度中心 (Mission Control)**

这是连接“外脑”与“仿真引擎”的桥梁，同时也是 **Agent 的注册中心**。

### **1.1 Agent Registry (智能体市场)**

为了实现“通用 Agent”在不同环境（Env）和外脑（Brain）之间的复用，我们需要一个全局注册表。

* **Global Agent Store**:  
  * 存储通用 Agent 的定义（System Prompt, Tools, Config）。  
  * 示例：Searcher\_v1, PythonCoder\_v2, Librarian\_History\_Expert。  
* **Distribution (分发)**:  
  * 当外脑需要整理数据时，调度器实例化一个 Librarian。  
  * 当 Playground 需要搜索功能时，调度器将同一个 Librarian 挂载到 Simulation Session 中。

### **1.2 触发机制 (Triggers)**

1. **手动触发**: 用户点击 UI。  
2. **定时触发**: Cron Job (用于外脑的维护 Agent 定时上班)。  
3. **事件触发**: 数据变动触发。

## **2\. 三大核心业务场景流程 (整合通用 Agent)**

### **场景 A：定时新闻播报 (含通用 Agent 协助)**

**模式**：Brain Agents (整理) \-\> Playground (模拟)

1. **Phase 1: Knowledge Prep (在外脑)**  
   * **Trigger**: 6:30 AM。  
   * **Action**: 调度器唤醒通用 Agent NewsAggregator（新闻聚合者）。  
   * **Execution**: NewsAggregator 调用 API 抓取数据，调用 Summarizer 生成简报，存入 Hot Memory。  
2. **Phase 2: Simulation (在 Playground)**  
   * **Trigger**: 7:00 AM。  
   * **Initialization**: 启动演播室 Env。  
   * **Mounting**:  
     * 实例化 Local Agent: Host (主持人), Guest (嘉宾)。  
     * **挂载 Universal Agent**: FactChecker (事实核查员)。  
   * **Workflow**:  
     * 主持人播报（基于 Phase 1 的简报）。  
     * 嘉宾点评。  
     * **关键点**：如果嘉宾说错了年份，Workflow 自动调用 FactChecker 进行修正提示。

### **场景 B：知识库自动维护 (Knowledge Gardening)**

**模式**：Universal Agents 独立工作

1. **Trigger**: 每日 2:00 AM。  
2. **Task**: "整理昨日未处理的文档"。  
3. **Workflow (Maintenance)**:  
   * 调度器启动一组通用 Agent：Classifier, TagExtractor, RelationBuilder。  
   * 它们并发工作，将 Raw Data 转化为 Graph Data。  
   * **Result**: 纯后台处理，不涉及 Env 可视化。

### **场景 C：手动沙盘推演 (The Oracle Help)**

**模式**：User \-\> Env-Native Agent \-\> Universal Agent (Help)

1. **User Goal**: "模拟 1929 大萧条下，如果当时有现代量化交易策略会怎样？"  
2. **Setup**:  
   * Env: StockMarket\_Sim。  
   * Local Agent: Quant\_Bot (现代交易策略)。  
   * **Universal Agent**: HistoryOracle (挂载了 1929 年所有新闻数据的通用 Agent)。  
3. **Simulation Loop**:  
   * Quant\_Bot 需要决策，但它不知道 1929 年当天的具体情绪。  
   * Quant\_Bot 发起请求: Ask(HistoryOracle, "1929-10-24 当天早上的市场传言是什么？")。  
   * HistoryOracle 查询外脑记忆，返回："此时市场上流传着银行家正在开会救市..."。  
   * Quant\_Bot 基于此信息决定："买入"。  
   * **价值**：Env 原生 Agent 专注于策略逻辑，通用 Agent 专注于提供准确的背景知识。

## **3\. 总结：系统全景图**

graph TD  
    Registry\[Global Agent Registry\\n(通用 Agent 仓库)\]  
      
    subgraph External Brain \[外脑系统\]  
        RawData\[原始数据\]  
        Knowledge\[知识图谱\]  
          
        BrainWorkers\[维护型通用 Agent\\n(Cleaner, Extractor)\]  
        BrainWorkers \--\>|加工| RawData  
        BrainWorkers \--\>|写入| Knowledge  
    end  
      
    subgraph Agent Playground \[仿真引擎\]  
        Env\[Environment\]  
        LocalAgents\[内置 Agent\\n(Player, Trader)\]  
        MountedAgents\[挂载的通用 Agent\\n(Searcher, Oracle)\]  
          
        Workflow \--\>|控制| LocalAgents  
        LocalAgents \--\>|请求帮助| MountedAgents  
        MountedAgents \--\>|查阅| Knowledge  
    end  
      
    Registry \--\>|实例化| BrainWorkers  
    Registry \--\>|挂载| MountedAgents  
