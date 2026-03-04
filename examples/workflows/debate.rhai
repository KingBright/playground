//! Debate Workflow
//!
//! 模拟辩论的workflow

// 初始化辩论
let topic = env_get_state();
log_info("Debate starting");

// 创建辩手
let pro_side = agent_create("Proponent");
let con_side = agent_create("Opponent");

log_info("Debaters created");

// 辩论轮次
let rounds = 3;

for round in 1..=rounds {
    step_begin("round_" + round);
    log_info("Round " + round);

    // 正方发言
    step_begin("pro_argument");
    sleep_ms(200);
    log_info("Proponent presented argument");

    // 反方发言
    step_begin("con_argument");
    sleep_ms(200);
    log_info("Opponent presented argument");

    // 请求Oracle评估本轮
    let eval = oracle_ask("Evaluate the arguments");
    log_debug("Round evaluation complete");
}

// 总结辩论
step_begin("summary");
log_info("Debate concluded");

// 请求Oracle总结
let summary = oracle_ask("Summarize the key points from both sides");

// 返回结果
#{"topic": "debate", "rounds": rounds, "completed": true}
