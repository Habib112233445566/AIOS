# T-01824: Network Bootstrap / CLI Surface: Implementation

## Overview
- **Task ID**: `T-01824`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Implement the minimal working behavior for the CLI surface of Network Bootstrap.

## Implementation Details
1. **Command Handler `cmd_network` in `code/aiosh-rust/aiosh-cli/src/main.rs`**:
   - Integrated with `aiosh_core::network_service::NetworkService` and `aiosh_core::network::validate_interface_name`.
   - Supported subcommands:
     - `list`: Calls `service.scan_interfaces()`, displays table in human mode or `{ interfaces, count }` in JSON mode.
     - `show <iface>`: Validates `<iface>` via `validate_interface_name`, queries `service.get_interface(name)`, displays details or error.
     - `routes`: Calls `service.scan_routes()`, displays destination, gateway, interface, metric.
     - `dns`: Calls `service.get_dns_config()`, displays nameservers and search domains.
     - `state`: Calls `service.get_network_state()`, displays complete host snapshot.
     - `up <iface>`: Validates `<iface>`, calls `service.bring_up(name)`, returns success/failure.
     - `down <iface>`: Validates `<iface>`, calls `service.bring_down(name)`, returns success/failure.
2. **Security & Invariants Maintained**:
   - PEP Gating & Audit Logging: Every path emits a structured audit record via `classify_and_emit(&mut ctx, "network", ...)`.
   - Path Hygiene: `--sysfs`, `--procfs`, `--resolv` flags are validated: length $\le 1024$ characters and no control characters; violations return exit code 2 and emit failure audit records.
   - Positional Name Sanitization: Interface names validated to prevent injection or directory traversal.
   - Terminal Sanitization: All user/system output passed through `sanitize_terminal` to neutralize ANSI escape codes.
   - Uniform JSON Output: `--json` outputs standard `{ "code": <int>, "data": ..., "error": ... }` envelopes.

## Verification
- `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli` compiles with 0 errors.
