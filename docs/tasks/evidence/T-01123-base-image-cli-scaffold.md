# T-01123 — Base Image Build / CLI Surface: Scaffold

**Date:** 2026-09-03
**Type:** Scaffold
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / CLI Surface

## 1. Scaffold Deliverables
- Scaffolded `cmd_image(args: &[String]) -> i32` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Added top-level dispatch `Some("image") => cmd_image(&args[1..])`.
- Connected `aiosh image list` handler and help text.
- Checked full workspace compilation (`cargo check`).
