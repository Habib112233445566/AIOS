# Research: T-02111 (PEP Decision Engine / core service: Research)

## 1. Executive Summary & Objective
This research establishes the architectural foundation and operational invariants for the **PEP Decision Service** (`PepDecisionService`), which acts as the stateful Policy Administration Point (PAP) and in-memory Policy Decision Point (PDP) for the AIOS Security Kernel.

## 2. Core Architectural Principles
1. **Thread-Safe Concurrent Evaluation**:
   - The decision engine is evaluated on every tool invocation, shell command, and IPC message. Evaluation must be lock-free or read-biased (`RwLock`/`ArcSwap`) to ensure sub-millisecond evaluation latency without blocking.
2. **Atomic Policy Updates**:
   - Ruleset loading, reload, and replacement must be atomic. Intermediate or partially-loaded rule states must never be exposed to concurrent evaluations.
3. **Multi-Index Optimization**:
   - Rules are indexed across three dimensions:
     - `by_subject`: Map from subject string or wildcard prefix to rule IDs.
     - `by_resource`: Map from resource URI prefix to rule IDs.
     - `by_action`: Map from action name to rule IDs.
4. **Audit Integration**:
   - Every evaluated decision produces an audit event structure compatible with `aios.audit.*` hash-chained WAL.

## 3. Invariants (`PEPSERV1..PEPSERV6`)

| Invariant | Name | Formal Rule |
|---|---|---|
| **`PEPSERV1`** | **Thread-Safe Concurrency** | The service must support concurrent evaluations across threads with $O(1)$ read contention. |
| **`PEPSERV2`** | **Atomic Policy Replacement** | Ruleset updates occur via atomic pointer swap or write-locked replacement; evaluations never observe intermediate states. |
| **`PEPSERV3`** | **Multi-Index Lookup** | Rules are indexed by subject, resource prefix, and action to prevent linear scans across large rulesets. |
| **`PEPSERV4`** | **Audit Trail Emission** | Every evaluation verdict emits an audit context record. |
| **`PEPSERV5`** | **Persistence & Non-Destructive Quarantine** | State persists atomically via temporary file replacement; corrupt files are quarantined. |
| **`PEPSERV6`** | **Capacity Enforcement** | The registry enforces a hard limit of `MAX_RULES_IN_SERVICE` (5,000 rules). |
