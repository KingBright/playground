# **核心架构文档二：仿真引擎 (Agent Playground)**

**核心定位**：基于知识的**运行时环境 (Runtime)**。它是一个通用的、基于 Agent 的状态机引擎。设计灵感源自 ECS (Entity-Component-System) 游戏架构。

## **1\. 核心概念模型**

Playground 由四个核心要素组成：**Environment (容器)**、**Local Agent (原生角色)**、**Universal Agent (通用服务)**、**Workflow (逻辑)**。

### **1.1 Environment (环境)：三位一体的容器**

环境不仅仅是背景，它是模拟的“物理法则”和“状态仓库”。

* **State Schema (数据定义)**：  
  * 定义环境包含哪些数据。例如：ChessBoard (8x8 数组), StockMarket (K线列表), DebateRoom (发言历史)。  
  * **关键点**：所有进入环境的数据（包括外脑注入的数据）必须符合 Schema。  
* **Validator Helpers (规则校验)**：  
  * 一组纯函数（Rust/Wasm 实现），用于判断行为合法性。  
  * 例如：isValidMove(board, move), isMarketOpen(time)。  
  * **注意**：环境不主动拒绝，而是提供校验方法供 Workflow 调用。  
* **Renderer Script (渲染逻辑)**：  
  * 定义如何将 State 可视化。是一段沙箱化的 JS 代码。

### **1.2 Agent Taxonomy (智能体分类学)**

为了支持复用和分工，我们将 Agent 分为两类：

#### **A. 环境原生 Agent (Env-Native Agents / Local Agents)**

* **定义**：为了特定环境而生，离开该环境无法生存。  
* **特性**：  
  * **强耦合**：必须严格遵守当前 Environment 的 Action Schema。  
  * **角色化**：通常扮演模拟中的具体角色（如：棋手、辩手、交易员）。  
  * **生命周期**：随 Session 创建和销毁。  
* **示例**：ChessPlayer, MarketTrader。

#### **B. 通用型 Agent (Universal Agents / Global Agents)**

* **定义**：功能导向，与特定环境解耦，可在不同环境间复用，甚至独立运行。  
* **特性**：  
  * **标准化接口**：输入输出遵循通用协议（如：Text/JSON），不依赖特定环境 Schema。  
  * **工具人属性**：通常拥有强大的通用能力（搜索、整理、编码、润色）。  
  * **服务化**：它们像“插件”一样被挂载到 Session 中，既可以被 Workflow 调用，也可以被 Local Agent 作为“Oracles”求助。  
* **示例**：  
  * KnowledgeLibrarian (知识整理者)：负责整理外脑知识。  
  * WebSearcher (搜索者)：实时联网搜索。  
  * FactChecker (事实核查员)：校验逻辑漏洞。

### **1.3 Workflow (工作流)：托管式逻辑脚本**

Workflow 不再是“一坨完整的代码”，而是由**可观测的节点 (Steps)** 组成的逻辑流。虽然用户以脚本形式编写，但系统通过特定的 API (step()) 将其解析为结构化的执行树。

* **职责**：  
  * **Orchestration (编排)**：定义步骤的顺序、循环和分支。  
  * **Observability (可观测)**：显式定义哪些操作是一个“节点”，以便系统追踪。  
  * **Safety (安全)**：运行在受限沙箱中，防止死循环或非法内存访问。  
* **技术选型**：推荐使用 **Rhai** (Rust 原生脚本) 或 **Lua**，嵌入在 Host 提供的沙箱中运行。

## **2\. 运行时架构 (Runtime Architecture)**

### **2.1 引擎核心 (Engine Core)**

采用 Rust 构建的高性能核心，维护 Session 的生命周期。

\[Host Process (Rust)\]  
   │  
   ├─ \[Sandbox Manager\] \<─── 进程/线程隔离边界  
   │     │  
   │     └─ \[Workflow VM (Rhai/Wasm)\]  
   │           │  
   │           ├─ Script Context (Variables)  
   │           └─ Execution Hooks (OnStepStart, OnStepEnd)  
   │  
   ├─ \[State Manager\] (Environment Data)  
   │     └─ Snapshots (用于异常回滚)  
   │  
   └─ \[Agent Runner\] (LLM Invocation Strategy)

### **2.2 跨 Agent 通信机制 (The Oracle Protocol)**

Local Agent 如何请求 Universal Agent 帮助？

* **机制**：Local Agent 在思考过程中，可以输出一个特殊的 ToolCall 或 HelpRequest。  
* **流程**：  
  1. **Local Agent** (e.g., 辩手) 发现自己缺乏论据。  
  2. **Output**: {"action": "SPEAK", "content": "...", "need\_help": {"target": "WebSearcher", "query": "2024 GDP Data"}}  
  3. **Workflow/Engine**: 拦截 need\_help 字段。  
  4. **Engine**: 暂停 Local Agent，调用挂载的 WebSearcher (Universal Agent)。  
  5. **WebSearcher**: 返回结果。  
  6. **Engine**: 将结果注入回 Local Agent 的 Context，Local Agent 重新生成最终发言。

