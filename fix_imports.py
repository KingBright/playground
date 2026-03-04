with open('crates/api/src/main.rs', 'r') as f:
    content = f.read()

# Fix imports at the top
content = content.replace('    extract::ws::{Message, WebSocket, WebSocketUpgrade},\n    body::Body,\n    extract::{Path, State, WebSocketUpgrade},', '    extract::ws::{Message, WebSocket, WebSocketUpgrade},\n    body::Body,\n    extract::{Path, State},')
content = content.replace('use serde_json::json;', 'use serde_json::json;\nuse sysinfo::System;')

# Fix imports in the middle
content = content.replace('use axum::extract::ws::{Message, WebSocket};\nuse tokio::time::{interval, Duration};', 'use tokio::time::{interval, Duration};')

# Fix json! macro and System import
content = content.replace('use clap::Parser;', 'use clap::Parser;\nuse serde_json::json;\nuse sysinfo::System;')

with open('crates/api/src/main.rs', 'w') as f:
    f.write(content)
