# AIOS — Audit Ring Retention Research Note (Sprint 3)

> **Purpose.** Resolve the open Sprint-0 gap: the hash-chained audit
> ring (`audit_ring`, SQLite WAL) grows without bound. This note
> researches how tamper-evident, append-only logs handle retention in
> the real world, and derives the AIOS retention design:
> **checkpointed segment rotation + JSONL archives + per-segment Bloom
> filters**, implemented identically on both substrates (TypeScript CLI
> and Python MCP) under the existing canonical-JSON invariant.
>
> **Binding constraints.** AI_CONSTITUTION.md v1.1:
>
> - **P-2 (Audit immutability):** "The audit ring is append-only and
>   hash-chained. No process, including the AI itself, may rewrite or
>   delete a past entry."
> - **O-4:** "The audit ring is hash-chained: each row's hash includes
>   the previous row's hash. Truncation or insertion triggers alert +
>   verifier."
> - **C-4 (Auditability before capability gain):** the audit story of a
>   tool must verify end-to-end or the tool is disabled.
>
> Any retention mechanism that destroys or mutates entries would
> violate P-2/O-4. Retention must therefore be *archival rotation*, not
> deletion.

---

## 1. Problem statement

The shipped audit ring (Sprint 0, `code/aiosh-cli/src/audit.ts`,
`code/aiosh-mcp/aiosh_mcp/audit_client.py`):

- One SQLite table `audit_ring`, WAL journal mode, `synchronous=FULL`.
- Every consequential action (CLI subcommand, MCP tool call, agent
  step, grant lifecycle event) appends exactly one row.
- Each row carries `prev_hash` (hash of the previous row, or the
  genesis sentinel `0x00...0`) and `hash = SHA-256(prev_hash ||
  canonical_json(row_proto))`.
- `verify()` walks the **entire** table from genesis and recomputes
  every hash. `tail(n)` is index-bounded (`LIMIT n`).

Consequences of unbounded growth:

1. `verify()` is O(n) full-table scan + O(n) SHA-256 work; it gets
   slower every session.
2. The `.db` file grows forever (SQLite does not return pages to the
   OS without `VACUUM`).
3. Backups, sync, and cold-start cost grow linearly.
4. The agent's `aios.audit.tail` responses get noisier as history
   accumulates.

## 2. Prior art

### 2.1 Certificate Transparency — log rotation with final checkpoints

