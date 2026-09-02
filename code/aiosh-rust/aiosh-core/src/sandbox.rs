//! Landlock + seccomp-bpf sandbox for `aiosh run` (Sprint 2).
//!
//! Port of `sandbox.py`. Applied in the child process BEFORE execve:
//!   1. prctl(PR_SET_NO_NEW_PRIVS) — required before seccomp filter.
//!   2. seccomp(SECCOMP_SET_MODE_FILTER) — default-allow BPF that KILLs
//!      a small blacklist of dangerous syscalls (ptrace, mount, reboot,
//!      init_module, setuid, chroot, ...). If unsupported, warn+continue.
//!   3. landlock_create_ruleset + add_rule + restrict_self — restricts
//!      file access to a configurable allow list. If unsupported
//!      (kernel < 5.13), warn+continue.
//!
//! x86_64 syscall numbers are used (the project's CI host).

use std::io::Write;

pub const DEFAULT_DENYLIST: &[&str] = &[
    "ptrace", "mount", "umount2", "reboot", "kexec_load", "kexec_file_load",
    "init_module", "finit_module", "delete_module", "setuid", "setgid",
    "setreuid", "setregid", "setresuid", "setresgid", "chroot", "pivot_root",
];

// x86_64 syscall numbers (Linux).
#[cfg(target_os = "linux")]
const SYS_PRCTL: libc::c_long = 157;
#[cfg(target_os = "linux")]
const SYS_SECCOMP: libc::c_long = 317;
#[cfg(target_os = "linux")]
const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
#[cfg(target_os = "linux")]
const SYS_LANDLOCK_ADD_RULE: libc::c_long = 445;
#[cfg(target_os = "linux")]
const SYS_LANDLOCK_RESTRICT_SELF: libc::c_long = 446;

#[cfg(target_os = "linux")]
const PR_SET_NO_NEW_PRIVS: libc::c_int = 38;
#[cfg(target_os = "linux")]
const SECCOMP_SET_MODE_FILTER: libc::c_uint = 1;
#[cfg(target_os = "linux")]
const LANDLOCK_RULE_PATH_BENEATH: libc::c_uint = 1;
#[cfg(target_os = "linux")]
const AT_FDCWD: libc::c_int = -100;
#[cfg(target_os = "linux")]
const LANDLOCK_CREATE_RULESET_VERSION: libc::c_uint = 1;

// Landlock FS access bits (linux/landlock.h).
const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;

// seccomp BPF constants.
const BPF_LD: u16 = 0x00;
const BPF_JMP: u16 = 0x05;
const BPF_RET: u16 = 0x06;
const BPF_W: u16 = 0x00;
const BPF_ABS: u16 = 0x20;
const BPF_JEQ: u16 = 0x10;
const BPF_K: u16 = 0x00;
const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;
const SECCOMP_RET_KILL: u32 = 0x00000000;
const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
const SECCOMP_DATA_NR_OFFSET: u32 = 0;
const AUDIT_ARCH_X86_64: u32 = 0xC000003E;

#[derive(Debug, Clone)]
pub struct PathRule {
    pub path: String,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub paths_ro: Vec<String>,
    pub paths_rw: Vec<String>,
    pub paths_execute: Vec<String>,
    pub no_new_privs: bool,
    pub seccomp_denylist: Vec<String>,
    pub inherit_defaults: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            paths_ro: vec![],
            paths_rw: vec![],
            paths_execute: vec![],
            no_new_privs: true,
            seccomp_denylist: vec![],
            inherit_defaults: true,
        }
    }
}

impl SandboxPolicy {
    pub fn from_json(raw: &str) -> serde_json::Result<Self> {
        let v: serde_json::Value = serde_json::from_str(raw)?;
        let arr = |k: &str| -> Vec<String> {
            v.get(k)
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default()
        };
        Ok(Self {
            paths_ro: arr("paths_ro"),
            paths_rw: arr("paths_rw"),
            paths_execute: arr("paths_execute"),
            no_new_privs: v.get("no_new_privs").and_then(|x| x.as_bool()).unwrap_or(true),
            seccomp_denylist: arr("seccomp_denylist"),
            inherit_defaults: v.get("inherit_defaults").and_then(|x| x.as_bool()).unwrap_or(true),
        })
    }

