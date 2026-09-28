# Specification: Sandbox Enforcement Observability Subsystem

## 1. Overview
The Sandbox Enforcement Observability Subsystem (`SandboxObservabilityReport`) provides runtime telemetry, health assessment, execution metric tallies, outcome distributions, profile utilization, and containment primitive detection across the AIOS environment.

---

## 2. Invariants & Rulesets

| Invariant | Name | Description |
|---|---|---|
| `SANDBOXOBS1` | **Point-in-Time Metrics** | Emits aggregated totals for executions, profiles, outcomes, and policy status. |
| `SANDBOXOBS2` | **Bounded Collection** | Caps frequency distribution maps to at most `MAX_OUTCOME_DISTRIBUTION_ENTRIES` (128) to prevent memory exhaustion. |
| `SANDBOXOBS3` | **Telemetry Sanitization** | Scrubs control characters and trims strings to `MAX_TELEMETRY_TEXT_LEN` (256 bytes) to eliminate ANSI terminal escape injection. |
| `SANDBOXOBS4` | **Host Capability Detection** | Reports probe results for platform-specific isolation primitives (e.g. Landlock LSM, Seccomp-BPF). |
| `SANDBOXOBS5` | **Health Invariant Verification** | Checks presence of factory baseline profiles (`standard`, `strict`, `permissive`) to determine system health. |
| `SANDBOXOBS6` | **Fail-Safe Envelope** | Errors are reported via standard result envelopes with `SANDBOXOBS_ERR_*` error codes without panics. |

---

## 3. Interfaces & Usage

### CLI Commands
```bash
# View human-readable sandbox observability report
aiosh sandbox stats

# View structured telemetry JSON payload
aiosh sandbox stats --json
```

### MCP Tool Surface
- Tool: `aios.sandbox.stats`
- Arguments: `{}` (empty object)
- Response Example:
```json
{
  "code": 0,
  "data": {
    "is_healthy": true,
    "policy_mode": "enforcing",
    "total_profiles_registered": 3,
    "total_executions_recorded": 12,
    "executions_by_outcome": {
      "ok": 8,
      "error": 4
    },
    "executions_by_profile": {
      "permissive": 12
    },
    "host_capabilities": {
      "platform": "windows",
      "landlock_supported": false,
      "landlock_abi_version": null,
      "seccomp_bpf_supported": false,
      "no_new_privs_supported": false
    },
    "generated_at_utc": "2026-09-28T21:22:05.083833700+00:00"
  },
  "error": null
}
```

---

## 4. Constraints & Known Limitations
1. Telemetry querying is bounded to the most recent 1,000 audit log rows (`tail(1000)`) to maintain strict sub-millisecond query execution bounds.
2. In-memory ephemeral executions without an attached `AuditRing` will reflect 0 total executions unless audit storage is configured.
3. Linux kernel capabilities (`Landlock`, `Seccomp`) evaluate to `false` when running on Windows hosts.

---

## 5. Linked Task Evidence
- Scaffold & Implementation: `docs/tasks/evidence/T-02474-observability-scaffold.md`
- Unit Testing: `docs/tasks/evidence/T-02475-observability-unit-test.md`
- Integration: `docs/tasks/evidence/T-02476-observability-integration.md`
- Security Review: `docs/tasks/evidence/T-02477-observability-security-review.md`
- Hardening: `docs/tasks/evidence/T-02478-observability-hardening.md`
