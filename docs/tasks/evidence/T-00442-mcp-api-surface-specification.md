# T-00442 — Documentation Index Control / MCP/API surface: Specification

## 1. Specification Overview
This document specifies the Model Context Protocol (MCP) tool registrations, input schemas, response structures, and audit invariants for Documentation Index Control in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

## 2. MCP Tools Specification

### A. `aios.doc.index.get`
- **Description**: Returns the active repository documentation index catalog.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "repo_path": {
        "type": "string",
        "description": "Optional absolute or relative path to the repository root"
      }
    }
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "manifest": {
      "version": "1.0.0",
      "entries": [
        {
          "path": "docs/README.md",
          "title": "Main Documentation",
          "section": "Documentation",
          "task_range": null,
          "links": ["docs/SPEC-TASK-LEDGER.md"]
        }
      ]
    }
  }
  ```

### B. `aios.doc.check`
- **Description**: Runs link verification against indexed documentation files and returns a structured validation report.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "repo_path": {
        "type": "string",
        "description": "Optional repository root path"
      }
    }
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "report": {
      "total_links_checked": 14,
      "broken_links": [],
      "is_valid": true
    }
  }
  ```

### C. `aios.doc.search`
- **Description**: Performs case-insensitive search across indexed document titles, paths, and sections.
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Search keyword query"
      },
      "repo_path": {
        "type": "string",
        "description": "Optional repository root path"
      }
    },
    "required": ["query"]
  }
  ```
- **Response Format**:
  ```json
  {
    "ok": true,
    "matches": [
      {
        "path": "docs/tasks/GOALS.md",
        "title": "Goals & Sequential Laws",
        "section": "Task Ledger",
        "task_range": null,
        "links": []
      }
    ]
  }
  ```

## 3. PEP & Audit Policy
- All three tools are read-only diagnostics and emit structured audit rows with `outcome: "ok"` or `outcome: "error"`.