    pub fn default_landlock_rules(&self, argv0: &str) -> Vec<PathRule> {
        let mut rules = vec![
            PathRule { path: "/usr".into(), read: true, write: false, execute: true },
            PathRule { path: "/lib".into(), read: true, write: false, execute: true },
            PathRule { path: "/lib64".into(), read: true, write: false, execute: true },
            PathRule { path: "/etc/ld.so.cache".into(), read: true, write: false, execute: false },
            PathRule { path: "/etc/ld.so.conf".into(), read: true, write: false, execute: false },
            PathRule { path: "/etc/ld.so.conf.d".into(), read: true, write: false, execute: false },
            PathRule { path: "/dev".into(), read: true, write: true, execute: false },
            PathRule { path: "/proc/self".into(), read: true, write: false, execute: false },
            PathRule { path: "/tmp".into(), read: true, write: true, execute: false },
        ];
        // argv[0]'s directory must be executable.
        let bin = std::path::Path::new(argv0);
        let bin_dir = if bin.is_absolute() {
            bin.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "/usr/bin".into())
        } else {
            "/usr/bin".into()
        };
        rules.push(PathRule { path: bin_dir, read: true, write: false, execute: true });
        if let Ok(cwd) = std::env::current_dir() {
            rules.push(PathRule {
                path: cwd.to_string_lossy().to_string(),
                read: true,
                write: true,
                execute: false,
            });
        }
        rules
    }

    pub fn to_landlock_rules(&self, argv0: &str) -> Vec<PathRule> {
        let mut rules: Vec<PathRule> = vec![];
        if self.inherit_defaults {
            rules.extend(self.default_landlock_rules(argv0));
        }
        for p in &self.paths_ro {
            rules.push(PathRule { path: p.clone(), read: true, write: false, execute: false });
        }
        for p in &self.paths_rw {
            rules.push(PathRule { path: p.clone(), read: true, write: true, execute: false });
        }
        for p in &self.paths_execute {
            rules.push(PathRule { path: p.clone(), read: false, write: false, execute: true });
        }
        rules
    }
}

// ---------------------------------------------------------------------
// seccomp BPF
// ---------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
struct SockFilter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

#[repr(C)]
struct SockFprog {
    len: u16,
    filter: *const SockFilter,
}

/// Build a default-allow BPF that KILLs the given syscall numbers.
fn build_blacklist_bpf(denied: &[i64], arch: u32) -> Vec<SockFilter> {
    if denied.is_empty() {
        return vec![SockFilter { code: BPF_RET | BPF_K, jt: 0, jf: 0, k: SECCOMP_RET_ALLOW }];
    }
    let l = denied.len();
    let kill_index = 4 + l;
    let mut stmts = Vec::with_capacity(kill_index + 1);
    // 0: LD arch
    stmts.push(SockFilter { code: BPF_LD | BPF_W | BPF_ABS, jt: 0, jf: 0, k: SECCOMP_DATA_ARCH_OFFSET });
    // 1: JEQ arch, jt=0, jf=kill_index-2
    stmts.push(SockFilter { code: BPF_JMP | BPF_JEQ | BPF_K, jt: 0, jf: (kill_index - 2) as u8, k: arch });
    // 2: LD nr
    stmts.push(SockFilter { code: BPF_LD | BPF_W | BPF_ABS, jt: 0, jf: 0, k: SECCOMP_DATA_NR_OFFSET });
    // 3..3+l-1: JEQ nr_i
    for (i, nr) in denied.iter().enumerate() {
        let cur = 3 + i;
        let jt = (kill_index - cur - 1) as u8;
        stmts.push(SockFilter { code: BPF_JMP | BPF_JEQ | BPF_K, jt, jf: 0, k: *nr as u32 });
    }
    // allow
    stmts.push(SockFilter { code: BPF_RET | BPF_K, jt: 0, jf: 0, k: SECCOMP_RET_ALLOW });
    // kill
    stmts.push(SockFilter { code: BPF_RET | BPF_K, jt: 0, jf: 0, k: SECCOMP_RET_KILL });
    stmts
}

#[cfg(target_os = "linux")]
fn apply_seccomp_blacklist(denied_syscalls: &[i64]) -> (bool, String) {
    if denied_syscalls.is_empty() {
        return (false, "no denylist provided; seccomp not installed".into());
    }
    let prog = build_blacklist_bpf(denied_syscalls, AUDIT_ARCH_X86_64);
    let prog_ptr = prog.as_ptr();
    let fp = SockFprog { len: prog.len() as u16, filter: prog_ptr };
    let rc = unsafe { libc::syscall(SYS_SECCOMP, SECCOMP_SET_MODE_FILTER, 0, &fp as *const SockFprog) };
    if rc < 0 {
        let err = std::io::Error::last_os_error();
        return (false, format!("seccomp(SET_MODE_FILTER) failed: {}", err));
    }
    (true, format!("seccomp-bpf installed: deny {} syscalls", denied_syscalls.len()))
}

