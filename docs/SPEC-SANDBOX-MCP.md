# Sandbox Enforcement MCP / API Specification

## 1. Overview
The Sandbox Enforcement MCP surface provides autonomous agents with JSON-RPC 2.0 tools to discover containment profiles, probe host platform sandboxing capabilities, and execute supervised tasks in isolated containment.

---

## 2. Tools Reference

### `aios.sandbox.profiles`
- **Description**: List registered sandbox containment profiles.
- **Example Request**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "aios.sandbox.profiles",
      "arguments": {}
    }
  }
  ```

### `aios.sandbox.probe`
- **Description**: Probe host kernel sandbox capabilities (Landlock, seccomp-bpf, no_new_privs).
- **Example Request**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "aios.sandbox.probe",
      "arguments": {}
    }
  }
  ```

### `aios.sandbox.exec`
- **Description**: Execute a command under sandbox containment with watchdog supervision and output capture.
- **Example Request**:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "aios.sandbox.exec",
      "arguments": {
        "command": "/usr/bin/python3",
        "args": ["-c", "print('hello from sandbox')"],
        "profile": "permissive",
        "cwd": "/tmp"
      }
    }
  }
  ```

---

## 3. Constraints & Limitations
- Directory traversal (`..`) in `cwd` is strictly prohibited.
- Output buffers are clamped at 10 MiB to prevent memory exhaustion.
- High-privilege or policy-restricted execution requires a valid PEP grant token.
