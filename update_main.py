import re

with open('crates/api/src/main.rs', 'r') as f:
    content = f.read()

# Add sysinfo import
content = content.replace('use axum::{', 'use axum::{\n    extract::ws::{Message, WebSocket, WebSocketUpgrade},')
content = content.replace('use serde_json::json;', 'use serde_json::json;\nuse sysinfo::System;')

# Add system_websocket route
content = content.replace('.route("/ws/missions", get(missions_websocket))', '.route("/ws/missions", get(missions_websocket))\n        .route("/ws/system", get(system_websocket))')

# Add system_websocket handler
ws_handler = """
async fn system_websocket(
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_system_socket(socket))
}

async fn handle_system_socket(mut socket: WebSocket) {
    info!("WebSocket connected for system monitoring");
    let mut sys = System::new_all();
    let pid = sysinfo::get_current_pid().expect("failed to get PID");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

    loop {
        interval.tick().await;
        sys.refresh_all();

        let mut cpu_usage = 0.0;
        let mut memory_usage = 0;

        if let Some(process) = sys.process(pid) {
            cpu_usage = process.cpu_usage();
            memory_usage = process.memory(); // in bytes
        }

        let total_memory = sys.total_memory();
        let memory_percent = if total_memory > 0 {
            (memory_usage as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let msg = json!({
            "type": "system_stats",
            "data": {
                "cpu_usage": cpu_usage,
                "memory_usage_bytes": memory_usage,
                "memory_usage_percent": memory_percent,
                "total_memory_bytes": total_memory,
            }
        });

        if socket.send(Message::Text(msg.to_string())).await.is_err() {
            info!("System WebSocket client disconnected");
            break;
        }
    }
}
"""

content = content + ws_handler

with open('crates/api/src/main.rs', 'w') as f:
    f.write(content)