#[cfg(not(target_os = "linux"))]
fn apply_seccomp_blacklist(denied_syscalls: &[i64]) -> (bool, String) {
    if denied_syscalls.is_empty() {
        (false, "no denylist provided; seccomp not installed".into())
    } else {
        (false, "seccomp-bpf unsupported on non-Linux".into())
    }
}

// ---------------------------------------------------------------------
// Landlock
// ---------------------------------------------------------------------

#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

/// `struct landlock_ruleset_attr { __u64 handled_access_fs; }`.
#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
}

/// Every FS right this sandbox may restrict (Landlock ABI v1 set).
/// `handled_access_fs` MUST cover all rights used by the rules, or
/// restrict_self silently leaves the unlisted rights unrestricted.
const LANDLOCK_HANDLED_FS: u64 = LANDLOCK_ACCESS_FS_EXECUTE
    | LANDLOCK_ACCESS_FS_WRITE_FILE
    | LANDLOCK_ACCESS_FS_READ_FILE
    | LANDLOCK_ACCESS_FS_READ_DIR
    | LANDLOCK_ACCESS_FS_REMOVE_DIR
    | LANDLOCK_ACCESS_FS_REMOVE_FILE
    | LANDLOCK_ACCESS_FS_MAKE_DIR
    | LANDLOCK_ACCESS_FS_MAKE_REG
    | LANDLOCK_ACCESS_FS_MAKE_SYM;

fn path_access_bits(rule: &PathRule) -> u64 {
    let mut bits = 0u64;
    if rule.read {
        bits |= LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR;
    }
    if rule.write {
        bits |= LANDLOCK_ACCESS_FS_WRITE_FILE
            | LANDLOCK_ACCESS_FS_REMOVE_FILE
            | LANDLOCK_ACCESS_FS_REMOVE_DIR
            | LANDLOCK_ACCESS_FS_MAKE_REG
            | LANDLOCK_ACCESS_FS_MAKE_DIR
            | LANDLOCK_ACCESS_FS_MAKE_SYM;
    }
    if rule.execute {
        bits |= LANDLOCK_ACCESS_FS_EXECUTE;
    }
    bits
}

#[cfg(target_os = "linux")]
fn apply_landlock(rules: &[PathRule]) -> (bool, String) {
    // 1. Probe the ABI version (flag LANDLOCK_CREATE_RULESET_VERSION).
    //    With this flag the kernel returns the highest supported ABI
    //    version — NOT a file descriptor — so this call is purely
    //    informational and must never be closed or used as an fd.
    let abi = unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            std::ptr::null::<u8>(),
            0 as usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    };
    if abi < 0 {
        let err = std::io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENOSYS) | Some(libc::EOPNOTSUPP) => {
                (false, "landlock not supported by kernel (pre-5.13?)".into())
            }
            _ => (
                false,
                format!("landlock_create_ruleset(version probe) failed: {}", err),
            ),
        };
    }
    // 2. Create the REAL ruleset: no flags, with handled_access_fs set.
    let attr = LandlockRulesetAttr { handled_access_fs: LANDLOCK_HANDLED_FS };
    let ruleset_fd = unsafe {
        libc::syscall(
            SYS_LANDLOCK_CREATE_RULESET,
            &attr as *const LandlockRulesetAttr,
            std::mem::size_of::<LandlockRulesetAttr>(),
            0 as libc::c_uint,
        )
    };
    if ruleset_fd < 0 {
        let err = std::io::Error::last_os_error();
        return match err.raw_os_error() {
            Some(libc::ENOSYS) | Some(libc::EOPNOTSUPP) => {
                (false, "landlock not supported by kernel (pre-5.13?)".into())
            }
            _ => (false, format!("landlock_create_ruleset failed: {}", err)),
        };
    }
    let mut added = 0u64;
    let mut ok = true;
    let mut detail = String::new();
    for rule in rules {
        let bits = path_access_bits(rule);
        if bits == 0 {
            continue;
        }
        let c_path = std::ffi::CString::new(rule.path.as_str()).unwrap_or_default();
        let attr = LandlockPathBeneathAttr { allowed_access: bits, parent_fd: AT_FDCWD };
        let rc = unsafe {
            libc::syscall(
                SYS_LANDLOCK_ADD_RULE,
                ruleset_fd,
                LANDLOCK_RULE_PATH_BENEATH,
                &attr as *const LandlockPathBeneathAttr,
                std::mem::size_of::<LandlockPathBeneathAttr>(),
            )
        };
        if rc < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::ENOENT) {
                continue; // path doesn't exist — non-fatal
            }
            ok = false;
            detail = format!("landlock_add_rule({:?}) failed: {}", rule.path, err);
            break;
        }
        added += 1;
        let _ = c_path;
    }
    if ok {
        let rc = unsafe { libc::syscall(SYS_LANDLOCK_RESTRICT_SELF, ruleset_fd, 0) };
        if rc < 0 {
            ok = false;
            detail = format!("landlock_restrict_self failed: {}", std::io::Error::last_os_error());
        }
    }
    unsafe { libc::close(ruleset_fd as i32) };
    if ok {
        (true, format!("landlock restricted: {} path rule(s) enforced", added))
    } else {
        (false, detail)
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_landlock(rules: &[PathRule]) -> (bool, String) {
    if rules.is_empty() {
        (false, "no path rules provided".into())
    } else {
        (false, "landlock unsupported on non-Linux".into())
    }
}

// ---------------------------------------------------------------------
// prctl
// ---------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn apply_no_new_privs() -> (bool, String) {
    let rc = unsafe { libc::syscall(SYS_PRCTL, PR_SET_NO_NEW_PRIVS as libc::c_long, 1, 0, 0, 0) };
    if rc < 0 {
        return (false, format!("prctl(PR_SET_NO_NEW_PRIVS) failed: {}", std::io::Error::last_os_error()));
    }
    (true, "no-new-privs set".into())
}

