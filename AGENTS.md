# Agent Instructions for Lux MCP

This file contains instructions for AI agents working with the Lux MCP codebase.

## Code Quality Commands

When asked to check code quality or before committing, run:

```bash
# Quick quality check
make check
# or
./check.sh

# Auto-fix issues
make fix

# Full CI checks (strict)
make ci
```

## Available Make Commands

```bash
make build    # Build debug version
make release  # Build release version  
make check    # Run all quality checks
make fmt      # Format code
make clippy   # Run clippy lints
make test     # Run tests
make clean    # Clean build artifacts
make run      # Build and run server
make install  # Install to ~/.cargo/bin
make config   # Show current configuration
```

## Rust-Specific Tools

This is a Rust project. Use these tools:
- `cargo fmt` - Format code
- `cargo check` - Check compilation
- `cargo clippy` - Lint code
- `cargo test` - Run tests
- `cargo doc` - Generate documentation

Do NOT use:
- ❌ npm, yarn, bun (no JavaScript/TypeScript)
- ❌ hardcheck (doesn't exist for Rust)
- ❌ eslint, prettier (JavaScript tools)

---

# Agent Configuration Guide

## ConPort (Context Portal) MCP Server Configuration

ConPort is a database-backed Model Context Protocol (MCP) server for managing structured project context, designed to be used by AI assistants and developer tools within IDEs.

### What is ConPort?

Context Portal (ConPort) is your project's **memory bank**. It helps AI assistants understand your specific software project better by storing important information like decisions, tasks, and architectural patterns in a structured way.

**Key Features:**
- Keeps track of project decisions, progress, and system designs
- Stores custom project data (like glossaries or specs)
- Helps AI find relevant project information quickly (semantic search)
- Enables AI to use project context for better responses (RAG)
- More efficient than simple text file-based memory banks

### Installation Methods

#### Recommended: Using uvx (from PyPI)

```json
{
  "mcpServers": {
    "conport": {
      "command": "uvx",
      "args": [
        "--from",
        "context-portal-mcp",
        "conport-mcp",
        "--mode",
        "stdio",
        "--workspace_id",
        "${workspaceFolder}",
        "--log-file",
        "./logs/conport.log",
        "--log-level",
        "INFO"
      ]
    }
  }
}
```

#### Development Installation (from Git)

```bash
git clone https://github.com/GreatScottyMac/context-portal.git
cd context-portal
uv venv
source .venv/bin/activate  # Linux/macOS
# or .venv\Scripts\activate.bat  # Windows
uv pip install -r requirements.txt
```

### Configuration Parameters

- **`command`**: `uvx` (handles environment automatically)
- **`${workspaceFolder}`**: IDE variable for current project workspace path
- **`--log-file`**: Optional path for server logs (defaults to stderr)
- **`--log-level`**: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)

### Available ConPort Tools

ConPort exposes comprehensive tools via MCP for project knowledge management:

#### Product & Active Context
- `get_product_context`: Retrieve project goals, features, architecture
- `update_product_context`: Update product context (full or partial)
- `get_active_context`: Get current working focus and recent changes
- `update_active_context`: Update active context (full or partial)

#### Decision Management
- `log_decision`: Log architectural/implementation decisions
- `get_decisions`: Retrieve logged decisions with filtering
- `search_decisions_fts`: Full-text search across decisions
- `delete_decision_by_id`: Remove specific decisions

#### Progress Tracking
- `log_progress`: Log progress entries and task status
- `get_progress`: Retrieve progress entries with filtering
- `update_progress`: Update existing progress entries
- `delete_progress_by_id`: Remove progress entries

#### System Patterns
- `log_system_pattern`: Store coding/architectural patterns
- `get_system_patterns`: Retrieve system patterns
- `delete_system_pattern_by_id`: Remove patterns

#### Custom Data Management
- `log_custom_data`: Store custom key-value data by category
- `get_custom_data`: Retrieve custom data
- `delete_custom_data`: Remove custom data entries
- `search_project_glossary_fts`: Search project glossary
- `search_custom_data_value_fts`: Full-text search across all custom data

