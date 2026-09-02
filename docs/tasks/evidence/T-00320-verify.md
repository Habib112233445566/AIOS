cargo.exe : warning: unused import: `std::fs::File`
At line:1 char:11
+ $output = & cargo test --manifest-path code/aiosh-rust/Cargo.toml 2>& ...
+           ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: (warning: unused import: `std::fs::File`:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError
 
   --> aiosh-core\src\release.rs:137:5
    |
137 | use std::fs::File;
    |     ^^^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
   --> aiosh-core\src\ledger.rs:146:9
    |
146 |     let mut opts = OpenOptions::new();
    |         ----^^^^
    |         |
    |         help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
  --> aiosh-core\src\release_config.rs:76:9
   |
76 |     let mut f = std::fs::File::open(p)
   |         ----^
   |         |
   |         help: remove this `mut`

warning: variable does not need to be mutable
  --> aiosh-core\src\toolchain_config.rs:36:13
   |
36 |         let mut f = std::fs::File::open(p)
   |             ----^
   |             |
   |             help: remove this `mut`

warning: constant `LOCK_POLL_MS` is never used
   --> aiosh-core\src\ledger.rs:617:7
    |
617 | const LOCK_POLL_MS: u64 = 50;
    |       ^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `0` is never read
   --> aiosh-core\src\ledger.rs:619:17
    |
619 | struct FileLock(File);
    |        -------- ^^^^
    |        |
    |        field in this struct
    |
    = help: consider removing this field

warning: constant `LANDLOCK_ACCESS_FS_EXECUTE` is never used
  --> aiosh-core\src\sandbox.rs:46:7
   |
46 | const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_WRITE_FILE` is never used
  --> aiosh-core\src\sandbox.rs:47:7
   |
47 | const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_READ_FILE` is never used
  --> aiosh-core\src\sandbox.rs:48:7
   |
48 | const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_READ_DIR` is never used
  --> aiosh-core\src\sandbox.rs:49:7
   |
49 | const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_REMOVE_DIR` is never used
  --> aiosh-core\src\sandbox.rs:50:7
   |
50 | const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_REMOVE_FILE` is never used
  --> aiosh-core\src\sandbox.rs:51:7
   |
51 | const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_MAKE_DIR` is never used
  --> aiosh-core\src\sandbox.rs:52:7
   |
52 | const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_MAKE_REG` is never used
  --> aiosh-core\src\sandbox.rs:53:7
   |
53 | const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_ACCESS_FS_MAKE_SYM` is never used
  --> aiosh-core\src\sandbox.rs:54:7
   |
54 | const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `BPF_LD` is never used
  --> aiosh-core\src\sandbox.rs:57:7
   |
57 | const BPF_LD: u16 = 0x00;
   |       ^^^^^^

warning: constant `BPF_JMP` is never used
  --> aiosh-core\src\sandbox.rs:58:7
   |
58 | const BPF_JMP: u16 = 0x05;
   |       ^^^^^^^

warning: constant `BPF_RET` is never used
  --> aiosh-core\src\sandbox.rs:59:7
   |
59 | const BPF_RET: u16 = 0x06;
   |       ^^^^^^^

warning: constant `BPF_W` is never used
  --> aiosh-core\src\sandbox.rs:60:7
   |
60 | const BPF_W: u16 = 0x00;
   |       ^^^^^

warning: constant `BPF_ABS` is never used
  --> aiosh-core\src\sandbox.rs:61:7
   |
61 | const BPF_ABS: u16 = 0x20;
   |       ^^^^^^^

warning: constant `BPF_JEQ` is never used
  --> aiosh-core\src\sandbox.rs:62:7
   |
62 | const BPF_JEQ: u16 = 0x10;
   |       ^^^^^^^

warning: constant `BPF_K` is never used
  --> aiosh-core\src\sandbox.rs:63:7
   |
63 | const BPF_K: u16 = 0x00;
   |       ^^^^^

warning: constant `SECCOMP_RET_ALLOW` is never used
  --> aiosh-core\src\sandbox.rs:64:7
   |
64 | const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;
   |       ^^^^^^^^^^^^^^^^^

warning: constant `SECCOMP_RET_KILL` is never used
  --> aiosh-core\src\sandbox.rs:65:7
   |
65 | const SECCOMP_RET_KILL: u32 = 0x00000000;
   |       ^^^^^^^^^^^^^^^^

warning: constant `SECCOMP_DATA_ARCH_OFFSET` is never used
  --> aiosh-core\src\sandbox.rs:66:7
   |
66 | const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `SECCOMP_DATA_NR_OFFSET` is never used
  --> aiosh-core\src\sandbox.rs:67:7
   |
67 | const SECCOMP_DATA_NR_OFFSET: u32 = 0;
   |       ^^^^^^^^^^^^^^^^^^^^^^

warning: constant `AUDIT_ARCH_X86_64` is never used
  --> aiosh-core\src\sandbox.rs:68:7
   |
68 | const AUDIT_ARCH_X86_64: u32 = 0xC000003E;
   |       ^^^^^^^^^^^^^^^^^

warning: struct `SockFilter` is never constructed
   --> aiosh-core\src\sandbox.rs:179:8
    |
179 | struct SockFilter {
    |        ^^^^^^^^^^

warning: struct `SockFprog` is never constructed
   --> aiosh-core\src\sandbox.rs:187:8
    |
187 | struct SockFprog {
    |        ^^^^^^^^^

warning: function `build_blacklist_bpf` is never used
   --> aiosh-core\src\sandbox.rs:193:4
    |
193 | fn build_blacklist_bpf(denied: &[i64], arch: u32) -> Vec<SockFilter> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: struct `LandlockPathBeneathAttr` is never constructed
   --> aiosh-core\src\sandbox.rs:249:8
    |
249 | struct LandlockPathBeneathAttr {
    |        ^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `LandlockRulesetAttr` is never constructed
   --> aiosh-core\src\sandbox.rs:256:8
    |
256 | struct LandlockRulesetAttr {
    |        ^^^^^^^^^^^^^^^^^^^

warning: constant `LANDLOCK_HANDLED_FS` is never used
   --> aiosh-core\src\sandbox.rs:263:7
    |
263 | const LANDLOCK_HANDLED_FS: u64 = LANDLOCK_ACCESS_FS_EXECUTE
    |       ^^^^^^^^^^^^^^^^^^^

warning: function `path_access_bits` is never used
   --> aiosh-core\src\sandbox.rs:273:4
    |
273 | fn path_access_bits(rule: &PathRule) -> u64 {
    |    ^^^^^^^^^^^^^^^^

warning: `aiosh-core` (lib) generated 34 warnings (run `cargo fix --lib -p aiosh-core` to apply 4 suggestions)
warning: value assigned to `argv` is never read
  --> aiosh-sandbox\src\main.rs:30:33
   |
30 |     let mut argv: Vec<String> = Vec::new();
   |                                 ^^^^^^^^^^ this value is reassigned later and never used
...
46 |             argv = rest[1..].to_vec();
   |             ---- `argv` is overwritten here before the previous value is read
   |
   = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: `aiosh-sandbox` (bin "aiosh-sandbox" test) generated 1 warning
warning: unused import: `std::path::Path`
   --> aiosh-core\src\release.rs:396:9
    |
396 |     use std::path::Path;
    |         ^^^^^^^^^^^^^^^
    |
    = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: `aiosh-core` (lib test) generated 20 warnings (19 duplicates) (run `cargo fix --lib -p aiosh-core --tests` to 
apply 1 suggestion)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.40s
     Running unittests src\main.rs (code\aiosh-rust\target\debug\deps\aiosh-8814d271a6af31ad.exe)

running 13 tests
test task_cli_tests::extra_operand_rejected_for_read_only_actions ... ok
test task_cli_tests::double_dash_allows_dash_leading_values ... ok
test task_cli_tests::id_must_be_decimal_gte_one ... ok
test task_cli_tests::evidence_item_cap_enforced_by_validate ... ok
test task_cli_tests::parses_done_with_note_and_repeatable_evidence ... ok
test task_cli_tests::parses_status_without_operand ... ok
test task_cli_tests::rejects_dash_leading_option_value ... ok
test task_cli_tests::rejects_empty_note ... ok
test task_cli_tests::rejects_missing_note ... ok
test task_cli_tests::rejects_missing_value_at_end ... ok
test task_cli_tests::rejects_unknown_option_token ... ok
test task_cli_tests::rejects_oversized_text_at_validate ... ok
test task_cli_tests::usage_text_lists_contract ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (code\aiosh-rust\target\debug\deps\aiosh_core-04423105fe4b6d57.exe)

running 86 tests
test agent::tests::normalize_plan_rejects_unknown_tool ... ok
test agent::tests::normalize_plan_accepts_valid_json ... ok
test agent::tests::stub_plan_audit_tail ... ok
test agent::tests::stub_plan_scan ... ok
test agent::tests::stub_stops_after_refusal ... ok
test audit::tests::hash_matches_manual_computation ... ok
test canonical::tests::canonical_escapes_strings ... ok
test canonical::tests::canonical_nested_and_arrays ... ok
test canonical::tests::canonical_sorts_keys_and_minimizes ... ok
test audit::tests::chain_extends_and_verifies ... ok
test audit::tests::verify_detects_tampering ... ok
test audit::tests::tail_is_ascending ... ok
test canonical::tests::sha256_hex_is_64_lowercase ... ok
test canonical::tests::utcnow_iso_has_z_and_microseconds ... ok
test classifier::tests::gui_is_caution ... ok
test classifier::tests::persist_flags_c3 ... ok
test classifier::tests::r01_cautions_pentest ... ok
test classifier::tests::r05a_cautions_dangerous_bin ... ok
test classifier::tests::python_fixture_matrix_sc1_to_sc10 ... ok
test classifier::tests::r09_refuses_external_aggregator ... ok
test classifier::tests::r11_refuses_prompt_injection_arg ... ok
test classifier::tests::read_only_tools_are_ok ... ok
test dispatch::tests::pep_refuses_without_grant ... ok
test dispatch::tests::gate_passes_with_valid_grant_and_commit_extends_chain ... ok
test ledger::tests::ancestor_walk_finds_repo_tasks_dir ... ok
test dispatch::tests::classifier_refusal_beats_grant ... ok
test ledger::tests::events_size_cap_rejects_oversized_log ... ok
test ledger::tests::invariants_check_passes_and_detects_gaps ... ok
test ledger::tests::complete_advances_pointer_exactly_one ... ok
test ledger::tests::no_skip_rejects_out_of_order ... ok
test ledger::tests::block_unblock_skip_flow ... ok
test ledger::tests::stale_tmp_files_are_cleaned_on_save ... ok
test ledger::tests::rebuild_recomputes_from_events ... ok
test ledger::tests::validate_state_clean_repo_is_consistent ... ok
test ledger::tests::validate_state_detects_drift_without_mutating ... ok
test ledger::tests::validate_state_detects_seq_gap ... ok
test ledger_config::tests::from_source_precedence_and_loud_errors ... ok
test ledger::tests::rebuild_replays_skip_and_unblock_pointers ... ok
test ledger_config::tests::scaffold_defaults_compose ... ok
test pentest::tests::subprocess_missing_binary ... ok
test pep::tests::check_enforces_network_scope ... ok
test pep::tests::check_enforces_paths ... ok
test pep::tests::check_enforces_tool_scope ... ok
test pep::tests::check_requires_grant_for_pentest ... ok
test pep::tests::create_get_revoke ... ok
test pep::tests::path_deny_wins ... ok
test pep::tests::tool_glob_matches ... ok
test release::observability_tests::test_run_external_packager_captures_error ... ok
test ledger::tests::rebuild_clamps_pointer_at_end_of_ledger ... ok
test release::recovery_tests::test_restore_backup_refuses_non_empty_dir ... ok
test pentest::tests::subprocess_runs_and_captures ... ok
test release::recovery_tests::test_validate_release_invalid_hash ... ok
test release::security_tests::test_check_release_policy_enforcement ... ok
test release::tests::test_generate_release_empty_components ... ok
test release::tests::test_create_backup_happy_path ... ok
test release::recovery_tests::test_restore_backup_requires_grant_if_checked ... ok
test release_config::tests::test_load_config_happy_path ... ok
test release_config::tests::test_load_config_rejects_absolute_paths ... ok
test release_config::tests::test_load_config_rejects_path_traversal ... ok
test release::tests::test_generate_release_happy_path ... ok
test retention::tests::bloom_filter_membership ... ok
test release_config::tests::test_load_config_size_bound ... ok
test retention::tests::dry_run_writes_nothing ... ok
test sandbox::tests::bpf_denylist_builds ... ok
test sandbox::tests::empty_denylist_builds_allow_only ... ok
test sandbox::tests::parse_sandbox_applied_finds_line ... ok
test sandbox::tests::policy_from_json ... ok
test retention::tests::rotate_refuses_broken_chain ... ok
test task_service::tests::execute_rebuild_replays_skip_pointer ... ok
test task_service::tests::grant_truth_table_matches_spec_d1 ... ok
test task_service::tests::parse_args_schema_bounds ... ok
test task_service::tests::execute_status_and_done_against_explicit_paths ... ok
test task_service::tests::parse_args_strict_types ... ok
test task_service::tests::parse_round_trip_and_unknowns ... ok
test retention::tests::rotate_archives_and_verifies_full ... ok
test task_service::tests::validate_conditional_requirements ... ok
test toolchain_config::tests::test_load_toolchain_config_empty_version ... ok
test toolchain_config::tests::test_load_toolchain_config_happy_path ... ok
test task_service::tests::resolver_helper_finds_and_refuses ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_file ... ok
test toolchain_config::tests::test_load_toolchain_config_missing_field ... ok
test toolchain_config::tests::test_load_toolchain_config_malformed_json ... ok
test toolchain_config::tests::test_to_json_with_sources ... ok
test types::tests::hash_proto_is_canonical_json_compatible ... ok
test types::tests::recompute_hash_matches_manual ... ok
test pentest::tests::subprocess_timeout ... ok

test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.45s

     Running unittests src\main.rs (code\aiosh-rust\target\debug\deps\aiosh_mcp-570b1a936abd2622.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (code\aiosh-rust\target\debug\deps\aiosh_sandbox-ee8798362c517cdf.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests aiosh_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