Authoritative source: **RFC 9162, "Certificate Transparency Version
2.0"** (Laurie, Messeri, Stradling; IETF, Dec 2021),
<https://www.rfc-editor.org/rfc/rfc9162.html>, and its predecessor
RFC 6962 (<https://www.rfc-editor.org/rfc/rfc6962.html>).

Established facts:

- A CT log is "a single, append-only Merkle Tree of submitted
  certificate and precertificate entries" (RFC 9162 §4).
- Logs have **finite lifetimes**. §4.13 "Shutting Down a Log"
  prescribes: stop accepting submissions → issue a **final Signed Tree
  Head (STH)** → publish it as a log parameter → keep serving the
  frozen data until its entries expire or exist in other logs. A
  successor log then starts fresh.
- The append-only property is enforced per-log via Merkle consistency
  proofs (§2.1.4): any later instance must prove it is a *superset* of
  any earlier instance. The final tree head is the **trust anchor**
  that binds the retired log to whatever follows.

**Lesson for AIOS:** the industry-standard way to bound an append-only
tamper-evident log is *rotation with a checkpoint*, never deletion.
The retired segment freezes at a final head hash; the new segment
chains from that head; verifiers treat the checkpoint as the segment's
trust anchor. This is exactly the structure that preserves P-2:
entries are never rewritten or destroyed — they move to cold storage
that is still cryptographically bound to the chain.

### 2.2 Blockchain pruning

Established fact: pruned Bitcoin nodes
(<https://bitcoin.org/en/full-node>, "Pruned full node") retain all
block *headers* — the hash chain — while discarding old block *bodies*.
Chain validity is still checkable from headers alone; bodies are
recoverable from peers/archives if ever needed. Ethereum's state
pruning follows the same principle (keep the commitment chain, discard
bulk data behind it).

**Lesson:** a small per-segment commitment (head hash + row count +
archive checksum) is enough to keep the tamper-evidence property of
rotated-out data, as long as the archive itself is retained and
re-verifiable.

### 2.3 Bloom filters for membership over cold data

Authoritative source: Bloom, B. H. "Space/Time Trade-offs in Hash
Coding with Allowable Errors", CACM 13(7), 1970; overview at
<https://en.wikipedia.org/wiki/Bloom_filter>.

Established facts:

- A Bloom filter answers set-membership with **no false negatives**
  and a tunable false-positive rate ε.
- Space: ~9.6 bits/element at ε = 1%; more generally
  `m = -n·ln(ε)/(ln 2)²` bits and optimal `k = (m/n)·ln 2` hash
  functions.
- No deletions (counting Bloom filters exist but cost 3-4×).
- In production: Bigtable/HBase/Cassandra/PostgreSQL use them to skip
  disk lookups for absent keys; Ethereum embeds a Bloom in every block
  header for log indexing; Grafana Tempo ships per-block Blooms.

**Lesson for AIOS:** after rotation, "was action hash H ever logged?"
should not require loading every archive. A per-segment Bloom filter
over row hashes answers it in O(k) with a documented false-positive
rate; a positive answer triggers a precise scan of that one archive
file (false positives are disambiguated there; false negatives cannot
occur).

### 2.4 SQLite mechanics

Established facts (SQLite documentation,
<https://www.sqlite.org/autoinc.html>,
<https://www.sqlite.org/lang_delete.html>,
<https://www.sqlite.org/wal.html>):

- `INTEGER PRIMARY KEY AUTOINCREMENT` ids are monotonically increasing
  and never reused; the high-water mark lives in `sqlite_sequence` and
  survives row deletion. So rotating rows out does not disturb id
  uniqueness or ordering.
- `DELETE` frees pages for reuse inside the DB file but does not
  shrink the file; `VACUUM` rewrites the file to reclaim space. WAL
  mode allows one writer + many readers concurrently.
- Both substrates already open the DB in WAL with
  `synchronous=FULL`, which is what makes an archival rotation durable
  in a single transaction.

**Lesson:** rotation can be a single SQLite transaction
(INSERT segment checkpoint, DELETE archived ids) followed by `VACUUM`
(optionally, out-of-band) to return disk space.

## 3. Options analysis

| Option | Bounds growth | Preserves P-2/O-4 | Verifiable history | Complexity |
|---|---|---|---|---|
| A. Do nothing | ✗ | ✓ | ✓ | none |
| B. Naive `DELETE` of old rows | ✓ | **✗ destroys entries; breaks chain at the seam (first survivor's prev_hash dangles)** | ✗ | low |
| C. Rewrite rows to compress (e.g. drop `args`) | partial | **✗ rewrite = hash change = chain break** | ✗ | medium |
| D. **Checkpointed rotation: archive rows byte-identical to JSONL, record segment checkpoint (ids, head hash, file sha256, Bloom), live ring continues from checkpoint head** | ✓ | ✓ entries preserved, never mutated | ✓ `verify --full` walks archives + live | medium |
| E. Bloom filter only, keep rows | ✗ | ✓ | ✓ | low (but solves wrong problem) |

B and C are constitutionally illegal (P-2: "no process ... may rewrite
or delete a past entry"; O-4: truncation triggers alert). E does not
bound storage. **D is the selected design** — it is the CT/RFC 9162
§4.13 pattern applied to a hash-chained SQLite ring.

## 4. First-principles check (REP Phase 7)

- *Why does the ring exist?* To be the only tamper-evident record of
  what the AI did (P-2 rationale). Retention must never weaken that.
- *Is unbounded growth necessary?* No. Tamper-evidence needs the chain
  of commitments and the entries to *exist somewhere verifiable* — it
  does not need them all resident in the hot write path.
- *Can we keep verifying forever?* Yes: `verify --full` replays
  archives segment-by-segment (each line re-hashed with the same
  canonical JSON), then continues into the live table. Cost is paid
  only when full-history proof is requested; routine `verify` walks
  only the live segment anchored at the latest checkpoint.
- *What can go wrong?* (1) Archive lost → full-history verify fails
  for that segment; mitigated by recording `archive_sha256` in the DB
  so loss/corruption is detected, not silent. (2) Rotation itself used
  as a covert erasure channel → mitigated by making rotation a PEP-
  gated, audited action whose own row records segment id, row count,
  head hash and archive checksum. (3) Cross-substrate divergence →
  mitigated by keeping the exact canonical-JSON invariant already
  proven in Sprint 0/1/2 and adding cross-language rotation smokes.

## 5. Selected design (summary; full contract in `docs/SPEC-AUDIT-RETENTION.md`)

1. **`audit_segments` table** — one row per rotated segment:
   `segment_id, closed_at, first_row_id, last_row_id, row_count,
   genesis_prev_hash, head_hash, archive_path, archive_sha256,
   bloom_m_bits, bloom_k, bloom_hex`.
2. **Archive files** — `$AIOSH_HOME/audit-archive/segment-<id>.jsonl`,
   one line per row: canonical JSON of `{...all hashed proto fields,
   id, hash}` — byte-identical to what was hashed, so archives
   re-verify offline.
3. **`rotate(keep_rows)`** — verify live chain → write archive
   (atomic tmp+rename) → one transaction: INSERT checkpoint + DELETE
   archived ids + INSERT `audit.rotate` row whose `prev_hash` is the
   checkpoint head (the rotation event is itself audited, per O-2).
4. **`verify(full=False)`** — anchored walk: live chain must start at
   the latest checkpoint `head_hash` (or genesis if no segments).
   `full=True` additionally replays every archive in segment order and
   checks inter-segment linkage + `archive_sha256`.
5. **Bloom per segment** — double-SHA256-derived k indices over row
   hashes; `seen(<hash>)` answers live-hit / segment-maybe /
   definitely-not-seen.
6. **Surfaces** — CLI: `aiosh audit rotate|segments|seen`, `audit
   verify --full`. MCP: `aios.audit.rotate` (PEP grant required —
   irreversible), `aios.audit.seen`, `aios.audit.verify` gains `full`.
7. **Cross-substrate parity** — identical schema, canonical JSON,
   Bloom index derivation, and archive line format in TS and Python;
   proven by a new smoke where each substrate rotates and the other
   verifies.

## 6. Sources

- RFC 9162, Certificate Transparency v2.0 — §4 (append-only log),
  §4.13 (Shutting Down a Log / final STH), §2.1.4 (consistency
  proofs). https://www.rfc-editor.org/rfc/rfc9162.html
- RFC 6962, Certificate Transparency v1.0.
  https://www.rfc-editor.org/rfc/rfc6962.html
- Bloom (1970), "Space/Time Trade-offs in Hash Coding with Allowable
  Errors", CACM 13(7). Overview:
  https://en.wikipedia.org/wiki/Bloom_filter
- Bitcoin Core docs — pruned full nodes.
  https://bitcoin.org/en/full-node
- SQLite docs — AUTOINCREMENT, DELETE, WAL.
  https://www.sqlite.org/autoinc.html ,
  https://www.sqlite.org/lang_delete.html ,
  https://www.sqlite.org/wal.html
- AIOS Constitution v1.1 — P-2, O-2, O-4, C-4
  (`mostimportanAIfolder/AI_CONSTITUTION.md`).
- ADR-0035 — S-rank agent architecture, §D-3 (one row per action),
  §D-4 (classifier gate).
- Shipped code: `code/aiosh-cli/src/audit.ts`,
  `code/aiosh-mcp/aiosh_mcp/audit_client.py`, `docs/SPRINT-0.md` §2
  (cross-substrate canonical-JSON proof).

*Established fact vs. design choice: everything in §2 is established
fact with cited sources; §5 is the AIOS design choice derived from it
(new in this sprint, not an upstream claim).*
