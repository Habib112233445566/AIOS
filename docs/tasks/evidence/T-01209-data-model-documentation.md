# T-01209: Package Management - Data Model: Documentation

## Metadata
- **Task ID:** `T-01209`
- **Subsystem:** `code/aiosh-rust/aiosh-core::package`
- **Component:** Package Management Data Model Documentation
- **Status:** Complete

## 1. Summary of Delivered Capabilities
The Package Management Data Model provides a unified, cross-distribution abstraction layer for Debian (APT/dpkg) and Alpine (APK) package systems:

1. **Core Data Structures (`code/aiosh-rust/aiosh-core/src/package.rs`)**:
   - `PackageFormat`: `Deb`, `Apk`, `Flatpak`, `Tarball`.
   - `PackageState`: `Available`, `Installed`, `Upgradable`, `PendingInstall`, `PendingRemoval`, `Broken`.
   - `PackageDependency`: Name, optional version constraint, optional flag.
   - `PackageSpec`: Full package specification including architecture, format, size, checksum, repository URL, and dependencies.
   - `PackageActionType` & `PackageAction`: Atomic discrete operations (`Install`, `Remove`, `Upgrade`, `Purge`).
   - `PackageTransaction`: Atomic batch execution container with dry-run capabilities and size delta calculation.
   - `PackageQuery`: Structured filtering container for searching package inventories.

2. **Validation Invariants (PM1..PM5)**:
   - `PM1`: Package naming syntax conforming to `^[a-z0-9][a-z0-9+.-]*$`, length `1..=128`.
   - `PM2`: Strict size bounds: version `<= 64`, architecture graphic ASCII, description `<= 4096` bytes, dependencies `<= 256`, package size `<= 100 GiB`.
   - `PM3`: Dependency hygiene: no self-dependencies, duplicate dependencies, or malformed constraints.
   - `PM4`: Checksum and provenance: exact 64-character hexadecimal SHA-256 digests and HTTPS repository URL enforcement.
   - `PM5`: State consistency: installed packages must possess positive size (`installed_size_bytes > 0`).

3. **Operator CLI Surface (`aiosh package`)**:
   - `aiosh package validate --name <name> [--json]`: Validate package name syntax.
   - `aiosh package validate --spec <file_or_inline_json> [--json]`: Deep-audit package specification with 1 MiB size cap.

4. **Autonomous Agent MCP Surface (`aios.package.validate`)**:
   - Accepts `{ "name": string }` or `{ "spec": object }`, returns standard response envelope.

## 2. Operator CLI Usage Examples

### Validating Package Name Syntax (PM1)
```bash
aiosh package validate --name curl
# VALID: Package name 'curl' conforms to PM1 naming syntax

aiosh package validate --name "bad name" --json
```
Output:
```json
{
  "code": 2,
  "data": {
    "name": "bad name",
    "valid": false
  },
  "error": {
    "code": "VALIDATION_FAILED",
    "errors": [
      "package name contains invalid character ' ' in 'bad name'"
    ],
    "message": "Package name 'bad name' is invalid: package name contains invalid character ' ' in 'bad name'"
  }
}
```

### Validating Complete Package Specification (PM1..PM5)
```bash
aiosh package validate --spec '{
  "name": "curl",
  "version": "8.5.0-2",
  "architecture": "x86_64",
  "format": "deb",
  "state": "available",
  "description": "command line tool for transferring data with URL syntax",
  "installed_size_bytes": 450000,
  "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "repository_url": "https://deb.debian.org/debian",
  "dependencies": []
}' --json
```

## 3. MCP Tool Invocation Example
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.package.validate",
    "arguments": {
      "name": "libc6"
    }
  }
}
```

Response:
```json
{
  "code": 0,
  "data": {
    "message": "Package name 'libc6' conforms to PM1 naming syntax",
    "name": "libc6",
    "ok": true,
    "tool": "aios.package.validate",
    "valid": true
  },
  "error": null
}
```

## 4. Constraints & Known Limitations
- The data model defines and validates package schemas and transactions; execution against host `apt-get` / `apk` package managers is deferred to subsequent sub-epics.
- Package specifications are constrained to a maximum of 256 dependencies and 100 GiB individual size ceiling.
- Spec file inputs are capped at 1 MiB (`1,048,576` bytes) to prevent resource exhaustion attacks.
