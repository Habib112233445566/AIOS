# T-00329 — Dependency & Toolchain Pinning: core service Documentation

## 1. Overview
This task documents the `toolchain_service` core module to make it discoverable and usable for operators and agents.

## 2. Documentation Updates
The core service specification (`docs/tasks/evidence/T-00322-core-service-specification.md`) was updated with a new section, "7. Implementation & Usage Details".

### Included Elements:
1. **Invocation**: Documented that it is wired into `aiosh-cli` as `aiosh toolchain check`.
2. **Example**: Provided a copy-pasteable execution command (`cargo run --bin aiosh -- toolchain check`) alongside an example JSON standard output envelope.
3. **Constraints and Limitations**: Honestly recorded the limitations regarding process timeouts (5000ms), `PATH` spoofing vulnerabilities (since it relies on resolving `rustc`/`python3` through `PATH`), the fact that hash enforcement is currently a no-op, and Node.js being optional.
4. **Evidence Links**: Added direct links to all T-00322 to T-00329 task evidence files for traceability.

## 3. Acceptance Criteria Met
- Docs updated with working example.
- Limitations stated, not omitted.
