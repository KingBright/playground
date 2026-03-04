import re

with open('crates/api/src/main.rs', 'r') as f:
    content = f.read()

missing_fields = """
        AppState {
            config: ServerConfig {
                bind: "0.0.0.0:8080".to_string(),
                static_dir: PathBuf::from("static"),
                api_only: true,
            },
            session_manager,
            brain_memory,
            registry: Arc::new(synergy::registry::AgentRegistry::new()),
            mission_control: Arc::new(synergy::scheduler::MissionControl::new(
                Arc::new(synergy::registry::AgentRegistry::new()),
                synergy::scheduler::SchedulerConfig::default()
            )),
        }
"""
content = re.sub(
    r'        AppState \{\n            config: ServerConfig \{\n                bind: "0\.0\.0\.0:8080"\.to_string\(\),\n                static_dir: PathBuf::from\("static"\),\n                api_only: true,\n            \},\n            session_manager,\n            brain_memory,\n        \}',
    missing_fields.strip(),
    content
)

with open('crates/api/src/main.rs', 'w') as f:
    f.write(content)
