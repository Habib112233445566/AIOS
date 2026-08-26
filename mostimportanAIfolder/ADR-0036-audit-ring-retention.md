# ADR-0036 — Audit Ring Retention: Checkpointed Segment Rotation

| Field | Value |
|---|---|
| **Status** | Accepted (binding) |
| **Date** | 2026-08-21 |
| **Authors** | AIOS Sprint 3 session |
| **Replaces** | — |
| **Supersedes** | — |
| **Index** | Entry **#36** in the project's ADR numbering; successor to ADR-0035 per its amendment procedure. |
| **Companion docs** | `docs/research/AIOS-AUDIT-RING-RETENTION-2026-08-21.md` (research note), `docs/SPEC-AUDIT-RETENTION.md` (formal spec), `mostimportanAIfolder/AI_CONSTITUTION.md` (P-2, O-2, O-4, C-4), `mostimportanAIfolder/ADR-0035-aios-s-rank-agent-architecture.md` (§D-3 audit-row invariant). |

---

## 1. Context

ADR-0035 §D-3 and Constitution P-2 make the **audit ring** the only
tamper-evident record of what the AIOS S-rank subsystem does: one
hash-chained row per consequential action, append-only, verifiable by
walking the chain from genesis (`SHA-256(prev_hash || canonical_json(row))`).

Since Sprint 0 the ring has been **unbounded**: every session appends rows
forever, `verify()` is an O(n) full-table re-hash, and the SQLite file never
shrinks. This was logged as an open gap in Sprint 0 and is the first work
item of Sprint 3 ("agent-loop hardening + audit-ring retention").

The naive fix — delete old rows — is constitutionally illegal:

- **P-2:** "No process, including the AI itself, may rewrite or delete a
  past entry."
- **O-4:** "Truncation or insertion triggers alert."

So retention needs a mechanism that **bounds the live store without
destroying or mutating any entry**.

## 2. Decision

### D-1 — Retention by checkpointed segment rotation (never by deletion)

Adopt the Certificate-Transparency log-retirement pattern
(**RFC 9162 §4.13, "Shutting Down a Log"**: freeze → publish final tree
head → successor log continues):

1. `rotate(keep_rows)` verifies the live chain, then **archives** the oldest
   `count − keep_rows` rows byte-identically (canonical JSON, same fields
   that were hashed) to `$AIOSH_HOME/audit-archive/segment-<id>.jsonl`.
2. A checkpoint row in a new `audit_segments` table records
   `{segment_id, first/last row id, row_count, genesis_prev_hash,
   head_hash, archive_path, archive_sha256, bloom_m_bits, bloom_k,
   bloom_hex}`.
3. The archived rows are removed from the **live table only** — they remain
   on disk, unmutated, and re-verifiable. The archive's `sha256` is pinned
   in the checkpoint, so archive loss or corruption is *detected*, never
   silent.
4. The live ring continues unbroken: the first retained row's `prev_hash`
   already equals the segment `head_hash`, and the `audit.rotate` event row
   itself chains onto the retained tail.
5. Rotation is **refused if the live chain is broken** — retention must
   never be usable to launder a tampered chain.

**Why binding:** this is the only option that satisfies P-2/O-4 *and*
bounds growth. Rejected alternatives (documented in the research note §3):
naive `DELETE` (destroys entries, breaks the chain at the seam), row
rewriting/compression (changes hashes → chain break), and "keep
everything" (does not solve the problem).

### D-2 — `verify()` is anchor-aware; `verify --full` replays archives

- Default `verify()` walks only the **live segment**, anchored at the newest
  checkpoint `head_hash` (or genesis when no segment exists). A live chain
  that starts at neither the anchor nor genesis reports `broken_at` — this
  preserves O-4's truncation-detection property across rotations.
- `verify(full=True)` additionally replays every archive file in segment
  order: file `sha256` vs. checkpoint, per-line re-hash, inter-segment
  linkage (`segment N genesis_prev_hash == segment N−1 head_hash`), then the
  live walk. Full-history proof costs I/O only when explicitly requested.

### D-3 — Per-segment Bloom filter answers "was this action ever logged?"

Each checkpoint stores a Bloom filter over the archived row hashes
(`m ≥ 16 bits/item, k = 8`, index `i` = `sha256(f"{i}:{hash}")[:8]` as
big-endian u64 mod m; deterministic and byte-identical across substrates).
`seen(<hash>)` returns `live` (exact match), `maybe` (Bloom hit, refined to
`archive` by exact scan on request), or `no` (definitive negative — Bloom
filters have no false negatives, Bloom 1970).

**Why:** post-rotation, historical membership queries must not require
loading every archive for routine checks; the Bloom pre-filter is the
standard pattern (Bigtable/HBase/Cassandra, Ethereum block logs).

### D-4 — Rotation is a gated, audited action

- Every rotation appends exactly one `audit.rotate` row (O-2) carrying
  `{segment_id, row_count, first/last ids, head_hash, archive_sha256,
  archive_path}` — rotation can never be a covert erasure channel; the
  event is in-band and itself hash-chained.
- On the MCP surface, `aios.audit.rotate` **requires an explicit PEP
  grant** (same explicit-grant pattern as `aios.fs.read`; classifier gate
  runs first per ADR-0035 §D-4 ordering). It mutates the audit store →
  C-3-style caution flag on its row.

### D-5 — Cross-substrate parity is part of the contract

`audit_segments` DDL, archive line format, canonical JSON, Bloom index
derivation, and the rotation-row proto are implemented identically in
TypeScript (`code/aiosh-cli/src/retention.ts`) and Python
(`code/aiosh-mcp/aiosh_mcp/retention.py`), extending the Sprint-0
canonical-JSON invariant. The retention smoke proves it both directions:
Python rotates → TS verifies, TS rotates → Python verifies, and Bloom
answers agree across substrates.

## 3. Consequences

- Live DB size is bounded by `keep_rows` + post-rotation traffic; `VACUUM`
  can reclaim file space out-of-band (not auto-run, to avoid write-path
  stalls).
- Full-history audit requires the archive files; their checksums are pinned
  in the DB, so missing/corrupt archives fail `verify --full` loudly.
- `tail()` on a mid-chain window now returns stored `prev_hash` values
  (the Sprint-0 hack that forced the window's first row to GENESIS is
  removed; after rotation it would have produced false data).
- Future work (queued, not in this ADR): automatic rotation policy
  (time/size triggers on the write path), archive export/signing for
  off-host cold storage, counting-Bloom variant if revocation-style
  membership removal is ever needed.

*ADR-0036 is binding as of 2026-08-21.*
