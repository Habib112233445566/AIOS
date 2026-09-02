# T-00272 — Release Packaging & Backup: Security Policy Specification

## Inputs, Outputs & Error Cases

### 1. PEP Gating (Policy Enforcement Point)
- **Input**: The agent's granted scopes (`Vec<String>`) and the requested action (e.g., `aios.release.generate`).
- **Logic**: The dispatcher intercepts the tool call. It calls `pep::check(grants, action)`.
- **Output (Happy Path)**: `Ok(())` if the grant matches. Execution continues to the handler.
- **Output (Failure Path)**: `Err(PolicyRefusal)` if the grant is missing. Execution aborts immediately. The agent receives a 403-equivalent refusal in its context window.

### 2. Audit Ring Emission
- **Input**: The successful result of a release generation or backup creation.
- **Logic**: Before returning the final result to the agent, the handler creates an Audit Event and pushes it to the ledger.
- **Output (Happy Path)**: The `MASTER_TASK_LEDGER.jsonl` contains a new row indicating what was exported and where.
  - *Event Type*: `ReleaseGenerated` or `BackupCreated`
  - *Payload*: `{"output_path": "/var/aios/release.iso", "size_bytes": 1048576, "components": ["aiosh-core"]}`
- **Output (Failure Path)**: If the export fails, an audit row for `ReleaseFailed` is written to ensure the attempt is logged, per ADR-0035 \S F-2.

## Interfaces
- **Reused**: `aiosh-core/src/pep.rs` (Policy Enforcement) and `aiosh-core/src/audit.rs` (Ledger writing).
- **New**: The specific event schema types `ReleaseGenerated`, `BackupCreated`, and `ReleaseFailed` will be added to the canonical JSON schema definitions.

## Consistency & Isolation
These changes strictly enforce the boundary that autonomous agents cannot exfiltrate or package the system image without possessing an explicit cryptographic grant token for that operation.
