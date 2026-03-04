-- Agent Playground - PostgreSQL 初始化脚本

-- 创建扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- 用于文本搜索

-- 创建 schema
CREATE SCHEMA IF NOT EXISTS agent_platform;

-- 设置搜索路径
ALTER DATABASE agent_platform SET search_path TO agent_platform, public;

-- 创建基本表

-- Agent 定义表
CREATE TABLE IF NOT EXISTS agent_platform.agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    agent_type VARCHAR(50) NOT NULL,
    description TEXT,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 会话表
CREATE TABLE IF NOT EXISTS agent_platform.sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'idle',
    config JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP WITH TIME ZONE,
    ended_at TIMESTAMP WITH TIME ZONE
);

-- 任务表
CREATE TABLE IF NOT EXISTS agent_platform.missions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    target_agent VARCHAR(255) NOT NULL,
    trigger_type VARCHAR(50) NOT NULL,
    parameters JSONB DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    executed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT
);

-- 数据收集日志
CREATE TABLE IF NOT EXISTS agent_platform.collection_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source VARCHAR(255) NOT NULL,
    source_type VARCHAR(50) NOT NULL,
    items_count INTEGER DEFAULT 0,
    status VARCHAR(50) NOT NULL,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_agents_name ON agent_platform.agents(name);
CREATE INDEX IF NOT EXISTS idx_agents_type ON agent_platform.agents(agent_type);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON agent_platform.sessions(status);
CREATE INDEX IF NOT EXISTS idx_missions_status ON agent_platform.missions(status);
CREATE INDEX IF NOT EXISTS idx_missions_agent ON agent_platform.missions(target_agent);
CREATE INDEX IF NOT EXISTS idx_collection_logs_source ON agent_platform.collection_logs(source);
CREATE INDEX IF NOT EXISTS idx_collection_logs_created ON agent_platform.collection_logs(created_at);

-- 插入示例数据
INSERT INTO agent_platform.agents (name, agent_type, description, config)
VALUES
    ('cleaner', 'universal', 'Text cleaning agent', '{"capabilities": ["clean"]}'),
    ('extractor', 'universal', 'Entity extraction agent', '{"capabilities": ["extract"]}'),
    ('summarizer', 'universal', 'Text summarization agent', '{"capabilities": ["summarize"]}'),
    ('tagger', 'universal', 'Content tagging agent', '{"capabilities": ["tag"]}')
ON CONFLICT (name) DO NOTHING;

-- 创建更新触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 创建触发器
DROP TRIGGER IF EXISTS update_agents_updated_at ON agent_platform.agents;
CREATE TRIGGER update_agents_updated_at
    BEFORE UPDATE ON agent_platform.agents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
