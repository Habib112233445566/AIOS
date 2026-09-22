# Task Evidence: T-02221 (Grant Lifecycle / CLI surface: Research)

## 1. Scope & Objective
Research and define the architecture, UX design, flag taxonomy, exit code contracts, and security boundaries for the dedicated Grant Lifecycle CLI surface (`aiosh pep grant <subcommand>`) in `code/aiosh-rust/aiosh-cli/src/main.rs`.

---

## 2. Command Grammar & Subcommand Taxonomy

The operator CLI surface provides complete administrative oversight over PEP authorization grants:

| Subcommand | Purpose | Required Flags / Positional | Optional Flags | Exit Codes |
|---|---|---|---|---|
| `issue` | Issue a new authorization grant | `--id`, `--subject`, `--scope-type`, `--rights` | `--issuer`, `--scope-path`, `--expires-at`, `--not-before`, `--max-invocations`, `--max-bytes`, `--delegation-depth`, `--store`, `--json` | `0`: Created<br>`2`: Validation error |
| `list` | List registered authorization grants | None | `--subject`, `--state`, `--store`, `--json` | `0`: Listed<br>`2`: Bad filter |
| `inspect` | Detailed grant inspection | `<grant_id>` | `--store`, `--json` | `0`: Found<br>`1`: Not found<br>`2`: Bad args |
| `validate` | Verify validity for subject & right | `<grant_id>` | `--subject`, `--right`, `--store`, `--json` | `0`: Valid<br>`1`: Invalid/Denied<br>`2`: Bad args |
| `attenuate` | Derive child grant with reduced rights | `<parent_id>` or `--parent`, `--child`, `--subject`, `--rights` | `--store`, `--json` | `0`: Derived<br>`1`: Attenuation failed<br>`2`: Bad args |
| `revoke` | Revoke grant and optional descendants | `<grant_id>` | `--reason`, `--cascade`, `--store`, `--json` | `0`: Revoked<br>`1`: Not found<br>`2`: Bad args |
| `sweep` | Sweep expired/exhausted grants | None | `--store`, `--json` | `0`: Swept<br>`1`: Sweep failed<br>`2`: Bad store |

---

## 3. Flag Taxonomy & Type Mapping

1. `--rights`: Comma-delimited list of capability rights (`read,write,execute,admin,delegate`).
2. `--scope-type`: String discriminator (`filesystem`, `network`, `ipc`, `system`).
3. `--scope-path`: String path, URI, or target resource descriptor.
4. `--delegation-depth`: Integer (0..8).
5. `--max-invocations`: Integer $\ge 0$.
6. `--max-bytes`: Integer $\ge 0$.
7. `--cascade`: Boolean flag enabling recursive transitive revocation.
8. `--json`: Standard structured envelope `{ "code": i32, "data": Value, "error": Value }`.
9. `--store`: Custom file path to grant JSON store (enforcing `.json` and path hygiene).

---

## 4. Security & Audit Considerations
1. **Sanitization**: All error messages and string outputs passed through `sanitize_terminal` to prevent terminal injection (ANSI control sequences).
2. **Audit Telemetry**: Every invocation calls `classify_and_emit` to record the operator action, grant ID, outcome (`success`/`failure`), and metadata into the local audit trail.
3. **Path Hygiene**: Target store path checked for `..` directory traversal and `.json` extension.
4. **Principle of Least Privilege**: Attenuation enforces strict subset rights check; parent must have `delegate` permission.

---

## 5. Acceptance Confirmation
- [x] Full command taxonomy and flag grammar researched and defined.
- [x] Exit code contracts formalized: 0 (Success/Permit), 1 (Denied/NotFound), 2 (Validation/Usage Error).
- [x] Input sanitization, path hygiene, and audit emission requirements aligned with AIOS standards.
