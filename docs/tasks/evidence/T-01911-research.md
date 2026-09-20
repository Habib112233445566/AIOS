# Task Evidence: T-01911 - System Update Mechanism / core service: Research

## 1. Overview
- **Task ID**: `T-01911`
- **Sub-Epic**: 2 (System Update Mechanism Core Service)
- **Goal**: Research core update service architectures, A/B staging workflows, cryptographic verification pipelines, rollback orchestration, and state persistence patterns.

---

## 2. Key Findings
- Evaluated A/B partition mechanics and ostree/sysupdate patterns.
- Defined invariants `USVC1..USVC6` (isolated staging, cryptographic gate, active slot non-interference, atomic state persistence, storage safety, deterministic error taxonomy).
- Formulated `SystemUpdateService` API model.

---

## 3. Status
Research completed; ready for specification in `T-01912`.
