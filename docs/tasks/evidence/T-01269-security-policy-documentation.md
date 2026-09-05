# T-01269: Package Management / Security Policy - Documentation

## 1. Overview & Architectural Goals
The AIOS Package Security Policy subsystem (`PackageSecurityPolicy`) enforces mandatory security baselines, organizational constraints, and integrity invariants across all package management operations in the Phase 1 Linux base environment.

The policy engine operates ahead of package registration, dependency planning, and transaction execution, ensuring that unsafe binaries, legacy/unencrypted services, tampered artifacts, or unapproved package architectures are rejected deterministically or flagged for compliance audit.

```
+-------------------------------------------------------------------------+
|                         AIOS Operational Layer                         |
|   CLI (`aiosh package policy`)   |   MCP Tool (`aios.package.policy`)   |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                  PackageSecurityPolicy Engine (`PP1..PP6`)              |
|  - Invariant PP1: Configuration Bounds & Structural Sanity             |
|  - Invariant PP2: Prohibited Package Enforcement (telnet, rsh, etc.)   |
|  - Invariant PP3: Mandatory SHA-256 Cryptographic Checksums            |
|  - Invariant PP4: Transport Protocol & Authorized Repository Security   |
|  - Invariant PP5: Architecture, Format, Size, & Dependency Hygiene     |
|  - Invariant PP6: Evaluation Modes (Enforcing, Audit, Permissive)       |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|             PackagePolicyVerdict (allowed, violations, audit)          |
|    Enforcing -> Block execution if fatal violations present             |
|    Audit     -> Permit execution; emit violation audit records          |
|    Permissive-> Block only critically dangerous packages (PP2)         |
+-------------------------------------------------------------------------+
```

---

## 2. Invariant Specifications (PP1..PP6)

| ID | Title | Specification & Enforcement Rules | Fatal Default |
|---|---|---|---|
| **PP1** | Configuration Bounds | Allowed architectures bounded to $[1 \dots 64]$; prohibited packages bounded to $[0 \dots 1024]$; package sizes constrained to $[10\text{ KiB} \dots 100\text{ GiB}]$; dependencies capped at $[1 \dots 1024]$. Policy files capped at 64 KiB. | `true` |
| **PP2** | Prohibited Packages | Case-insensitive blocking of legacy, unencrypted, or dangerous network utilities (`telnet`, `rsh-client`, `rsh-server`, `rlogin`, `rexec`, `nis`, `yp-tools`). Checked on package names and dependencies. | `true` |
| **PP3** | Checksum Enforcement | When `require_checksum = true`, package specification must contain a valid 64-character lowercase hex SHA-256 digest (`[0-9a-f]{64}`). Missing or corrupted hashes are rejected. | `true` |
| **PP4** | Transport Security | When `require_https_or_file_repo = true`, repository URLs must strictly begin with `https://` or `file://`. Plaintext `http://` is unconditionally blocked. Optional mirror whitelisting via `allowed_repositories`. | `true` |
| **PP5** | System Hygiene | Validates target architecture against approved architectures (`amd64`, `x86_64`, `aarch64`, etc.), format against approved list (`deb`, `tar_gz`, `raw_binary`), installed size within maximum ceiling, and dependency count within budget. | `true` (arch/format/size), `false` (dep count warning) |
| **PP6** | Operational Modes | Supports three distinct enforcement modes: `Enforcing` (denies any fatal violation), `Audit` (permits operations while generating forensic logs), and `Permissive` (blocks only PP2 prohibited packages). | Mode-dependent |

---

## 3. Data Structures & Rust API

Defined in `code/aiosh-rust/aiosh-core/src/package_policy.rs`:

