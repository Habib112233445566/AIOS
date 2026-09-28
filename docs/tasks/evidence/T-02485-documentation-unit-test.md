# Task T-02485 Evidence: Sandbox Documentation Unit Testing

## Goal
Implement focused automated unit tests for the Sandbox Documentation Subsystem (`SandboxDocIndex`).

## Test Vectors Evaluated
1. **Canonical Topic Coverage**: Verifies at least 6 canonical topics exist (`overview`, `profiles`, `isolation`, `policy`, `observability`, `reference`).
2. **Topic Retrieval & Normalization**: Asserts lookup by exact slug, case-insensitive uppercase slug (`PROFILES`), whitespace trimming, and non-existent slug yielding `None`.
3. **Lexical Search Scorer**: Evaluates queries matching isolation, policy, non-matching terms, and whitespace-only queries.
4. **Structural Completeness**: Asserts non-empty titles, summaries, sections, and valid category string tags across all topics.
5. **Serialization Parity**: Verifies serde serialization and deserialization preservation.

## Execution Output
```
running 5 tests
test test_sandbox_doc_categories_and_completeness ... ok
test test_sandbox_doc_get_topic ... ok
test test_sandbox_doc_list_topics ... ok
test test_sandbox_doc_search ... ok
test test_sandbox_doc_serialization ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
