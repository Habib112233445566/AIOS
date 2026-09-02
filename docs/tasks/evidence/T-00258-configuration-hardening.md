# T-00258 — Configuration: Hardening

## Hardening Applied
- **Size Cap on Config File**: Replaced `fs::read_to_string` with `fs::File::open` and `std::io::Read::take(65_536).read_to_string(...)`. This bounds the maximum bytes read from the config path to 64KB, preventing OOM / Denial of Service.
- **Path Injection Prevention**: Added explicit validation to `cfg.output_dir` to reject `..` (path traversal), absolute paths (leading `/`, `\`), and Windows drive letters (`:`). This ensures the output directory stays relative and safe.
- **Errors**: Standard result envelopes (`Result<ReleaseConfig, String>`) are preserved. If loading/validating fails, an explicit `Err` string is returned instead of silently defaulting.
- **Resource Cleanup**: Used idiomatic Rust standard library which automatically cleans up file handles when `f` is dropped (even on early return via `?`).

## Acceptance Validation
- File reading is strictly bounded to 64KB.
- Attempting to use a malicious `output_dir` will now return a hard error, failing safely.
- No DB connections or temp files are leaked on the error path because Rust's RAII cleans up the local file descriptor automatically.