```rust
pub enum PackagePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

pub struct PackageSecurityPolicy {
    pub mode: PackagePolicyMode,
    pub prohibited_packages: Vec<String>,
    pub allowed_architectures: Vec<String>,
    pub allowed_formats: Vec<PackageFormat>,
    pub require_checksum: bool,
    pub require_https_or_file_repo: bool,
    pub max_package_size_bytes: u64,
    pub max_dependencies_per_package: usize,
    pub allowed_repositories: Vec<String>,
}

pub struct PackagePolicyViolation {
    pub rule_id: String,
    pub package_name: String,
    pub description: String,
    pub fatal: bool,
}

pub struct PackagePolicyVerdict {
    pub package_name: String,
    pub allowed: bool,
    pub mode: PackagePolicyMode,
    pub violations: Vec<PackagePolicyViolation>,
    pub evaluated_at: String,
}
```

### Core Methods:
- `PackageSecurityPolicy::validate(&self) -> Result<(), String>`: Validates all structural bounds (PP1).
- `PackageSecurityPolicy::evaluate_spec(&self, spec: &PackageSpec) -> PackagePolicyVerdict`: Evaluates a single package specification against all active policy rules (PP2..PP6).
- `PackageSecurityPolicy::evaluate_transaction(&self, tx: &PackageTransaction, store: &PackageStore) -> PackagePolicyVerdict`: Evaluates all candidate package actions in a planned transaction against the store and security policy prior to mutation.
- `PackageSecurityPolicy::from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`: Loads JSON configuration file with path sanitization and 64 KiB size limiting.
- `PackageSecurityPolicy::resolve(custom_path: Option<&str>) -> Result<Self, String>`: Resolves policy following precedence: explicit path -> `/etc/aios/package_policy.json` -> environment variables -> built-in defaults.

---

## 4. Command-Line Interface (CLI)

Subcommand: `aiosh package policy`

```bash
# View active security policy configuration
aiosh package policy

# View active security policy in JSON format
aiosh package policy --json

# Evaluate an installed/registered package against policy
aiosh package policy --package curl

# Evaluate a package with a custom policy configuration file
aiosh package policy --config /etc/aios/package_policy.json --package telnet
```

### Return Codes:
- `0`: Operation succeeded; package or transaction conforms to security policy.
- `1`: I/O or system error reading policy/package store.
- `2`: Policy violation detected under `Enforcing` mode, or invalid CLI arguments/syntax.

---

## 5. Model Context Protocol (MCP) API

Tool: `aios.package.policy`

### Input Schema:
```json
{
  "type": "object",
  "properties": {
    "config_path": {
      "type": "string",
      "description": "Optional path to custom policy JSON configuration file"
    },
    "package_name": {
      "type": "string",
      "description": "Optional name of package to evaluate against policy"
    },
    "package": {
      "type": "string",
      "description": "Alias for package_name"
    },
    "store_path": {
      "type": "string",
      "description": "Optional path to package store state file"
    }
  }
}
```

### Output Format (Package Evaluation):
```json
{
  "ok": true,
  "tool": "aios.package.policy",
  "verdict": {
    "package_name": "curl",
    "allowed": true,
    "mode": "enforcing",
    "violations": [],
    "evaluated_at": "2026-09-04T00:00:00Z"
  }
}
```

---

## 6. Environment Variable Configuration Hierarchy

| Variable | Type / Valid Values | Description |
|---|---|---|
| `AIOS_PACKAGE_POLICY_MODE` | `enforcing` \| `audit` \| `permissive` | Sets system-wide package policy enforcement mode |
| `AIOS_PACKAGE_REQUIRE_CHECKSUM` | `1` \| `0` \| `true` \| `false` | Enforces mandatory SHA-256 checksum presence |
| `AIOS_PACKAGE_REQUIRE_HTTPS` | `1` \| `0` \| `true` \| `false` | Mandates HTTPS or file:// repository endpoints |
| `AIOS_PACKAGE_MAX_SIZE_BYTES` | Integer (bytes) | Overrides maximum permitted package size ceiling |

---

## 7. Audit & Forensic PEP Integration
Every evaluation executed via CLI or MCP records an event to the AIOS policy enforcement point (PEP) ring buffer, logging:
- Timestamp (UTC ISO-8601)
- Actor identification (`operator` or `agent:mcp`)
- Action (`aios.package.policy`)
- Decision verdict (`allowed` or `denied`)
- Associated rule IDs and violation descriptions
