# Task Evidence: T-01916 - System Update Mechanism / core service: Integration

## 1. Overview
- **Task ID**: `T-01916`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Author and execute cross-surface Python integration smoke tests for `SystemUpdateService` enforcing invariants `USVC1..USVC6`.

---

## 2. Results Summary
- Integration test file: `code/aiosh-cli/tests/test_system_update_service_smoke.py`
- Invariants verified: `USVC1..USVC6` (staging sandboxing, SHA-256 digest validation, active slot isolation, atomic persistence, rollback, boot confirmation).
- Result: **5/5 tests passed in 0.10s**.
