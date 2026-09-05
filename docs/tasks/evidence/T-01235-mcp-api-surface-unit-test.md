# T-01235: Package Management - MCP/API Surface: Unit Tests

## Metadata
- **Task ID:** `T-01235`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Unit Tests
- **Status:** Complete

## 1. Unit Test Coverage & Scope
Expanded automated tests in `code/aiosh-rust/aiosh-mcp/src/main.rs` (`tests::test_mcp_package_tools`) to comprehensively cover valid, invalid, boundary, and negative failure modes:

1. **`aios.package.validate`**:
   - Valid name (`curl`) -> `ok: true`, `valid: true`.
   - Invalid name syntax (`Curl`) -> `ok: false`.
   - Control-character name (`bad\x07name`) -> `ok: false`.
   - Valid specification -> `ok: true`, `valid: true`.
   - Self-dependency violation -> `ok: false`.
   - Missing arguments -> `ok: false`.
2. **`aios.package.list`**:
   - Complete package catalogue enumeration (8 canonical packages).
   - Format filtering (`deb` -> 5 packages).
3. **`aios.package.get`**:
   - Exact lookup (`curl`) -> `ok: true`, full `PackageSpec`.
   - Non-existent package lookup (`non-existent`) -> `ok: false`.
4. **`aios.package.plan`**:
   - Valid multi-action plan -> `ok: true`, verifies size delta calculation (9,437,184 bytes).
   - Dependency closure failure -> `ok: false`.
5. **`aios.package.search`**:
   - Substring pattern lookup (`curl`) -> `ok: true`, 1 match.
   - Missing pattern -> `ok: false`.
   - Custom limit bound (`limit: 2`) -> `ok: true`, <= 2 matches returned.
   - Control characters in pattern (`bad\0pattern`) -> `ok: false`.
   - Oversized pattern (>256 characters) -> `ok: false`.
   - Invalid limit argument (`limit: 0`) -> `ok: false`.
6. **`aios.package.apply`**:
   - Missing arguments -> `ok: false`.
   - Dry-run execution via `actions` -> `ok: true`, `dry_run: true`.
   - Execution via pre-computed `plan` -> `ok: true`.
   - Dependency closure failure -> `ok: false`.
   - Control-character store path (`bad\0store.json`) -> `ok: false`.
   - Malformed plan JSON (`not_an_object`) -> `ok: false`.
   - Real atomic persistence with disk roundtrip and state assertion (`state == Installed`).
7. **Tool Discovery (`Server::tool_manifest`)**:
   - Verified that `aios.package.search` and `aios.package.apply` are discoverable via MCP `tools/list`.

## 2. Test Execution Output
```text
running 1 test
test tests::test_mcp_package_tools ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.13s
```