#[cfg(not(target_os = "linux"))]
fn apply_no_new_privs() -> (bool, String) {
    (false, "no-new-privs unsupported on non-Linux".into())
}

// ---------------------------------------------------------------------
// Sandbox application
// ---------------------------------------------------------------------

/// Apply the sandbox stack to the CURRENT process (call before execve).
pub fn apply_in_child(argv0: &str, policy: &SandboxPolicy) -> Vec<(String, String)> {
    let mut log: Vec<(String, String)> = vec![];
    if policy.no_new_privs {
        let (ok, detail) = apply_no_new_privs();
        log.push(("no_new_privs".into(), if ok { "ok".into() } else { format!("FAIL: {}", detail) }));
    }
    let denylist = resolve_denylist(&policy.seccomp_denylist);
    if !denylist.is_empty() {
        let (ok, detail) = apply_seccomp_blacklist(&denylist);
        log.push(("seccomp".into(), if ok { "ok".into() } else { format!("FAIL: {}", detail) }));
    }
    let rules = policy.to_landlock_rules(argv0);
    if !rules.is_empty() {
        let (ok, detail) = apply_landlock(&rules);
        log.push(("landlock".into(), if ok { "ok".into() } else { format!("FAIL: {}", detail) }));
    }
    log
}

/// Resolve syscall names to x86_64 numbers.
fn resolve_denylist(names: &[String]) -> Vec<i64> {
    let table: &[(&str, i64)] = &[
        ("ptrace", 101), ("mount", 165), ("umount2", 166), ("reboot", 169),
        ("kexec_load", 246), ("kexec_file_load", 320), ("init_module", 175),
        ("finit_module", 313), ("delete_module", 176), ("setuid", 105),
        ("setgid", 106), ("setreuid", 113), ("setregid", 114), ("setresuid", 117),
        ("setresgid", 119), ("chroot", 161), ("pivot_root", 155),
    ];
    let list: Vec<String> = if names.is_empty() {
        DEFAULT_DENYLIST.iter().map(|s| s.to_string()).collect()
    } else {
        names.to_vec()
    };
    list.iter()
        .filter_map(|n| table.iter().find(|(name, _)| name == n).map(|(_, nr)| *nr))
        .collect()
}

/// Emit the one-line `sandbox_applied` JSON to stderr (parent parses it).
pub fn emit_sandbox_applied(log: &[(String, String)]) {
    let components: Vec<Vec<String>> = log.iter().map(|(k, v)| vec![k.clone(), v.clone()]).collect();
    let obj = serde_json::json!({"event": "sandbox_applied", "components": components});
    let mut err = std::io::stderr();
    let _ = writeln!(err, "{}", obj);
}

