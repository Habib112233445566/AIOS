# Documentation Research: Hardware Detection Subsystem (T-01781)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Documentation Subsystem (`aiosh-core::hardware_doc`)
- **Task**: `T-01781`
- **Scope**: Researching offline documentation indexing, topic modeling, categorized search, snippet extraction, and terminal rendering for the AIOS hardware detection stack.
- **Status**: **PASS (Research Complete)**

---

## 2. Prior Art & Authoritative Sources
1. **Linux Kernel Documentation (`Documentation/admin-guide/sysfs-rules.rst`, `Documentation/ABI/testing/sysfs-bus-pci`)**:
   - Outlines proper rules for reading sysfs attributes, symlink traversals, driver binding conventions, and udev events.
2. **CIS Linux Benchmarks (Hardware & Peripheral Devices)**:
   - Recommends disabling unneeded USB storage, wireless, and firewire modules, and monitoring hardware additions via udev.
3. **AIOS Reference Subsystem (`KernelModuleDocIndex`, `KD1..KD6`)**:
   - `code/aiosh-rust/aiosh-core/src/kernel_module_doc.rs`: Demonstrates an in-memory, self-contained documentation index with categorized topics, ranked search, tag matching, and markdown formatting.

---

## 3. Facts vs Assumptions

### Facts
- AIOS operates in air-gapped, isolated, and containerized environments where external web access or internet documentation is unavailable.
- In-memory documentation indexes (`DocIndex`) provide fast, offline, and zero-dependency guidance to operators, AI agents, and CLI users.
- `kernel_module_doc.rs` establishes standard constraints: `MAX_DOC_QUERY_LEN = 256`, `MAX_DOC_SEARCH_RESULTS = 50`, `MAX_TOPIC_ID_LEN = 64`.

### Assumptions & Constraints
- Hardware documentation topics must cover:
  1. `hw-sysfs-topology`: Linux `/sys` hierarchy for PCI, USB, block, net, CPU, DMI.
  2. `hw-security-policy`: Declarative gatekeeping, allowlists, denylists, and attribute redaction (`HSEC1..HSEC5`).
  3. `hw-observability-telemetry`: Telemetry metrics, driver binding rates, and compliance reporting (`HO1..HO6`).
  4. `hw-config-options`: Environment variables and configuration options (`HCFG1..HCFG5`).
  5. `hw-mcp-tools`: MCP tool interface (`aios.hardware.*`).
  6. `hw-troubleshooting`: Diagnostic procedures for missing devices, permission errors, and mock testing.
- The index must be $\le 500$ KB in memory.

---

## 4. Key Design Decisions
1. **Data Structures**: Define `HardwareDocCategory`, `HardwareDocTopic`, `HardwareDocSection`, `HardwareDocSearchResult`, and `HardwareDocIndex`.
2. **Search Scoring**:
   - Exact ID match: 100 points.
   - Title match: 50 points.
   - Tag match: 30 points.
   - Summary/Content match: 10 points.
3. **Markdown Rendering**: Provide `format_topic_markdown(&DocTopic) -> String` for rich CLI and MCP text display.
