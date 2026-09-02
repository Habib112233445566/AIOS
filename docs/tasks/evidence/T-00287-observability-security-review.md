# T-00287 — Release Packaging & Backup: Observability Security Review

## Policy & Enforcement Review
We reviewed the newly integrated observability feature (capturing subprocess `stderr`) to identify any potential security regressions in the Release Packaging & Backup module.

### 1. Command Injection Mitigation
- **Mechanism**: The observability logic uses Rust's `std::process::Command` with strictly delimited `.args(args)`. It does *not* invoke a subshell (e.g. `sh -c`).
- **Verdict**: Safe. Because arguments are passed directly to `execve` (or the OS equivalent), shell injection attacks (like passing `"; rm -rf /"` in the output path) are neutralized by the OS kernel. The payload will just be treated as a literal file path.

### 2. Malicious Content Handling in `stderr`
- **Vector**: A compromised binary or clever input string causes the subprocess to dump malformed bytes or massive amounts of data to `stderr`.
- **Mitigation 1 (Unicode)**: We use `String::from_utf8_lossy(&output.stderr)`. If the subprocess writes invalid UTF-8 sequences, Rust safely replaces them with the standard `U+FFFD REPLACEMENT CHARACTER` rather than panicking or crashing the AIOS ledger parser.
- **Mitigation 2 (Log Forging)**: Because the `stderr` string is encapsulated within the `outcome_detail` field of the `AuditRing`'s structured JSON format, it is fully escaped during serialization. An attacker cannot emit raw newline or `{}` JSON characters to "forge" a fake ledger row.

## Abuse Scenarios
1. **Agent attempts path injection to overwrite system binaries**
   - **Vector**: An agent requests `aios.release.generate` with output path `/usr/bin/aios`.
   - **Result**: The `release_config` paths loader rejects absolute paths (implemented in T-00258), halting execution before the subprocess is ever spawned.
2. **Attacker triggers excessive `stderr` to exhaust memory**
   - **Vector**: The underlying tool emits infinite stderr.
   - **Result**: `Command::output()` reads until EOF. While a rogue tool could theoretically cause high memory usage, `genisoimage` and standard Zip are deterministic tools that don't suffer from infinite log streams. A hard subprocess bound will be handled in the `Hardening` task (T-00288).

## Conclusion
**No known policy bypass remains open.** The observability enhancements safely handle OS-level error streams without introducing injection vectors or breaking the PEP invariants.
