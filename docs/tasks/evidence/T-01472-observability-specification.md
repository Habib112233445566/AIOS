# T-01472: User Session Bootstrap — Observability: Specification

See formal specification in [T-01472-spec.md](T-01472-spec.md).

## Summary
- Defined contract for `SessionObservabilityReport` with fields covering inventory, state breakdown, seat allocation, scope distribution, type/class metrics, idle tracking, user distribution, and security policy compliance.
- Codified criteria `SSO1..SSO6` specifying exact arithmetic aggregation rules and string mapping helpers.
- Documented CLI subcommand `aiosh session stats` and MCP tool `aios.session.stats`.
