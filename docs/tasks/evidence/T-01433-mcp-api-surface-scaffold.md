# T-01433: User Session Bootstrap - MCP/API Surface: Scaffold

## Metadata
- **Task ID:** `T-01433`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** User Session Bootstrap MCP/API Surface Scaffold (`code/aiosh-rust/aiosh-mcp`)
- **Status:** Complete
- **Date:** 2026-09-09
- **Milestone:** Sub-Epic: User Session Bootstrap (3/10) — MCP/API Surface Scaffold

---

## 1. Scaffold Implementation

### 1.1 Tool Manifest Registration (`tool_manifest`)
Extended `code/aiosh-rust/aiosh-mcp/src/main.rs::tool_manifest` with the specification schema for `aios.session.create`:
```rust
tools.push(json!({
    "name": "aios.session.create",
    "description": "Bootstrap and register a new user or autonomous AI agent session into the store",
    "inputSchema": {
        "type": "object",
        "properties": {
            "spec": { "type": "object", "description": "Complete UserSessionSpec payload defining session configuration" },
            "store_path": { "type": "string", "description": "Optional path to custom session store JSON file" },
            "grant_id": { "type": "string", "description": "Optional PEP authorization grant ID" }
        },
        "required": ["spec"],
        "additionalProperties": false
    }
}));
```

### 1.2 Fail-Loud Dispatch Stub (`call_tool`)
Added dispatch branch in `call_tool` routing through `dispatch::recorded_call`:
```rust
"aios.session.create" => {
    let spec_opt = arguments.get("spec");
    let store_path_opt = arguments.get("store_path").and_then(|v| v.as_str()).map(|s| s.to_string());

    let f = move || -> Result<Value, String> {
        let _ = spec_opt.ok_or_else(|| "Missing required 'spec' parameter".to_string())?;
        let _ = store_path_opt;
        Err("aios.session.create scaffold stub: implementation pending T-01434".into())
    };
    dispatch::recorded_call(
        &mut self.ring, &self.pep,
        "aios.session.create", "Bootstrap and register a new user or agent session", arguments,
        None, grant_id, false, dispatch::DEFAULT_ACTOR_ID, dispatch::DEFAULT_ACTOR, f,
    )
}
```

### 1.3 In-Tree Unit Test Stub Verification
Added test step 14 in `tests::test_mcp_session_validate_tools`:
```rust
// 14. Discovery of aios.session.create and fail-loud scaffold stub
assert!(tools.iter().any(|t| t.get("name").and_then(|v| v.as_str()) == Some("aios.session.create")));
let res_create_scaffold = server.call_tool("aios.session.create", &json!({
    "spec": {
        "session_id": "test-scaffold-sess"
    }
}));
assert_eq!(res_create_scaffold.get("ok").and_then(|v| v.as_bool()), Some(false));
assert!(res_create_scaffold.get("error").and_then(|v| v.as_str()).unwrap().contains("scaffold stub"));
```

---

## 2. Test Verification Output

### 2.1 MCP Unit Test Execution
```text
running 1 test
test tests::test_mcp_session_validate_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.08s
```

### 2.2 Master Session Runner (`python tools/test_session_suites.py`)
```text
[+] SB1 session data model integrity & invariants (SB1..SB5)
[+] SB2 session CLI surface commands & options (validate, help, errors)
[+] SB3 session MCP tool surface (aios.session.validate)
[+] SB4 session core service lifecycle, seat arbitration & invariants (CS1..CS5)

PASS: session_suites criteria (SB1..SB4)
```

---

## 3. Acceptance Verification
- [x] Module skeleton and interface defined in `code/aiosh-rust/aiosh-mcp/src/main.rs`.
- [x] Fail-loud stub implemented with typed parameters.
- [x] Project compiles and builds with zero errors.
- [x] Interface verified by test stub assertion.
