# Evidence: T-02066 - security policy: Integration

## Task Overview
- **Task ID**: `T-02066`
- **Sub-Epic**: Sub-Epic 7: Security Policy (`T-02061`..`T-02070`)
- **Phase**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Goal**: Integrate Capability Security Policy with the surrounding system (MCP JSON-RPC interface and registry persistence).

## Integration Architecture & Verification
1. **MCP Surface Integration**:
   - `aiosh-mcp` tools `aios.capability.issue` and `aios.capability.attenuate` delegate to `CapabilityService`.
   - `CapabilityService` automatically invokes `CapabilitySecurityPolicy::evaluate_issuance` and `CapabilitySecurityPolicy::evaluate_attenuation`.
   - Any policy violations are rejected with descriptive error payloads (`CAPSEC_PROHIBITED_PATH`, `CAPSEC_PROHIBITED_HOST`, `CAPSEC_DISALLOWED_RIGHT`, etc.).
2. **Integration Smoke Suite (`code/aiosh-mcp/tests/test_capability_policy_smoke.py`)**:
   - `test_prohibited_paths_over_mcp`: Blocks root issuance with `/etc/shadow` and `/proc/cpuinfo`; permits `/workspace/safe`.
   - `test_prohibited_network_hosts_over_mcp`: Blocks root issuance for `169.254.169.254` (cloud metadata service); permits `api.internal`.
   - `test_disallowed_rights_and_attenuation_over_mcp`: Blocks `Admin` rights for `untrusted:*` subjects during root issuance and attenuation; permits `Read` attenuation.
3. **Execution Output**:
   ```text
   Running Capability Policy Integration Smoke against binary: C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\aiosh-mcp.exe
   TEST: prohibited filesystem path blocking over MCP ... OK
   TEST: prohibited network host blocking over MCP ... OK
   TEST: disallowed rights & attenuation policy over MCP ... OK
   ALL POLICY INTEGRATION TESTS PASSED
   ```
4. **Regression Verification**:
   - `cargo test --test test_capability_automated`: 8/8 tests passed in 0.04s.
   - `cargo test --test test_capability_policy`: 8/8 tests passed in 0.00s.
