# T-00254 — Configuration: Implementation
Wired `release_config.load_config()` into `release.py`:
- `generate_release` reads `output_dir` from config instead of hardcoded `output/release`.
- `physical_create_zip` reads `max_file_size_bytes` from config instead of hardcoded 2GB.
- All tests still pass (9/9).
