# T-00942 — Agent Handoff Protocol / MCP/API Surface: Specification

## 1. MCP Tool Endpoints & JSON Schema

### 1. `aios.handoff.list`
```json
{
  "name": "aios.handoff.list",
  "description": "List tracked agent handoffs with optional status or active filtering.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "active": { "type": "boolean", "description": "Filter to only active (Pending/Accepted) handoffs" },
      "status": { "type": "string", "description": "Filter by specific status (pending, accepted, rejected, completed, cancelled, expired)" },
      "store_path": { "type": "string", "description": "Path to handoff_store.json" }
    }
  }
}
```

### 2. `aios.handoff.show`
```json
{
  "name": "aios.handoff.show",
  "description": "Retrieve full metadata, status, context summary, and payload of a specific handoff.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "id": { "type": "string", "description": "Handoff identifier (e.g. HND-a1b2c3d4)" },
      "store_path": { "type": "string", "description": "Path to handoff_store.json" }
    },
    "required": ["id"]
  }
}
```

### 3. `aios.handoff.initiate`
```json
{
  "name": "aios.handoff.initiate",
  "description": "Initiate and enqueue a new handoff from a sender agent to a receiver agent.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "sender_agent_id": { "type": "string", "description": "Identifier of the sender agent" },
      "receiver_agent_id": { "type": "string", "description": "Identifier of the receiver agent" },
      "context_summary": { "type": "string", "description": "Summary of agent context and handoff intent" },
      "task_id": { "type": "integer", "description": "Optional associated task ID" },
      "payload_json": { "type": "string", "description": "Optional detailed structured JSON payload" },
      "priority": { "type": "string", "description": "Priority level (low, normal, high, urgent)" },
      "store_path": { "type": "string", "description": "Path to handoff_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["sender_agent_id", "receiver_agent_id", "context_summary"]
  }
}
```

### 4. `aios.handoff.accept`
```json
{
  "name": "aios.handoff.accept",
  "description": "Accept a pending handoff request.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "id": { "type": "string", "description": "Handoff identifier" },
      "notes": { "type": "string", "description": "Optional acceptance notes" },
      "store_path": { "type": "string", "description": "Path to handoff_store.json" },
      "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
    },
    "required": ["id"]
  }
}
```

### 5. `aios.handoff.reject`, `aios.handoff.complete`, `aios.handoff.cancel`
Same schema as `aios.handoff.accept` with appropriate transition semantics.
