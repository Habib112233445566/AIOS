# T-01642: Kernel Module Management — Configuration: Specification

## Metadata
- **Task ID:** `T-01642`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Kernel Module Management
- **Status:** Complete — specification.
- **Date:** 2026-09-19
- **Dependencies:** `T-01641` (Research).
- **Feeds:** `T-01643` — "configuration: Scaffold".
- **Artifacts:** `docs/tasks/evidence/T-01642-configuration-specification.md` and `docs/tasks/evidence/T-01642-spec.md`.

---

## 1. Specification Overview

This specification formalizes the configuration subsystem contract for Kernel Module Management in AIOS, covering the canonical JSON document format, validation rules, import/export interfaces, and persistence behaviors.

---

## 2. Configuration Schema & Types

### 2.1 Canonical Configuration Schema (`KernelModuleConfig`)
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "KernelModuleConfig",
  "type": "object",
  "required": ["id", "description", "rules", "autoload_modules", "created_at"],
  "additionalProperties": false,
  "properties": {
    "id": {
      "type": "string",
      "minLength": 1,
      "maxLength": 64,
      "pattern": "^[a-zA-Z0-9_-]+$"
    },
    "description": {
      "type": "string",
      "maxLength": 256
    },
    "rules": {
      "type": "array",
      "maxItems": 1024,
      "items": {
        "oneOf": [
          {
            "type": "object",
            "required": ["blacklist"],
            "additionalProperties": false,
            "properties": {
              "blacklist": {
                "type": "object",
                "required": ["module"],
                "properties": {
                  "module": { "type": "string", "maxLength": 64, "pattern": "^[a-zA-Z0-9_-]+$" }
                }
              }
            }
          },
          {
            "type": "object",
            "required": ["options"],
            "additionalProperties": false,
            "properties": {
              "options": {
                "type": "object",
                "required": ["module", "options"],
                "properties": {
                  "module": { "type": "string", "maxLength": 64, "pattern": "^[a-zA-Z0-9_-]+$" },
                  "options": {
                    "type": "array",
                    "items": { "type": "string", "maxLength": 128 }
                  }
                }
              }
            }
          },
          {
            "type": "object",
            "required": ["install"],
            "additionalProperties": false,
            "properties": {
              "install": {
                "type": "object",
                "required": ["module", "command"],
                "properties": {
                  "module": { "type": "string", "maxLength": 64, "pattern": "^[a-zA-Z0-9_-]+$" },
                  "command": { "type": "string", "maxLength": 256 }
                }
              }
            }
          }
        ]
      }
    },
    "autoload_modules": {
      "type": "array",
      "maxItems": 256,
      "items": {
        "type": "string",
        "maxLength": 64,
        "pattern": "^[a-zA-Z0-9_-]+$"
      }
    },
    "created_at": {
      "type": "string"
    }
  }
}
```

---

## 3. Configuration Invariants (CFG-KM1..CFG-KM5)

- **CFG-KM1 (Identifier & Identity)**: Every configuration must possess a unique non-empty alphanumeric `id`.
- **CFG-KM2 (Syntax Validation)**: All module names must match `^[a-zA-Z0-9_-]+$`. All option parameters must be valid key-value pairs or tokens without shell metacharacters or whitespace.
- **CFG-KM3 (Mutual Exclusion)**: An autoloaded module cannot be simultaneously blacklisted or disabled via install directive. A blacklisted or install-disabled module cannot be added to `autoload_modules`.
- **CFG-KM4 (Atomic Persistence)**: Serialized store documents must not exceed 10 MiB (`MAX_MODULE_DOC_BYTES`). Saves must write to a sibling temporary file, fsync, and atomically rename over the target.
- **CFG-KM5 (Bidirectional Interoperability)**:
  - Export generates standard `/etc/modprobe.d/*.conf` and `/etc/modules-load.d/*.conf` directives.
  - Import parses standard modprobe and modules-load configuration files, stripping comments and validating directives into the canonical store.

---

## 4. Import & Export Interfaces

```rust
pub trait KernelModuleConfigIO {
    fn import_modprobe_conf(&mut self, content: &str) -> Result<usize, String>;
    fn import_modules_load_conf(&mut self, content: &str) -> Result<usize, String>;
    fn export_modprobe_conf(&self) -> String;
    fn export_modules_load_conf(&self) -> String;
}
```

---

## 5. Error Codes & Envelopes

| Error Code | Description |
|---|---|
| `CONFIG_PARSE_ERROR` | Malformed JSON or syntax violation in modprobe.d / modules-load.d source |
| `CONFLICT_AUTOLOAD_BLACKLIST` | Module cannot be autoloaded and blacklisted simultaneously |
| `INVALID_MODULE_NAME` | Name does not match `^[a-zA-Z0-9_-]+$` |
| `INVALID_PARAMETER` | Parameter syntax invalid |
| `DOCUMENT_TOO_LARGE` | Store document exceeds 10 MiB limit |
