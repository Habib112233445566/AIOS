# Task Evidence: T-02148 (Hardening)

See [T-02148-configuration-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02148-configuration-hardening.md) for full details.
- Size caps: 64 KiB config read limit, 50,000 max rules, 100 MB max store size.
- Standard error envelopes with taxonomy `PEPCONF_ERR_*`.
- Atomic persistence with `.tmp.<pid>` and symlink rejection.
- Fail-loud configuration error reporting in CLI.
