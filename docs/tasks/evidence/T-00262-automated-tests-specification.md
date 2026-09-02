# T-00262 — Automated Tests: Specification

## Objective
Specify the exact contract for backfilling automated tests for the Release Packaging & Backup configuration loader (`release_config.rs`).

## Test Cases (Inputs & Expected Outputs)

### 1. 64KB Read Bounding (OOM Prevention)
- **Input**: A temporary JSON configuration file padded to 100KB.
- **Expected Output**: The loader truncates the read at 64KB. Because the truncated JSON will be syntactically invalid (the trailing brace is cut off), `load_config` must return `Err("Malformed release config: ...")`.
- **Persistence Effect**: The temporary file must be deleted after the test.

### 2. Output Directory Path Traversal Rejection
- **Input**: A valid JSON configuration file where `output_dir` is set to `../../../etc/`.
- **Expected Output**: `load_config` must return `Err` indicating illegal characters or an absolute path.
- **Persistence Effect**: Temporary file cleanup.

### 3. Absolute Path Rejection
- **Input**: A valid JSON configuration file where `output_dir` is set to `/var/backups` or `C:\\Backups`.
- **Expected Output**: `load_config` must return `Err` indicating an absolute path was detected.
- **Persistence Effect**: Temporary file cleanup.

### 4. Happy Path Configuration
- **Input**: A valid JSON configuration file containing: `{"max_file_size_bytes": 104857600, "output_dir": "custom_output"}`
- **Expected Output**: `Ok(ReleaseConfig)` with the correctly parsed values.
- **Persistence Effect**: Temporary file cleanup.

## Implementation Contract
- **Existing Interfaces Reused**: We will use the existing `load_config(path: Option<&str>)` interface.
- **New Interfaces**: None.
- **Framework**: `cargo test` using standard Rust `#[test]` modules in `aiosh-core/src/release_config.rs`. No external integration dependencies required.

## Audit Effects
- Configuration loading is read-only and does not emit audit ring rows. The tests will not generate audit entries.
