import re

with open('crates/api/src/main.rs', 'r') as f:
    content = f.read()

# Fix list_agents types
list_agents_new = """async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let active_agent_names = state.registry.list().await;
    let mut agents = Vec::new();

    for name in active_agent_names {
        if let Some(agent_def) = state.registry.get(&name).await {
            agents.push(Agent {
                id: name.clone(), // using name as id for simplicity since registry uses name as key
                name: agent_def.name.clone(),
                type_: format!("{:?}", agent_def.agent_type).to_lowercase(),
                description: agent_def.description.clone().unwrap_or_default(),
                capabilities: vec![], // Not stored in AgentDefinition directly
                status: "active".to_string(),
                version: "1.0.0".to_string(), // placeholder
                icon: Some("smart_toy".to_string()),
            });
        }
    }

    Json(AgentListResponse { agents })
}"""
content = re.sub(
    r'async fn list_agents\(State\(state\): State<AppState>\) -> impl IntoResponse \{[\s\S]*?Json\(AgentListResponse \{ agents \}\)\n\}',
    list_agents_new,
    content
)

with open('crates/api/src/main.rs', 'w') as f:
    f.write(content)