/// Parse the sandbox_applied line from stderr (parent side).
pub fn parse_sandbox_applied(stderr: &str) -> Option<serde_json::Value> {
    for line in stderr.lines() {
        let t = line.trim();
        if !t.starts_with('{') {
            continue;
        }
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(t) {
            if obj.get("event").and_then(|v| v.as_str()) == Some("sandbox_applied") {
                return Some(obj);
            }
        }
    }
    None
}

/// Resolve argv[0] to an absolute path.
pub fn execve_path(argv0: &str) -> String {
    let p = std::path::Path::new(argv0);
    if p.is_absolute() && p.exists() {
        return argv0.to_string();
    }
    let path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into());
    for d in path.split(':') {
        let cand = std::path::Path::new(d).join(argv0);
        if cand.exists() {
            return cand.to_string_lossy().to_string();
        }
    }
    argv0.to_string()
}

#[cfg(target_os = "linux")]
/// Fork; in the child apply the sandbox then execve; parent reaps.
/// Returns the child's exit code.
pub fn sandbox_exec(argv: &[String], policy: &SandboxPolicy) -> i32 {
    if argv.is_empty() {
        eprintln!("sandbox_exec: empty argv");
        return 2;
    }
    let abs_argv0 = execve_path(&argv[0]);
    let argv_owned: Vec<std::ffi::CString> = std::iter::once(abs_argv0.clone())
        .chain(argv[1..].iter().cloned())
        .map(|s| std::ffi::CString::new(s).unwrap_or_default())
        .collect();
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        // Child.
        let log = apply_in_child(&abs_argv0, policy);
        emit_sandbox_applied(&log);
        let mut c_ptrs: Vec<*const libc::c_char> = argv_owned.iter().map(|c| c.as_ptr()).collect();
        c_ptrs.push(std::ptr::null());
        unsafe {
            libc::execv(std::ffi::CString::new(abs_argv0).unwrap_or_default().as_ptr(), c_ptrs.as_ptr());
        }
        // execv only returns on failure.
        eprintln!("sandbox: execv failed: {}", std::io::Error::last_os_error());
        unsafe { libc::_exit(126) };
    }
    // Parent: reap.
    let mut status: i32 = 0;
    unsafe { libc::waitpid(pid, &mut status, 0) };
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status)
    } else if libc::WIFSIGNALED(status) {
        let sig = libc::WTERMSIG(status);
        eprintln!("sandbox: child killed by signal {} (seccomp violation likely)", sig);
        128 + sig
    } else {
        1
    }
}

#[cfg(not(target_os = "linux"))]
pub fn sandbox_exec(argv: &[String], policy: &SandboxPolicy) -> i32 {
    if argv.is_empty() {
        eprintln!("sandbox_exec: empty argv");
        return 2;
    }
    let abs_argv0 = execve_path(&argv[0]);
    let log = apply_in_child(&abs_argv0, policy);
    emit_sandbox_applied(&log);
    match std::process::Command::new(&argv[0]).args(&argv[1..]).status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(e) => {
            eprintln!("sandbox: exec failed: {}", e);
            126
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpf_denylist_builds() {
        let prog = build_blacklist_bpf(&[101, 165], AUDIT_ARCH_X86_64);
        assert_eq!(prog.len(), 7); // 3 setup + 2 checks + allow + kill
        // Last instruction kills.
        assert_eq!(prog.last().unwrap().k, SECCOMP_RET_KILL);
    }

    #[test]
    fn empty_denylist_builds_allow_only() {
        let prog = build_blacklist_bpf(&[], AUDIT_ARCH_X86_64);
        assert_eq!(prog.len(), 1);
        assert_eq!(prog[0].k, SECCOMP_RET_ALLOW);
    }

    #[test]
    fn policy_from_json() {
        let p = SandboxPolicy::from_json(
            r#"{"paths_ro":["/etc"],"paths_rw":["/tmp"],"seccomp_denylist":["mount"]}"#,
        )
        .unwrap();
        assert_eq!(p.paths_ro, vec!["/etc".to_string()]);
        assert_eq!(p.paths_rw, vec!["/tmp".to_string()]);
        assert!(p.inherit_defaults);
    }

    #[test]
    fn parse_sandbox_applied_finds_line() {
        let stderr = "some noise\n{\"event\":\"sandbox_applied\",\"components\":[[\"seccomp\",\"ok\"]]}\n";
        let v = parse_sandbox_applied(stderr);
        assert!(v.is_some());
    }
}
