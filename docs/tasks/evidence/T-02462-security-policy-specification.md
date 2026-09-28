# T-02462: Sandbox Enforcement Security Policy Specification

## 1. Specification Overview
This specification defines the declarative security policy system for Sandbox Enforcement (`SandboxSecurityPolicy`), providing an overarching security gate that evaluates every `SandboxExecutionRequest` before execution.

---

## 2. Invariants & Rulesets

| Invariant | Description |
|---|---|
| `SANDBOXPOL1` | **Tri-Mode Evaluation**: Policy operates in `Enforcing`, `Permissive`, or `Disabled` modes. In `Enforcing` mode, any policy violation rejects execution immediately. |
| `SANDBOXPOL2` | **Prohibited Commands & Binaries**: Rejects execution requests whose primary command matches prohibited patterns (e.g., `rm`, `dd`, `mkfs`, `format`, `shutdown`). |
| `SANDBOXPOL3` | **Environment Injection Confinement**: Prohibits dangerous library injection or loader hijacking environment variables (`LD_PRELOAD`, `DYLD_INSERT_LIBRARIES`, `PYTHONPATH`, `NODE_OPTIONS`). |
| `SANDBOXPOL4` | **PEP Grant Mandate**: Execution under high-privilege profiles (e.g. `permissive`) strictly requires a valid `pep_grant_id`. |
| `SANDBOXPOL5` | **Global Resource Ceiling**: Imposes hard upper bounds on execution runtime (`max_permissible_wall_time_ms`) and memory usage (`max_permissible_memory_bytes`). |
| `SANDBOXPOL6` | **Bounded Storage & Hermetic I/O**: Policy JSON configuration files are strictly bounded to `MAX_SANDBOX_POLICY_BYTES` (64 KiB) and reject directory traversal. |

---

## 3. Data Types & API Signatures

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxPolicyMode {
    Enforcing,
    Permissive,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum SandboxPolicyVerdict {
    Permit,
    PermitWithWarning { warning: String },
    Deny { reason: String, code: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxSecurityPolicy {
    pub version: String,
    pub mode: SandboxPolicyMode,
    pub prohibited_commands: Vec<String>,
    pub prohibited_env_vars: Vec<String>,
    pub require_pep_grant_for_profiles: Vec<String>,
    pub max_permissible_wall_time_ms: u64,
    pub max_permissible_memory_bytes: u64,
}

impl SandboxSecurityPolicy {
    pub fn default() -> Self;
    pub fn validate(&self) -> Result<(), String>;
    pub fn evaluate(&self, req: &SandboxExecutionRequest) -> SandboxPolicyVerdict;
    pub fn load_from_path(p: &Path) -> Result<Self, String>;
    pub fn save_to_path(&self, p: &Path) -> Result<(), String>;
}
```
