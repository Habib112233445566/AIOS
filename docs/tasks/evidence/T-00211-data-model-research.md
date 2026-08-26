# T-00211 — Phase 0 — Release Packaging & Backup / Data Model: Research

## Goal
Establish facts, constraints, and prior art for the data model of Release Packaging & Backup.

## Facts and Prior Art (Release Packaging)
1. **Traceability & Configuration Management**: A release packaging data model centers around artifacts, dependencies, and configuration data. It provides immutability for shipped versions (e.g., ISOs, APT packages, tarballs) [1, 2].
2. **Debian/Ubuntu Packaging**: Since Pillar A relies on a Debian 13 / Ubuntu 24.04 LTS host, release packaging entails creating reproducible ISOs and deterministic package manifests (e.g., `apt` sources, `.deb` packaging) [3].
3. **Artifact Integrity**: The current backup mechanism (`zip` and `workspace.py sync` to R2) relies on SHA-256 hash validation to ensure the integrity of the release artifacts.
4. **Metadata Storage**: Release packaging involves metadata linking build IDs, environments, and artifacts. The project manifests (`PROJECT_MANIFEST.yaml`) currently track version state and task dependencies.

## Facts and Prior Art (Backup Data Model)
1. **Workspace Backup**: T-00008 verified a `zip` and `workspace.py sync` pipeline that pushes changes to an external R2 bucket.
2. **Audit Ring Retention**: Sprint 3 implemented an audit-ring retention policy (rotation + bloom filter) for immutable logging. Backups of the system state must include these audit archive segments (`audit-archive/segment-<id>.jsonl`) [ADR-0036].
3. **Semantic Memory**: The AIOS uses an encrypted SQLite database for per-engagement semantic memory. Any backup data model must support transactional backups of SQLite databases.

## Assumptions vs. Facts
* **Assumption**: The backup data model only needs to support simple zip files.
* **Fact**: The backup data model must support transactional consistency for SQLite (audit ring, semantic memory) and preserve metadata (checksums, file permissions, PEP grants) during packaging and restore operations.
* **Assumption**: Release packaging is just an ISO generator.
* **Fact**: Release packaging requires a deterministic build environment, signing (e.g., GPG for apt repositories), and tracking artifact dependencies (KDE Plasma 6, Wine 11.x, Proton 11.x).

## Unknowns & Decisions Needed
1. **Snapshot Mechanism**: Will system backups use block-level snapshots (e.g., Btrfs/ZFS, LVM) or file-level snapshots (`rsync`, `tar`, `zip`)?
2. **Release Format**: Do we deliver AIOS as a complete Debian/Ubuntu ISO image (Live CD/Installer) or as an overlay package (an `apt` repository + install script `aiosh install`)?
3. **Secrets Management**: How will the backup data model handle the encryption keys for the semantic memory SQLite database and PEP capability grants?
4. **Retention Policies**: Should backups implement a tiered retention policy (similar to the audit ring's segment rotation), or rely entirely on external R2 bucket lifecycle rules?

## Citations
[1] *Software Release Packaging Data Models* - Traceability and immutability in CI/CD configuration management.
[2] *Debian Repository Format* - Standard for `apt` package metadata and checksums.
[3] *AIOS Project Manifest & Roadmap* - Identifies Debian 13/Ubuntu 24.04 as host OS and requirements for reproducible installation (`aiosh install`).
