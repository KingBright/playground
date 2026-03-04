//! Chess Game Workflow
//!
//! 模拟两个玩家下棋的workflow

// 初始化棋局
let board = env_get_state();
log_info("Starting chess game");

// 创建两个玩家
let white_player = agent_create("Alice");
let black_player = agent_create("Bob");

log_info("Players created");

// 模拟对局
let move_count = 0;
let max_moves = 10;

while move_count < max_moves {
    step_begin("white_move");
    log_info("White's turn");
    // 白方移动
    sleep_ms(100);
    move_count += 1;

    if move_count >= max_moves {
        break;
    }

    step_begin("black_move");
    log_info("Black's turn");
    // 黑方移动
    sleep_ms(100);
    move_count += 1;
}

// 游戏结束
log_info("Game completed");

// 请求Oracle分析棋局
let analysis = oracle_ask("Who is winning in this position?");
log_debug("Analysis complete");

// 返回结果
#{"winner": "draw", "moves": move_count}