### **2.3 Workflow 运行时沙箱与隔离 (Sandbox & Isolation)**

为了保证平台的稳定性，Workflow 必须运行在**轻量级沙箱**中。

1. **资源隔离**：  
   * **Instruction Limit**: 限制脚本的最大指令数，防止 while(true) 死循环卡死主进程。  
   * **Memory Quota**: 限制脚本可分配的内存上限。  
   * **Timeout Interrupt**: 单个 Step 执行超时的强制中断机制。  
2. **API 访问控制**：  
   * 脚本**不能**直接访问文件系统或网络。  
   * 所有 IO 操作（读写 Env、调用 Agent）必须通过 Host 暴露的 system.\* 或 env.\* 函数句柄进行。

### **2.4 节点级状态追踪与异常保护 (Step Tracking & Recovery)**

为了让 Workflow 可视化且健壮，我们引入**节点 (Step)** 概念。

1. **节点化执行 (Step-based Execution)**：  
   * 脚本不应该是一泻千里的，关键逻辑需包裹在 step("Name", || { ... }) 闭包中。  
   * **Host 监控**：每进入一个 step，Host 记录状态为 RUNNING，推送到前端 UI 高亮对应节点；完成后标记为 SUCCESS。  
2. **自动快照与回滚 (Auto-Checkpoint & Rollback)**：  
   * **机制**：在每个 step 开始前，引擎自动对 Environment State 进行序列化快照（Copy-on-Write）。  
   * **异常熔断**：如果某个 Step 抛出异常（如 Agent API 500 错误，或脚本逻辑错误）：  
     * 引擎捕获异常，暂停 Workflow。  
     * 状态标记为 PAUSED (ERROR)。  
     * 自动回滚 Env State 到该 Step 开始前的快照。  
   * **人工干预 (Human-in-the-loop)**：用户可以在 UI 上查看错误，**修改脚本中的该 Step 逻辑**，或**手动修改 Env 数据**，然后点击“重试 (Retry)”或“跳过 (Skip)”。

### **2.5 动态可视化系统**

* **前端沙箱**：UI 提供 Canvas 容器。  
* **数据驱动刷新**：Workflow 更新 State \-\> WebSocket 推送 \-\> JS 重绘。  
* **Workflow 状态面板**：实时显示当前执行的 Step 树状图、耗时、日志输出。

## **3\. 示例：下棋 Workflow (展示 Step 追踪与异常保护)**

// 场景：两人下棋  
// 使用 \`step\` 函数将逻辑分段，实现细粒度追踪和异常保护

let max\_turns \= 20;

// Step 1: 初始化  
// 系统会在进入此块前自动 Snapshot  
system.step("Initialization", || {  
    if env.get\_state("board") \== () {  
        env.update("board", "start\_position");  
    }  
    system.narrate("棋局开始。");  
});

for turn in 0..max\_turns {  
      
    // Step 2: 包装每一个回合为一个大节点  
    system.step(\`Turn\_${turn}\`, || {  
          
        // 子节点：红方思考  
        // 如果这里 LLM 调用超时失败，系统会暂停，状态回滚到 Turn 开始前  
        let move \= system.step("Red\_Think", || {  
            let obs \= \#{ board: env.get\_state("board") };  
            return agent("Player\_Red").act(obs);  
        });  
          
        // 子节点：规则校验  
        system.step("Validate\_Move", || {  
            if \!env.call("is\_valid\_move", move) {  
                // 抛出致命错误，触发熔断  
                throw \`红方违规移动: ${move}\`;  
            }  
            // 更新环境  
            env.update("board", move);  
        });  
          
        system.narrate(\`第 ${turn} 回合结束，红方落子 ${move}\`);  
    });  
}

// Step 3: 结算  
system.step("Finalize", || {  
    let winner \= env.call("check\_winner");  
    system.narrate(\`比赛结束，获胜者: ${winner}\`);  
});

## **4\. 关键技术总结**

1. **托管沙箱 (Managed Sandbox)**：利用 Rhai/Lua 的嵌入特性，通过 Host 严格控制脚本的 CPU/内存资源，实现进程内的逻辑隔离。  
2. **节点化追踪 (Step Instrumentation)**：通过 system.step 闭包，将线性脚本转化为可监控、可调试的执行树（Execution Tree）。  
3. **写时复制快照 (COW Snapshots)**：配合 Step 机制，实现颗粒度极细的“时光倒流”能力，将运行时错误的影响范围限制在单个节点内，提供企业级的稳定性兜底。