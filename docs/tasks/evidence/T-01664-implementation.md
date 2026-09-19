# T-01664: Implementation

See full implementation details in [T-01664-security-policy-implementation.md](./T-01664-security-policy-implementation.md).
Features implemented:
- Invariants SP-KM1..SP-KM6 in `KernelModuleSecurityPolicy`.
- `validate()`, `evaluate_rule()`, `evaluate_autoload()`, `evaluate_store()`.
- Clean compilation in `aiosh-core`.
