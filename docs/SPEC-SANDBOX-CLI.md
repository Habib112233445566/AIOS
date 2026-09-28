# Sandbox Enforcement CLI Specification (`aiosh sandbox` / `aiosh sb`)

## 1. Overview
The Sandbox Enforcement CLI surface provides operators and orchestration agents with unified control over containment profiles, host kernel capability probing, and supervised sandboxed process execution.

---

## 2. Command Synopsis

```text
aiosh sandbox <subcommand> [options]
aiosh sb <subcommand> [options]
```

### Subcommands
1. `profiles [--json]`: Lists all pre-configured and dynamically registered sandbox profiles.
2. `probe [--json]`: Probes host kernel support for Landlock LSM, ABI versions, Seccomp-BPF filters, and `no_new_privs`.
3. `exec [--profile <NAME>] [--grant <TOKEN>] [--cwd <PATH>] [--json] -- <BIN> [ARGS...]`: Executes a command under sandbox containment.

---

## 3. Usage Examples

### Listing Available Profiles
```bash
# Human readable
aiosh sandbox profiles

# JSON format
aiosh sandbox profiles --json
```

### Probing Kernel Containment Capabilities
```bash
aiosh sandbox probe --json
```

### Executing Under Sandbox Containment
```bash
# Permissive profile
aiosh sandbox exec --profile permissive -- python -c "print('hello')"

# Standard profile with specific working directory
aiosh sandbox exec --profile standard --cwd /tmp -- ls -la

# Enforcing PEP capability authorization
aiosh sandbox exec --grant grant_tok_12345 -- /usr/bin/tool --flag
```

---

## 4. Error Handling & Exit Codes
- `0`: Child process completed successfully with exit code 0.
- `1`: PEP authorization denied (`ERR_SANDBOX_PEP_UNAUTHORIZED`).
- `2`: Invalid argument, missing `--` delimiter, directory traversal in `--cwd`, or unknown profile.
- `127`: Executable not found or failed to spawn.
- `N`: Non-zero exit code returned directly by child process.

---

## 5. Constraints & Limitations
- Directory traversal sequences (`..`) in `--cwd` are strictly forbidden.
- Command flags must precede the `--` delimiter. Arguments following `--` are passed verbatim to the target binary.
- On non-Linux platforms (e.g. Windows/macOS), kernel containment features (Landlock, Seccomp-BPF) report unavailable, and processes execute with process-level isolation and resource limit tracking.
