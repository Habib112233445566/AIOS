# Task Evidence: T-02087 (documentation: Security Review)

## Overview
- **Task ID**: T-02087
- **Sub-Epic**: Sub-Epic 9: Capability Documentation Subsystem
- **Component**: `aiosh-core::capability_doc`
- **Objective**: Conduct formal security review and threat modeling for the Capability Documentation Subsystem.

## Threat Model & Security Invariants

| Threat ID | Threat Vector | Impact | Mitigation Strategy |
|---|---|---|---|
| **`THREAT-CAPDOC-01`** | **Query Length Exhaustion (DoS)** | Excessive CPU consumption or memory pressure via massive query payloads. | Enforce `MAX_DOC_QUERY_LEN = 256`. Reject queries exceeding bound immediately in $O(1)$. |
| **`THREAT-CAPDOC-02`** | **Multibyte UTF-8 Slicing Panic (N-21 / H-6 class)** | Process panic/abort when snippet extraction slices across multibyte UTF-8 code points (e.g. CJK, emojis). | `extract_utf8_snippet` uses `char_indices()` to align slicing offsets to valid character boundaries, guaranteeing zero panic. |
| **`THREAT-CAPDOC-03`** | **Control Character & Whitespace Evasion** | Terminal escaping or injection via embedded `\0`, `\r`, `\n`, ANSI sequences. | `get_topic` and `search` inspect raw input with `chars().any(|c| c.is_control())` *before* trimming, rejecting all control characters. |
| **`THREAT-CAPDOC-04`** | **Result Set Flooding** | Denial-of-service via massive result vectors. | Enforce `MAX_DOC_SEARCH_RESULTS = 50`, truncating ranked search results before serialization. |
| **`THREAT-CAPDOC-05`** | **Documentation Store Tampering** | Adversarial modification or injection of fake capability documentation. | Index immutability: canonical topics are hardcoded and loaded into memory at initialization; no runtime mutation APIs exist. |

## Verification
- Validated via unit tests in `tests/test_capability_doc.rs` and live smoke test in `test_capability_doc_smoke.py`.
