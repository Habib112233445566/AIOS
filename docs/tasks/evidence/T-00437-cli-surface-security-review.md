# T-00437 — Documentation Index Control / CLI surface: Security Review

## 1. Review Scope
This security review assesses the CLI surface implementation (`aiosh doc`) in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. Threat Scenarios & Mitigations

### 1. Argument Injection & Flag Misuse
- **Threat**: Malicious CLI arguments containing embedded nulls, control characters, or non-UTF-8 byte sequences causing panic crashes.
- **Mitigation**: `main.rs` converts OS arguments lossily (`to_string_lossy`) and performs explicit flag stripping (`strip_flags`) and positional argument parsing before evaluation.

### 2. Unauthorized File System Traversal via `--repo`
- **Threat**: An attacker passes `--repo ../../sensitive_dir` to trigger disclosure of unauthorized file trees.
- **Mitigation**: `aiosh doc` operations only read predefined or validated documentation files bounded by `MAX_DOC_BYTES` (16 MiB), and `validate_doc_links` verifies root containment.

### 3. Audit Traceability
- **Policy**: All diagnostic executions (`doc.show`, `doc.check`, `doc.search`) emit structured audit records with execution outcomes and target metadata into the local audit ring.

## 3. Verdict
No known policy bypasses or security vulnerabilities remain open.