#### Knowledge Graph & Relationships
- `link_conport_items`: Create relationships between items
- `get_linked_items`: Retrieve linked items and relationships

#### Utilities
- `get_item_history`: Version history for contexts
- `get_recent_activity_summary`: Recent ConPort activity
- `get_conport_schema`: Tool schema information
- `export_conport_to_markdown`: Export data to markdown
- `import_markdown_to_conport`: Import from markdown
- `batch_log_items`: Bulk operations
- `semantic_search_conport`: AI-powered semantic search

### Initial Setup

1. **Create `projectBrief.md`** in your project root with:
   - Project goals and purpose
   - Key features and components
   - Target audience
   - Architecture overview
   - Core technologies

2. **Automatic Import**: ConPort-aware agents will detect and offer to import this file into the Product Context.

### Custom Instructions

ConPort includes tailored strategy files for different environments:
- `roo_code_conport_strategy`: For Roo Code VS Code extension
- `cline_conport_strategy`: For CLine VS Code extension  
- `cascade_conport_strategy`: For Windsurf Cascade
- `generic_conport_strategy`: Platform-agnostic instructions

### Database & Storage

- Uses SQLite (one DB per workspace: `context_portal/context.db`)
- Automatic database creation and schema migration
- Vector embeddings for semantic search
- Multi-workspace support via `workspace_id`

### Repository

- **GitHub**: https://github.com/GreatScottyMac/context-portal
- **License**: Apache-2.0
- **Language**: Python
- **Stars**: 554+ (actively maintained)

### Prerequisites

- Python 3.8+
- uv (recommended package manager)
- MCP-compatible IDE or client

ConPort transforms how AI assistants understand and work with your projects by providing structured, queryable project memory that enables more accurate and context-aware responses.

## ConPort Memory Strategy (Generic)

The following is the complete ConPort memory strategy for AI agents to effectively use ConPort tools:

