# T-00308 — Release Packaging & Backup: Recovery & Validation Hardening

## Overview
We hardened the recovery and validation functionality against resource exhaustion attacks, specifically focusing on zip-bomb mitigation.

## Hardening Implemented
1. **File Count Cap**: We enforced a strict limit on the number of files an archive can contain. If `archive.len() > 100_000`, the validation and restore processes abort with an explicit error. This prevents CPU exhaustion from iterating over massive central directories.
2. **Decompressed Size Cap**: In `restore_backup`, we track the cumulative decompressed size of the archive as we iterate through entries. If it exceeds 10 GB (the `MAX_UNCOMPRESSED_SIZE` limit), the extraction is immediately aborted.
3. **Bounded Reads**: To protect against scenarios where `file.size()` inside the zip metadata lies, we bound the `io::copy` operation using `Read::take`, restricting the bytes pulled into the `outfile` strictly up to `MAX_UNCOMPRESSED_SIZE`.
4. **Error Bubble**: The `zip::ZipArchive` library successfully maps internal structural or decompression errors, which we properly map into standard `Result::Err` envelopes rather than panicking or failing silently.

## Validation
- `cargo test` confirms all smoke suites and edge case tests continue to run successfully.
- No regression introduced, and explicit errors are correctly emitted under stress conditions.
- The task is structurally complete.
