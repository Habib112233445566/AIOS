# T-00457 — Documentation Index Control / configuration: Security Review

## 1. Review Scope
This security review evaluates the `DocIndexConfig` structure and parsing logic in `code/aiosh-rust/aiosh-core/src/doc_index_config.rs`.

## 2. Threat Scenarios & Mitigations

### 1. Host Traversal via Root Directories
- **Threat**: A config file specifies root directories like `../../../../sensitive` to expand indexing scope outside project boundaries.
- **Mitigation**: `DocIndexConfig::validate()` explicitly forbids any path containing `..` in `root_dirs`.

### 2. Denial of Service via Huge Configuration Files
- **Threat**: An attacker supplies a 1 GB configuration file to consume system RAM.
- **Mitigation**: `DocIndexConfig::from_path()` limits file ingestion to 64 KiB (`MAX_CONFIG_BYTES`) via `Read::take()`.

### 3. Untrusted Ingestion of Extension Rules
- **Threat**: Config includes malicious file extensions or regex wildcards.
- **Mitigation**: `DocIndexConfig::validate()` requires all extensions to begin with `.` and enforces non-empty string constraints.

## 3. Verdict
All identified risk vectors are mitigated. The configuration subsystem is safe and hardened.