```yaml
# --- ConPort Memory Strategy ---
conport_memory_strategy:
  # CRITICAL: At the beginning of every session, the agent MUST execute the 'initialization' sequence
  # to determine the ConPort status and load relevant context.
  workspace_id_source: "The agent must obtain the absolute path to the current workspace to use as `workspace_id` for all ConPort tool calls. This might be available as `${workspaceFolder}` or require asking the user."

  initialization:
    thinking_preamble: |

    agent_action_plan:
      - step: 1
        action: "Determine `ACTUAL_WORKSPACE_ID`."
      - step: 2
        action: "Invoke a \"list files\" tool for `ACTUAL_WORKSPACE_ID + \"/context_portal/\"`."
        parameters: 'path: ACTUAL_WORKSPACE_ID + "/context_portal/"'
      - step: 3
        action: "Analyze result and branch based on 'context.db' existence."
        conditions:
          - if: "'context.db' is found"
            then_sequence: "load_existing_conport_context"
          - else: "'context.db' NOT found"
            then_sequence: "handle_new_conport_setup"

  load_existing_conport_context:
    thinking_preamble: |

    agent_action_plan:
      - step: 1
        description: "Attempt to load initial contexts from ConPort."
        actions:
          - "Invoke `get_product_context`... Store result."
          - "Invoke `get_active_context`... Store result."
          - "Invoke `get_decisions` (limit 5 for a better overview)... Store result."
          - "Invoke `get_progress` (limit 5)... Store result."
          - "Invoke `get_system_patterns` (limit 5)... Store result."
          - "Invoke `get_custom_data` (category: \"critical_settings\")... Store result."
          - "Invoke `get_custom_data` (category: \"ProjectGlossary\")... Store result."
          - "Invoke `get_recent_activity_summary` (default params, e.g., last 24h, limit 3 per type) for a quick catch-up. Store result."
      - step: 2
        description: "Analyze loaded context."
        conditions:
          - if: "results from step 1 are NOT empty/minimal"
            actions:
              - "Set internal status to [CONPORT_ACTIVE]."
              - "Inform user: \"ConPort memory initialized. Existing contexts and recent activity loaded.\""
              - "Ask follow up questions with suggestions like \"Review recent activity?\", \"Continue previous task?\", \"What would you like to work on?\"."
          - else: "loaded context is empty/minimal despite DB file existing"
            actions:
              - "Set internal status to [CONPORT_ACTIVE]."
              - "Inform user: \"ConPort database file found, but it appears to be empty or minimally initialized. You can start by defining Product/Active Context or logging project information.\""
              - "Ask follow up questions with suggestions like \"Define Product Context?\", \"Log a new decision?\"."
      - step: 3
        description: "Handle Load Failure (if step 1's `get_*` calls failed)."
        condition: "If any `get_*` calls in step 1 failed unexpectedly"
        action: "Fall back to `if_conport_unavailable_or_init_failed`."

  handle_new_conport_setup:
    thinking_preamble: |

    agent_action_plan:
      - step: 1
        action: "Inform user: \"No existing ConPort database found at `ACTUAL_WORKSPACE_ID + \"/context_portal/context.db\"`.\""
      - step: 2
        action: "Ask follow up questions"
        parameters:
          question: "Would you like to initialize a new ConPort database for this workspace? The database will be created automatically when ConPort tools are first used."
          suggestions:
            - "Yes, initialize a new ConPort database."
            - "No, do not use ConPort for this session."
      - step: 3
        description: "Process user response."
        conditions:
          - if_user_response_is: "Yes, initialize a new ConPort database."
            actions:
              - "Inform user: \"Okay, a new ConPort database will be created.\""
              - description: "Attempt to bootstrap Product Context from projectBrief.md (this happens only on new setup)."
                thinking_preamble: |

                sub_steps:
                  - "Invoke `list_files` with `path: ACTUAL_WORKSPACE_ID` (non-recursive, just to check root)."
                  - description: "Analyze `list_files` result for 'projectBrief.md'."
                    conditions:
                      - if: "'projectBrief.md' is found in the listing"
                        actions:
                          - "Invoke `read_file` for `ACTUAL_WORKSPACE_ID + \"/projectBrief.md\"`."
                          - action: "Ask follow up questions"
                            parameters:
                              question: "Found projectBrief.md in your workspace. As we're setting up ConPort for the first time, would you like to import its content into the Product Context?"
                              suggestions:
                                - "Yes, import its content now."
                                - "No, skip importing it for now."
                          - description: "Process user response to import projectBrief.md."
                            conditions:
                              - if_user_response_is: "Yes, import its content now."
                                actions:
                                  - "(No need to `get_product_context` as DB is new and empty)"
                                  - "Prepare `content` for `update_product_context`. For example: `{\"initial_product_brief\": \"[content from projectBrief.md]\"}`."
                                  - "Invoke `update_product_context` with the prepared content."
                                  - "Inform user of the import result (success or failure)."
                      - else: "'projectBrief.md' NOT found"
                        actions:
                          - action: "Ask follow up questions."
                            parameters:
                              question: "`projectBrief.md` was not found in the workspace root. Would you like to define the initial Product Context manually now?"
                              suggestions:
                                - "Define Product Context manually."
                                - "Skip for now."
                          - "(If \"Define manually\", guide user through `update_product_context`)."
              - "Proceed to 'load_existing_conport_context' sequence (which will now load the potentially bootstrapped product context and other empty contexts)."
          - if_user_response_is: "No, do not use ConPort for this session."
            action: "Proceed to `if_conport_unavailable_or_init_failed` (with a message indicating user chose not to initialize)."

  if_conport_unavailable_or_init_failed:
    thinking_preamble: |

    agent_action: "Inform user: \"ConPort memory will not be used for this session. Status: [CONPORT_INACTIVE].\""

  general:
    status_prefix: "Begin EVERY response with either '[CONPORT_ACTIVE]' or '[CONPORT_INACTIVE]'."
    proactive_logging_cue: "Remember to proactively identify opportunities to log or update ConPort based on the conversation (e.g., if user outlines a new plan, consider logging decisions or progress). Confirm with the user before logging."
    proactive_error_handling: "When encountering errors (e.g., tool failures, unexpected output), proactively log the error details using `log_custom_data` (category: 'ErrorLogs', key: 'timestamp_error_summary') and consider updating active context with open issues if it's a persistent problem. Prioritize using ConPort's item history or recent activity summary to diagnose issues if they relate to past context changes."
    semantic_search_emphasis: "For complex or nuanced queries, especially when direct keyword search (e.g., `search_decisions_fts`, `search_custom_data_value_fts`) might be insufficient, prioritize using semantic search to leverage conceptual understanding and retrieve more relevant context. Explain to the user why semantic search is being used."

  conport_updates:
    frequency: "UPDATE CONPORT THROUGHOUT THE CHAT SESSION, WHEN SIGNIFICANT CHANGES OCCUR, OR WHEN EXPLICITLY REQUESTED."
    workspace_id_note: "All ConPort tool calls require the `workspace_id`."
    
  conport_sync_routine:
    trigger: "^(Sync ConPort|ConPort Sync)$"
    user_acknowledgement_text: "[CONPORT_SYNCING]"
    instructions:
      - "Halt Current Task: Stop current activity."
      - "Acknowledge Command: Send `[CONPORT_SYNCING]` to the user."
      - "Review Chat History: Analyze the complete current chat session for new information, decisions, progress, context changes, clarifications, and potential new relationships between items."
    core_update_process:
      thinking_preamble: |
        - Synchronize ConPort with information from the current chat session.
        - Use appropriate ConPort tools based on identified changes.
        - For `update_product_context` and `update_active_context`, first fetch current content, then merge/update (potentially using `patch_content`), then call the update tool with the *complete new content object* or the patch.
        - All tool calls require the `workspace_id`.
    post_sync_actions:
      - "Inform user: ConPort synchronized with session info."
      - "Resume previous task or await new instructions."

  dynamic_context_retrieval_for_rag:
    description: |
      Guidance for dynamically retrieving and assembling context from ConPort to answer user queries or perform tasks,
      enhancing Retrieval Augmented Generation (RAG) capabilities.
    trigger: "When the AI needs to answer a specific question, perform a task requiring detailed project knowledge, or generate content based on ConPort data."
    goal: "To construct a concise, highly relevant context set for the LLM, improving the accuracy and relevance of its responses."
    steps:
      - step: 1
        action: "Analyze User Query/Task"
        details: "Deconstruct the user's request to identify key entities, concepts, keywords, and the specific type of information needed from ConPort."
      - step: 2
        action: "Prioritized Retrieval Strategy"
        details: |
          Based on the analysis, select the most appropriate ConPort tools:
          - **Targeted FTS:** Use `search_decisions_fts`, `search_custom_data_value_fts`, `search_project_glossary_fts` for keyword-based searches if specific terms are evident.
          - **Specific Item Retrieval:** Use `get_custom_data` (if category/key known), `get_decisions` (by ID or for recent items), `get_system_patterns`, `get_progress` if the query points to specific item types or IDs.
          - **Semantic Search:** Prioritize semantic search tools for conceptual queries.
          - **Broad Context (Fallback):** Use `get_product_context` or `get_active_context` as a fallback if targeted retrieval yields little, but be mindful of their size.
      - step: 3
        action: "Retrieve Initial Set"
        details: "Execute the chosen tool(s) to retrieve an initial, small set (e.g., top 3-5) of the most relevant items or data snippets."
      - step: 4
        action: "Contextual Expansion (Optional)"
        details: "For the most promising items from Step 3, consider using `get_linked_items` to fetch directly related items (1-hop). This can provide crucial context or disambiguation. Use judiciously to avoid excessive data."
      - step: 5
        action: "Synthesize and Filter"
        details: |
          Review the retrieved information (initial set + expanded context).
          - **Filter:** Discard irrelevant items or parts of items.
          - **Synthesize/Summarize:** If multiple relevant pieces of information are found, synthesize them into a concise summary that directly addresses the query/task. Extract only the most pertinent sentences or facts.
      - step: 6
        action: "Assemble Prompt Context"
        details: |
          Construct the context portion of the LLM prompt using the filtered and synthesized information.
          - **Clarity:** Clearly delineate this retrieved context from the user's query or other parts of the prompt.
          - **Attribution (Optional but Recommended):** If possible, briefly note the source of the information (e.g., "From Decision D-42:", "According to System Pattern SP-5:").
          - **Brevity:** Strive for relevance and conciseness. Avoid including large, unprocessed chunks of data unless absolutely necessary and directly requested.
    general_principles:
      - "Prefer targeted retrieval over broad context dumps."
      - "Iterate if initial retrieval is insufficient: try different keywords or tools."
      - "Balance context richness with prompt token limits."

  proactive_knowledge_graph_linking:
    description: |
      Guidance for the AI to proactively identify and suggest the creation of links between ConPort items,
      enriching the project's knowledge graph based on conversational context.
    trigger: "During ongoing conversation, when the AI observes potential relationships (e.g., causal, implementational, clarifying) between two or more discussed ConPort items or concepts that are likely represented as ConPort items."
    goal: "To actively build and maintain a rich, interconnected knowledge graph within ConPort by capturing relationships that might otherwise be missed."
    steps:
      - step: 1
        action: "Monitor Conversational Context"
        details: "Continuously analyze the user's statements and the flow of discussion for mentions of ConPort items (explicitly by ID, or implicitly by well-known names/summaries) and the relationships being described or implied between them."
      - step: 2
        action: "Identify Potential Links"
        details: |
          Look for patterns such as:
          - User states "Decision X led to us doing Y (which is Progress item P-3)."
          - User discusses how System Pattern SP-2 helps address a concern noted in Decision D-5.
          - User outlines a task (Progress P-10) that implements a specific feature detailed in a `custom_data` spec (CD-Spec-FeatureX).
      - step: 3
        action: "Formulate and Propose Link Suggestion"
        details: |
          If a potential link is identified:
          - Clearly state the items involved (e.g., "Decision D-5", "System Pattern SP-2").
          - Describe the perceived relationship (e.g., "It seems SP-2 addresses a concern in D-5.").
          - Propose creating a link.
          - Example Question: "I noticed we're discussing Decision D-5 and System Pattern SP-2. It sounds like SP-2 might 'address_concern_in' D-5. Would you like me to create this link in ConPort? You can also suggest a different relationship type."
          - Suggested Answers:
            - "Yes, link them with 'addresses_concern_in'."
            - "Yes, but use relationship type: [user types here]."
            - "No, don't link them now."
          - Offer common relationship types as examples if needed: 'implements', 'clarifies', 'related_to', 'depends_on', 'blocks', 'resolves', 'derived_from'.
      - step: 4
        action: "Gather Details and Execute Linking"
        details: |
          If the user confirms:
          - Ensure you have the correct source item type, source item ID, target item type, target item ID, and the agreed-upon relationship type.
          - Ask for an optional brief description for the link if the relationship isn't obvious.
          - Invoke the `link_conport_items` tool.
      - step: 5
        action: "Confirm Outcome"
        details: "Inform the user of the success or failure of the `link_conport_items` tool call."
    general_principles:
      - "Be helpful, not intrusive. If the user declines a suggestion, accept and move on."
      - "Prioritize clear, strong relationships over tenuous ones."
      - "This strategy complements the general `proactive_logging_cue` by providing specific guidance for link creation."
```

### Key ConPort Strategy Points

1. **Initialization Required**: Every session must start with the initialization sequence to check for existing ConPort database and load context.

2. **Status Tracking**: All responses should begin with `[CONPORT_ACTIVE]` or `[CONPORT_INACTIVE]` status.

3. **Proactive Logging**: AI should identify opportunities to log decisions, progress, and context changes throughout conversations.

4. **Sync Command**: Users can trigger `Sync ConPort` to update all ConPort data with session information.

5. **RAG Enhancement**: Dynamic context retrieval strategy for better question answering using ConPort data.

6. **Knowledge Graph Building**: Proactive identification and creation of relationships between ConPort items.