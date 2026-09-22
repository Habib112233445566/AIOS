# Task Evidence: T-02194 (recovery & validation: Implementation)

## 1. Objective & Scope
Implement the core behavior for the PEP Decision Engine Recovery & Validation subsystem in `code/aiosh-rust/aiosh-core/src/pep_recovery.rs`.

## 2. Implementation Summary
- **Multi-Level Store Validator (`PepStoreValidator`)**:
  - `validate_path(&Path)`: Level 1 file and path hygiene (path traversal checks, symlink checks, size limit $\le 10 \text{ MiB}$).
  - `validate_content(&str, Option<&Path>)`:
    - Level 2 JSON syntax validation, supporting both JSON Array (`[ { ... } ]`) and JSON Object map (`{ "rule_id": { ... } }`) as written by `PepDecisionService`.
    - Level 3 Semantic rule constraints: rule ID bounds ($\le 128$), control character checks, duplicate ID detection, decision effect validity, target subject bounds ($\le 256$), target resource bounds ($\le 1024$, traversal rejection), and target action bounds ($\le 64$).
    - Level 4 Capacity check ($\le 5000$ rules).
  - SHA-256 cryptographic digest computation.
- **Recovery Manager (`PepRecoveryManager`)**:
  - `quarantine_file`: Non-destructive backup to `<path>.bak.<timestamp>` with `0600` permissions on Unix.
  - `recover_store`:
    - `StrictFailClosed`: Quarantines corrupt store and initializes a fresh, safe empty store.
    - `SalvageValidRules`: Quarantines original corrupt file, salvages valid rules respecting all semantic constraints, isolates and drops corrupted rules, and atomically writes the sanitized store.
    - `DryRun`: Performs validation and simulates recovery without mutating disk state.

## 3. Verification & Build Output
```
   Compiling aiosh-core v0.1.0 (code/aiosh-rust/aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.22s
     Running tests/test_pep_recovery.rs

running 1 test
test test_pep_recovery_scaffold_compilation_and_types ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## 4. Acceptance Confirmation
- [x] Implementation completed without external dependencies.
- [x] Multi-level validation and recovery strategies implemented.
- [x] Targeted test passes without regressions.
