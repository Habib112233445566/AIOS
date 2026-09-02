# T-00342 — Dependency & Toolchain Pinning / MCP/API surface: Specification

## 1. MCP Surface Contract

### 1.1 Tool: `aios.toolchain.check`

**Synopsis:**
Exposes the core environment validation logic to the agent via MCP.

**JSON-RPC Input Schema:**
```json
{
    "type": "object",
    "properties": {},
    "additionalProperties": false
}
```
*Note: No `config_path` override is provided. The MCP server must enforce the host's actual environment configuration.*

**Behavior:**
1. Resolve the `ToolchainManifest` via `ToolchainManifest::from_env()`.
2. Call `enforce_toolchain(&manifest)`.
3. If successful, return:
   ```json
   {
       "ok": true,
       "tool": "aios.toolchain.check",
       "config": <manifest_with_sources>
   }
   ```
4. If failed, return:
   ```json
   {
       "ok": false,
       "tool": "aios.toolchain.check",
       "error": "<message>"
   }
   ```

### 1.2 Tool: `aios.toolchain.config.get`
*Already exists.*
Returns the manifest without running `enforce_toolchain()`.

## 2. Reused vs New

- **Reused**: `dispatch::recorded_call()` is used to execute the tool safely.
- **Reused**: `aiosh_core::toolchain_service::enforce_toolchain()`.
- **Reused**: `ToolchainManifest::from_env()`.
- **New**: Registration of `aios.toolchain.check` in `tool_manifest()`.
- **New**: Match arm for `aios.toolchain.check` in `call_tool()`.

## 3. Security and Audit Effects
- **PEP Grant**: Not required. The check is read-only validation of host state. `requires_grant = false`.
- **Audit**: `dispatch::recorded_call()` guarantees that exactly one audit row is emitted for this tool invocation, containing the tool name `"aios.toolchain.check"` and outcome detail.
