# Lux MCP with SeaORM & SvelteKit Architecture

## Overview

Modern architecture using:
- **SeaORM**: Type-safe ORM for Rust with async support
- **SvelteKit**: Full-stack framework for building the dashboard
- **PostgreSQL**: Primary database
- **tRPC or REST**: API communication between Rust backend and SvelteKit

## Tech Stack

### Backend (Rust)
```toml
# Cargo.toml
[dependencies]
# Database
sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
sea-orm-migration = "0.12"

# Web server for API
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# MCP + existing deps
# ...
```

### Frontend (SvelteKit)
```json
{
  "devDependencies": {
    "@sveltejs/adapter-auto": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "svelte": "^4.2.0",
    "vite": "^5.0.0"
  },
  "dependencies": {
    "@tanstack/svelte-query": "^5.0.0",
    "bits-ui": "^0.11.0",
    "clsx": "^2.0.0",
    "lucide-svelte": "^0.294.0",
    "mode-watcher": "^0.1.0",
    "tailwind-merge": "^2.0.0",
    "tailwindcss": "^3.3.0"
  }
}
```

## SeaORM Entity Structure

```rust
// src/entities/session.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub session_type: String,
    pub session_external_id: String,
    pub query: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub status: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::reasoning_step::Entity")]
    ReasoningSteps,
    #[sea_orm(has_many = "super::synthesis_state::Entity")]
    SynthesisStates,
    #[sea_orm(has_many = "super::session_model::Entity")]
    SessionModels,
}

impl Related<super::reasoning_step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReasoningSteps.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

## API Architecture

### 1. Hybrid Approach
- **MCP Server**: Continues to handle MCP protocol
- **HTTP API**: Axum server for SvelteKit dashboard
- **Shared Database**: Both access same PostgreSQL

### 2. API Endpoints (Axum)
```rust
// src/api/mod.rs
use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{get, post},
    Router,
};

pub fn create_router(db: DatabaseConnection) -> Router {
    Router::new()
        // Sessions
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/:id", get(get_session))
        .route("/api/sessions/:id/steps", get(get_session_steps))
        .route("/api/sessions/:id/synthesis", get(get_synthesis_evolution))
        
        // Analytics
        .route("/api/analytics/overview", get(get_analytics_overview))
        .route("/api/analytics/models", get(get_model_performance))
        
        // Real-time
        .route("/api/sessions/:id/subscribe", get(websocket_handler))
        
        .with_state(db)
}
```

## SvelteKit Dashboard Structure

```
lux-dashboard/
├── src/
│   ├── routes/
│   │   ├── +layout.svelte          # App shell with navigation
│   │   ├── +page.svelte            # Dashboard home
│   │   ├── sessions/
│   │   │   ├── +page.svelte        # Sessions list
│   │   │   └── [id]/
│   │   │       ├── +page.svelte    # Session detail view
│   │   │       └── +page.ts        # Load session data
│   │   ├── analytics/
│   │   │   └── +page.svelte        # Analytics dashboard
│   │   └── api/
│   │       └── [...path]/
│   │           └── +server.ts      # Proxy to Rust API
│   ├── lib/
│   │   ├── components/
│   │   │   ├── SessionList.svelte
│   │   │   ├── ReasoningStep.svelte
│   │   │   ├── SynthesisEvolution.svelte
│   │   │   ├── BiasAnalysis.svelte
│   │   │   └── ModelPerformance.svelte
│   │   ├── api.ts                  # API client
│   │   └── types.ts                # TypeScript types
│   └── app.html
├── static/
├── package.json
└── vite.config.ts
```

## Key SvelteKit Components

### Session List View
```svelte
<!-- src/routes/sessions/+page.svelte -->
<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { fetchSessions } from '$lib/api';
  import SessionList from '$lib/components/SessionList.svelte';
  
  const sessions = createQuery({
    queryKey: ['sessions'],
    queryFn: fetchSessions
  });
</script>

<div class="container mx-auto p-6">
  <h1 class="text-3xl font-bold mb-6">Reasoning Sessions</h1>
  
  {#if $sessions.isLoading}
    <p>Loading sessions...</p>
  {:else if $sessions.error}
    <p>Error: {$sessions.error.message}</p>
  {:else}
    <SessionList sessions={$sessions.data} />
  {/if}
</div>
```

### Real-time Session View
```svelte
<!-- src/lib/components/ReasoningStep.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { ReasoningStep } from '$lib/types';
  
  export let sessionId: string;
  let steps: ReasoningStep[] = [];
  let ws: WebSocket;
  
  onMount(() => {
    ws = new WebSocket(`ws://localhost:3001/api/sessions/${sessionId}/subscribe`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'new_step') {
        steps = [...steps, data.step];
      }
    };
    
    return () => ws.close();
  });
</script>

<div class="space-y-4">
  {#each steps as step}
    <div class="card p-4">
      <div class="flex justify-between mb-2">
        <span class="font-semibold">Step {step.step_number}</span>
        <span class="text-sm text-gray-500">{step.model_used}</span>
      </div>
      <div class="prose max-w-none">
        {@html step.content}
      </div>
    </div>
  {/each}
</div>
```

## Database Integration Flow

1. **MCP Request** → Create session in DB
2. **LLM Call** → Store raw response
3. **Parse Synthesis** → Update synthesis_states table
4. **WebSocket** → Notify dashboard subscribers
5. **SvelteKit** → Update UI in real-time

## Migration from Current Architecture

### Phase 1: Add SeaORM entities
```bash
# Generate entities from existing schema
sea-orm-cli generate entity -o src/entities
```

### Phase 2: Dual operation
- Keep in-memory for MCP responses
- Add DB writes in parallel
- No breaking changes

### Phase 3: API server
- Add Axum routes
- Serve alongside MCP server
- Same binary, different ports

### Phase 4: SvelteKit dashboard
- Start with read-only views
- Add real-time updates
- Gradually add features

## Benefits

1. **Type Safety**: SeaORM entities match DB schema
2. **Modern UI**: SvelteKit provides reactive, fast UI
3. **Real-time**: WebSockets for live updates
4. **Scalable**: Can separate API and MCP servers
5. **Maintainable**: Clear separation of concerns