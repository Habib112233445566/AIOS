# T-00474 — Documentation Index Control / security policy: Implementation

## 1. Implementation Scope
This task implements `check_doc_index_policy` in `code/aiosh-rust/aiosh-core/src/doc_index_service.rs` enforcing PEP policy token requirements for mutating actions and allowing unauthenticated read-only queries.

## 2. Implementation Details
- `check_doc_index_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String>`:
  - Invokes `crate::pep::is_irreversible(tool_name)`.
  - For mutating irreversible commands (`aios.doc.set`, `doc.set`), requires a non-empty PEP grant token.
  - Read-only actions (`aios.doc.index.get`, `aios.doc.check`, `aios.doc.search`, `doc.show`, `doc.check`, `doc.search`) execute without token gating.

## 3. Test Verification
```text
running 1 test
test doc_index_service::tests::test_check_doc_index_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out; finished in 0.13s
```
