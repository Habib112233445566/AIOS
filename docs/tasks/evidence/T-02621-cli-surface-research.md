# T-02621: Secrets Handling CLI Surface Research

- **Task**: `T-02621`
- **Sub-Epic**: Secrets Handling / CLI surface (Sub-Epic 3 of 10)
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Context & Objectives
The Secrets Handling CLI surface provides administrative, agentic, and developer command-line interactions for the runtime secret vault (`SecretService`). It maps terminal commands (`aiosh secret [store|get|list|rotate|revoke]`) to the core service while strictly enforcing scoped security, redaction, and audit logging.

## 2. Research Findings & Command Structure
1. **Subcommand Namespace**:
   - `aiosh secret` (alias: `aiosh sec`) avoiding collision with earlier scanner command `aiosh secrets <scan|check>`.
2. **Operations**:
   - `aiosh secret store`: Registers a new secret entry with required `--id`, `--name`, `--kind`, and optional `--scope`, `--target`, `--value`, `--store`.
   - `aiosh secret get`: Fetches secret by `--id`. By default, prints masked string; requires `--expose` to output unmasked plaintext.
   - `aiosh secret list`: Lists all vaulted secret metadata with optional filtering by `--kind` and `--scope`. Never outputs plaintext values.
   - `aiosh secret rotate`: Rotates target secret value with `--id` and `--value`.
   - `aiosh secret revoke`: Revokes target secret with `--id`, zeroing memory payload and locking future retrievals.
3. **Safety & Audit Controls**:
   - Plaintext values are never written to audit trails or error logs.
   - Support `--json` output across all subcommands.
   - Default path fallback with traversal checks.
