# T-01684: Kernel Module Management Documentation Implementation

## Sub-Epic
Kernel Module Management / Documentation (T-01684)

## Objective
Implement the complete in-memory `KernelModuleDocIndex`, search scoring engine, Markdown formatter, and canonical documentation topics in `code/aiosh-rust/aiosh-core/src/kernel_module_doc.rs`.

## Implementation Details

1. **`DocCategory` Enum**:
   - Categorizes topics into `Directive`, `Lifecycle`, `Security`, `Observability`, and `Baseline`.

2. **`DocSection`, `DocTopic`, and `DocSearchResult`**:
   - Structured representations enabling precise navigation, tagging, citations, examples, and scored matching.

3. **Canonical Built-In Topics (7 Total)**:
   - `modprobe-directives`: Directives syntax (`blacklist`, `alias`, `options`, `install`, `remove`, `softdep`).
   - `cis-benchmark-hardening`: CIS distribution benchmark filesystem and protocol disabling baselines.
   - `lifecycle-workflows`: Module runtime states, dependencies, refcounts, and unloading constraints.
   - `observability-and-procfs`: Introspecting `/proc/modules`, `/sys/module/*`, memory footprints, and KASLR address protection.
   - `security-policy-and-pep`: SP-KM1..SP-KM6 rules, PEP capability authorization, prohibited and protected module rules.
   - `container-isolation`: Namespace isolation, overlayfs configurations (`metacopy=on`), and network bridging.
   - `wireless-pentest`: Wireless driver parameters (`ath9k_htc nohwcrypt=1`, `rtl8812au`), frame injection, and monitor mode.

4. **Search Scoring Engine (KD3)**:
   - Exact topic ID match: +100.
   - Exact/partial tag match: +50 / +20.
   - Title match: +25.
   - Summary match: +15.
   - Section content match: +10.
   - Deterministic tie-breaking by topic ID.

5. **Markdown Formatter (KD4)**:
   - `format_topic_markdown` outputs formatted Markdown with metadata headers, code blocks, reference lists, and tags.

## Verification
- Successfully compiled via `cargo check -p aiosh-core`.
