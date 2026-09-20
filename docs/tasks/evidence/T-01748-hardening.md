# T-01748: Hardware Detection Configuration Hardening

See detailed report in [T-01748-configuration-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01748-configuration-hardening.md).

- Enforced `ParentDir` (`..`) path traversal rejection in `HardwareConfig::validate()`.
- Added atomic temporary file + rename in `HardwareConfig::save_to_path()`.
- Added post-validation fallback guard to `HardwareConfig::from_env()`.
