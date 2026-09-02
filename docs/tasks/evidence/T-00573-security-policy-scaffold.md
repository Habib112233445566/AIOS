# T-00573 — Evidence & Audit Trail / security policy: Scaffold

## 1. Scaffold Scope
This task verifies and scaffolds the security policy interfaces, PEP grant validation hooks, and unit test signatures in `code/aiosh-rust/aiosh-core`.

## 2. Scaffold Implementation Details
- **`code/aiosh-rust/aiosh-core/src/evidence_service.rs`**:
  ```rust
  pub fn check_evidence_policy(grant: Option<&str>, tool_name: &str) -> Result<(), String> {
      match tool_name {
          "aios.evidence.record" | "evidence.record" | "aios.evidence.set" | "evidence.set" => {
              match grant {
                  Some(g) if !g.trim().is_empty() => Ok(()),
                  _ => Err("PermissionDenied: mutating evidence actions require a valid PEP grant".into()),
              }
          }
          _ => Ok(()),
      }
  }
  ```
- **`code/aiosh-rust/aiosh-core/src/pep.rs`**:
  - Registered `aios.evidence.record`, `evidence.record`, `aios.evidence.set`, `evidence.set` as irreversible mutating operations.

## 3. Test Verification
```text
running 1 test
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 151 filtered out; finished in 0.00s
```
