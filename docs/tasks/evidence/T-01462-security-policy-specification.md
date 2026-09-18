# T-01462: User Session Bootstrap — Security Policy: Specification

See formal specification document in [T-01462-spec.md](T-01462-spec.md).

## Summary
- Defined contract for `UserSessionSecurityPolicy`, `SessionPolicyMode`, `SessionPolicyViolation`, and `SessionPolicyVerdict`.
- Formulated criteria `SSP1..SSP7` covering UID bounds, root prevention, greeter isolation, seat0 remote protection, environment variable stripping, concurrency quotas, agent sandboxing, and policy modes.
- Established file loading cap of 64 KiB with stream-bounded ingestion and fail-closed validation.
