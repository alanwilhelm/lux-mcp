-- Lux MCP PostgreSQL Schema
-- Tracks reasoning sessions, synthesis evolution, and metacognitive monitoring

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Sessions table (for all tool types)
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_type VARCHAR(50) NOT NULL, -- 'biased_reasoning', 'traced_reasoning', 'planner', 'chat'
    session_external_id VARCHAR(255) UNIQUE NOT NULL, -- e.g., 'bias_5c80349917708f6e'
    query TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) DEFAULT 'active', -- 'active', 'completed', 'failed'
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Models used in sessions
CREATE TABLE session_models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- 'primary', 'verifier', 'assistant'
    model_name VARCHAR(255) NOT NULL,
    model_provider VARCHAR(50), -- 'openai', 'openrouter'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Reasoning steps (for biased_reasoning and traced_reasoning)
CREATE TABLE reasoning_steps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    step_type VARCHAR(50) NOT NULL, -- 'query', 'reasoning', 'bias_analysis', 'synthesis', 'thought'
    content TEXT NOT NULL,
    raw_llm_response TEXT, -- Store full LLM response
    model_used VARCHAR(255),
    confidence_score FLOAT,
    clarity_score FLOAT,
    thinking_time_ms INTEGER,
    tokens_used INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    UNIQUE(session_id, step_number)
);

-- Synthesis evolution (for biased_reasoning)
CREATE TABLE synthesis_states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    step_number INTEGER, -- Which step triggered this update
    current_understanding TEXT,
    confidence_score FLOAT DEFAULT 0.0,
    clarity_score FLOAT DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    raw_update_call TEXT, -- The actual update_synthesis() call
    parsed_data JSONB DEFAULT '{}'::jsonb, -- Parsed pros, cons, etc.
    UNIQUE(session_id, version)
);

-- Key insights
CREATE TABLE insights (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    synthesis_state_id UUID REFERENCES synthesis_states(id) ON DELETE CASCADE,
    insight_text TEXT NOT NULL,
    confidence FLOAT,
    source_step INTEGER,
    supported_by_evidence BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Action items
CREATE TABLE action_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    synthesis_state_id UUID REFERENCES synthesis_states(id) ON DELETE CASCADE,
    action_text TEXT NOT NULL,
    priority VARCHAR(20), -- 'high', 'medium', 'low'
    rationale TEXT,
    dependencies TEXT[],
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Bias detections
CREATE TABLE bias_detections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    has_bias BOOLEAN NOT NULL,
    severity VARCHAR(20), -- 'critical', 'high', 'medium', 'low', 'none'
    bias_types TEXT[],
    suggestions TEXT[],
    confidence FLOAT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Metacognitive monitoring events
CREATE TABLE monitoring_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    step_number INTEGER,
    event_type VARCHAR(50), -- 'circular_reasoning', 'distractor_fixation', 'quality_degradation'
    severity VARCHAR(20),
    intervention_message TEXT,
    pattern_data JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Chat/confer conversations
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL, -- 'user', 'assistant', 'system'
    content TEXT NOT NULL,
    model_used VARCHAR(255),
    response_time_ms INTEGER,
    tokens_used JSONB, -- {prompt: x, completion: y, total: z}
    temperature FLOAT,
    max_tokens INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Planning steps (for planner tool)
CREATE TABLE planning_steps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    is_revision BOOLEAN DEFAULT false,
    revises_step_number INTEGER,
    is_branch BOOLEAN DEFAULT false,
    branch_from_step INTEGER,
    branch_id VARCHAR(255),
    model_used VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(session_id, step_number)
);

-- Indexes for performance
CREATE INDEX idx_sessions_type ON sessions(session_type);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created ON sessions(created_at DESC);
CREATE INDEX idx_reasoning_steps_session ON reasoning_steps(session_id, step_number);
CREATE INDEX idx_synthesis_states_session ON synthesis_states(session_id, version);
CREATE INDEX idx_bias_detections_session ON bias_detections(session_id, step_number);
CREATE INDEX idx_monitoring_events_session ON monitoring_events(session_id, created_at);

-- Views for easier querying
CREATE VIEW session_summaries AS
SELECT 
    s.id,
    s.session_type,
    s.session_external_id,
    s.query,
    s.status,
    s.created_at,
    s.updated_at,
    s.completed_at,
    COUNT(DISTINCT rs.id) as total_steps,
    MAX(rs.step_number) as latest_step,
    AVG(rs.confidence_score) as avg_confidence,
    MAX(ss.version) as synthesis_versions,
    COUNT(DISTINCT bd.id) FILTER (WHERE bd.has_bias = true) as biases_detected,
    COUNT(DISTINCT me.id) as monitoring_events
FROM sessions s
LEFT JOIN reasoning_steps rs ON s.id = rs.session_id
LEFT JOIN synthesis_states ss ON s.id = ss.session_id
LEFT JOIN bias_detections bd ON s.id = bd.session_id
LEFT JOIN monitoring_events me ON s.id = me.session_id
GROUP BY s.id;

-- Function to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();