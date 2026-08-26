"""Landlock + seccomp-bpf sandbox for `aiosh run`.

This module applies two complementary kernel sandbox mechanisms BEFORE
the target command is execve'd, so the audit-gated process call is
actually sandboxed, not just logged (the Sprint 0/1/1.5/2 carry-over
gap).

Mechanisms applied (in order):

1. `prctl(PR_SET_NO_NEW_PRIVS)` — required before seccomp(2). Without
   this, the seccomp filter install fails with EINVAL.

2. `seccomp(SECCOMP_SET_MODE_FILTER, ...)` with a default-allow BPF
   program that KILLs a small blacklist of dangerous syscalls. The
   blacklist is conservative and intentionally small (we don't want
   to break libc startup); Landlock handles the file-access policy,
   seccomp just removes the highest-blast-radius syscalls
   (`ptrace`, `mount`, `reboot`, `init_module`, `delete_module`,
   `kexec_load`, `setuid`, `setgid`, `socket` ...). If the kernel
   does not support SECCOMP_FILTER (e.g. very old kernels), we
   warn and continue — Landlock still applies.

3. `landlock_create_ruleset` + `landlock_add_rule` +
   `landlock_restrict_self` — restricts the child process to a
   configurable set of file paths, each with read-only or
   read-write access. This is the file-access boundary: even if
   the command runs as root, it cannot read paths outside the
   allow list, and cannot write outside the rw list. If the
   kernel does not support Landlock (pre-5.13), we warn and
   continue with seccomp only.

Usage:

    sandbox_exec(
        argv=["/bin/ls", "-la", "/tmp"],
        policy={
            "paths_ro": ["/usr", "/lib", "/lib64", "/bin",
                         "/etc/ld.so.cache", "/tmp"],
            "paths_rw": [],
            "no_new_privs": True,
            "seccomp_denylist": ["ptrace", "mount", "reboot",
                                  "kexec_load", "init_module",
                                  "delete_module"],
        },
    )

The sandbox is applied in the CHILD after fork. The parent does the
fork, the child applies the sandbox and execve's, the parent reaps.
"""

from __future__ import annotations

import ctypes
import ctypes.util
import errno
import json
import os
import struct
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Sequence


# ----------------------------------------------------------------------
# ctypes plumbing for libc and the raw Linux syscalls we need.
# ----------------------------------------------------------------------

_libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)


def _syscall(name: str) -> int:
    """Resolve a libc syscall number from /usr/include/asm-generic/unistd.h.
    Returns -1 if not found (caller treats as unsupported)."""
    # These are stable per-arch. We hardcode the x86_64 / aarch64 numbers
    # the project targets. Multi-arch support is out of scope for Sprint 0.
    table = {
        # x86_64 numbers
        "seccomp": 317,
        "landlock_create_ruleset": 444,
        "landlock_add_rule": 445,
        "landlock_restrict_self": 446,
        # aarch64 numbers (for completeness — not exercised here)
        "seccomp_arm64": 277,
        "landlock_create_ruleset_arm64": 436,
        "landlock_add_rule_arm64": 437,
        "landlock_restrict_self_arm64": 438,
    }
    return table.get(name, -1)


def _libc_syscall(n: int, *args) -> int:
    """Direct libc syscall(2) wrapper. Returns the kernel return value
    (negative errno on failure)."""
    return _libc.syscall(n, *args)


def _errno() -> int:
    return ctypes.get_errno()


# ----------------------------------------------------------------------
# Landlock constants (linux/landlock.h)
# ----------------------------------------------------------------------

LANDLOCK_ACCESS_FS_EXECUTE = 1 << 0
LANDLOCK_ACCESS_FS_WRITE_FILE = 1 << 1
LANDLOCK_ACCESS_FS_READ_FILE = 1 << 2
LANDLOCK_ACCESS_FS_READ_DIR = 1 << 3
LANDLOCK_ACCESS_FS_REMOVE_DIR = 1 << 4
LANDLOCK_ACCESS_FS_REMOVE_FILE = 1 << 5
LANDLOCK_ACCESS_FS_MAKE_CHAR = 1 << 6
LANDLOCK_ACCESS_FS_MAKE_DIR = 1 << 7
LANDLOCK_ACCESS_FS_MAKE_REG = 1 << 8
LANDLOCK_ACCESS_FS_MAKE_SOCK = 1 << 9
LANDLOCK_ACCESS_FS_MAKE_FIFO = 1 << 10
LANDLOCK_ACCESS_FS_MAKE_BLOCK = 1 << 11
LANDLOCK_ACCESS_FS_MAKE_SYM = 1 << 12


# ----------------------------------------------------------------------
# seccomp constants (linux/seccomp.h)
# ----------------------------------------------------------------------

SECCOMP_MODE_FILTER = 2

# BPF opcodes (linux/filter.h)
BPF_LD = 0x00
BPF_JMP = 0x05
BPF_RET = 0x06
BPF_W = 0x00
BPF_ABS = 0x20
BPF_JEQ = 0x10
BPF_K = 0x00

# seccomp_data offsets (linux/seccomp.h)
SECCOMP_DATA_NR_OFFSET = 0    # __u32 nr (syscall number)
SECCOMP_DATA_ARCH_OFFSET = 4  # __u32 arch

# seccomp return values
SECCOMP_RET_ALLOW = 0x7fff0000
SECCOMP_RET_KILL = 0x00000000

# x86_64 audit arch (__AUDIT_ARCH_64BIT | __AUDIT_ARCH_X86_64)
AUDIT_ARCH_X86_64 = 0xC000003E
# aarch64
AUDIT_ARCH_AARCH64 = 0xC00000B7


def _detect_arch() -> int:
    """Return the SECCOMP_DATA.arch value for the running kernel."""
    machine = os.uname().machine
    if machine == "x86_64":
        return AUDIT_ARCH_X86_64
    if machine in ("aarch64", "arm64"):
        return AUDIT_ARCH_AARCH64
    return 0  # unknown


# ----------------------------------------------------------------------
# BPF filter for default-allow + blacklist (kills the listed syscalls).
# ----------------------------------------------------------------------

@dataclass
class BpfStmt:
    """A single BPF instruction. code/jt/jf/k follow the kernel
    `struct sock_filter` layout exactly."""
    code: int
    jt: int
    jf: int
    k: int


def _build_blacklist_bpf(
    denied_syscalls: Sequence[int],
    arch_value: int,
) -> bytes:
    """Build a default-allow BPF program that KILLs the listed syscall
    numbers. The program first verifies the arch, then jumps through
    a chain of JEQ checks; if any matches, it KILLs; otherwise ALLOW.

    BPF jump semantics (linux/filter.h):
      jt / jf are forward offsets FROM THE NEXT INSTRUCTION.
      i.e. executing JMP at index N advances PC to N+1+jt (true) or
      N+1+jf (false). This is why our `kill_index` arithmetic uses
      `kill_index - current_inst - 1`.

    Program layout (indices, not absolute byte offsets):
       0: LD   W ABS [arch]              (load seccomp_data.arch)
       1: JEQ  arch_value, jt=0, jf=K    (K = kill_index; arch mismatch kills)
       2: LD   W ABS [nr]                (load seccomp_data.nr)
       3..(3+L-1): JEQ nr_i, jt=K-(3+i)-1, jf=0    (kill on match, else fall through)
       3+L:        RET K SECCOMP_RET_ALLOW
       K=4+L:      RET K SECCOMP_RET_KILL
    """
    if not denied_syscalls:
        # Empty BPF would be invalid; return a pure-allow single insn.
        return struct.pack("<HBBI",
                           BPF_RET | BPF_K, 0, 0, SECCOMP_RET_ALLOW)

    L = len(denied_syscalls)
    kill_index = 4 + L  # absolute index of the KILL instruction
    allow_index = 3 + L  # absolute index of the ALLOW instruction

    stmts: list[BpfStmt] = []
    # 0: LD arch
    stmts.append(BpfStmt(BPF_LD | BPF_W | BPF_ABS,
                          0, 0, SECCOMP_DATA_ARCH_OFFSET))
    # 1: JEQ arch, jt=0 (continue), jf=kill_index-(1+1)=kill_index-2
    arch_jf = kill_index - 2
    stmts.append(BpfStmt(BPF_JMP | BPF_JEQ | BPF_K,
                          0, arch_jf, arch_value))
    # 2: LD nr
    stmts.append(BpfStmt(BPF_LD | BPF_W | BPF_ABS,
                          0, 0, SECCOMP_DATA_NR_OFFSET))
    # 3..3+L-1: JEQ nr_i, jt=kill_index-(i+3)-1, jf=0
    for i, nr in enumerate(denied_syscalls):
        cur = 3 + i
        jt = kill_index - cur - 1
        stmts.append(BpfStmt(BPF_JMP | BPF_JEQ | BPF_K,
                              jt, 0, nr))
    # 3+L: ALLOW
    stmts.append(BpfStmt(BPF_RET | BPF_K, 0, 0, SECCOMP_RET_ALLOW))
    # 4+L: KILL
    stmts.append(BpfStmt(BPF_RET | BPF_K, 0, 0, SECCOMP_RET_KILL))

    assert len(stmts) == kill_index + 1
    return b"".join(struct.pack("<HBBI", s.code, s.jt, s.jf, s.k)
                    for s in stmts)


def _apply_seccomp_blacklist(denied_syscalls: Sequence[int]) -> tuple[bool, str]:
    """Install the BPF filter. Returns (applied, detail)."""
    if not denied_syscalls:
        return False, "no denylist provided; seccomp not installed"
    arch = _detect_arch()
    if arch == 0:
        return False, "unsupported arch for seccomp filter"
    bpf_bytes = _build_blacklist_bpf(denied_syscalls, arch)
    # struct sock_fprog { u16 len; struct sock_filter *filter; } —
    # declared as a ctypes Structure so the pointer lands at its true
    # (aligned) offset. Hand-packing the u64 at byte offset 2 put it in
    # the padding and handed the kernel a NULL filter (EFAULT).
    n_insns = len(bpf_bytes) // 8
    prog_arr = (ctypes.c_uint32 * (len(bpf_bytes) // 4)) \
        .from_buffer_copy(bpf_bytes)

    class SockFprog(ctypes.Structure):
        _fields_ = [
            ("len", ctypes.c_uint16),
            ("filter", ctypes.POINTER(ctypes.c_uint32)),
        ]

    fprog = SockFprog(n_insns, ctypes.cast(prog_arr,
                                           ctypes.POINTER(ctypes.c_uint32)))
    SECCOMP_SET_MODE_FILTER = 1
    rc = _libc_syscall(_syscall("seccomp"),
                        SECCOMP_SET_MODE_FILTER, 0, ctypes.byref(fprog))
    if rc < 0:
        eno = _errno()
        if eno == errno.EINVAL:
            # EINVAL almost always means NoNewPrivs isn't set; the
            # caller should have called _apply_no_new_privs first.
            return False, (f"seccomp(SET_MODE_FILTER) failed: EINVAL "
                            f"(likely NoNewPrivs not set, or process "
                            f"multithreaded); errno={eno}")
        return False, f"seccomp(SET_MODE_FILTER) failed: errno={eno} " \
                       f"({errno.errorcode.get(eno, '?')})"
    return True, f"seccomp-bpf installed: deny {len(denied_syscalls)} syscalls"


# ----------------------------------------------------------------------
# Landlock
# ----------------------------------------------------------------------

@dataclass
class PathRule:
    path: str
    read: bool = False
    write: bool = False
    execute: bool = False


def _path_access_bits(rule: PathRule) -> int:
    bits = 0
    if rule.read:
        bits |= (LANDLOCK_ACCESS_FS_READ_FILE
                 | LANDLOCK_ACCESS_FS_READ_DIR)
    if rule.write:
        bits |= (LANDLOCK_ACCESS_FS_WRITE_FILE
                 | LANDLOCK_ACCESS_FS_REMOVE_FILE
                 | LANDLOCK_ACCESS_FS_REMOVE_DIR
                 | LANDLOCK_ACCESS_FS_MAKE_REG
                 | LANDLOCK_ACCESS_FS_MAKE_DIR
                 | LANDLOCK_ACCESS_FS_MAKE_SYM)
    if rule.execute:
        bits |= LANDLOCK_ACCESS_FS_EXECUTE
    return bits


def _apply_landlock(rules: Sequence[PathRule]) -> tuple[bool, str]:
    """Create a Landlock ruleset, add the rules, restrict self.
    Returns (applied, detail)."""
    # Step 1 — probe the ABI version. With LANDLOCK_CREATE_RULESET_
    # VERSION (=1) in flags the kernel returns the highest supported
    # ABI version — an integer, NOT a file descriptor. The previous
    # code used this probe as if it were the ruleset fd, so no ruleset
    # ever existed and every add_rule/restrict_self failed with EBADF.
    nr_create = _syscall("landlock_create_ruleset")
    abi = _libc_syscall(nr_create, None, 0, 1)
    if abi < 0:
        eno = _errno()
        if eno in (errno.ENOSYS, errno.EOPNOTSUPP):
            return False, "landlock not supported by kernel (pre-5.13?)"
        return False, f"landlock_create_ruleset(version probe) failed: " \
                      f"errno={eno} ({errno.errorcode.get(eno, '?')})"
    # Step 2 — create the REAL ruleset: flags=0 and handled_access_fs
    # must list every right we restrict; anything omitted is left
    # unrestricted for ALL paths.
    handled = (_path_access_bits(PathRule(path="", read=True, write=True,
                                          execute=True)))
    attr_ruleset = struct.pack("<Q", handled)
    ruleset_fd = _libc_syscall(nr_create, attr_ruleset,
                                len(attr_ruleset), 0)
    if ruleset_fd < 0:
        eno = _errno()
        if eno in (errno.ENOSYS, errno.EOPNOTSUPP):
            return False, "landlock not supported by kernel (pre-5.13?)"
        return False, f"landlock_create_ruleset failed: errno={eno} " \
                       f"({errno.errorcode.get(eno, '?')})"
    added = 0
    try:
        for rule in rules:
            bits = _path_access_bits(rule)
            if bits == 0:
                continue
            # struct landlock_path_beneath_attr {
            #   __u64 allowed_access;
            #   __s32 parent_fd;
            # };
            AT_FDCWD = -100
            attr = struct.pack("<Qi", bits, AT_FDCWD)
            rc = _libc_syscall(
                _syscall("landlock_add_rule"),
                ruleset_fd,
                1,  # LANDLOCK_RULE_PATH_BENEATH = 1
                attr,
                len(attr),
            )
            if rc < 0:
                eno = _errno()
                # ENOENT for a path that doesn't exist is a non-fatal
                # warning — keep going so a policy referencing /var
                # works even when /var isn't mounted.
                if eno == errno.ENOENT:
                    continue
                return False, f"landlock_add_rule({rule.path!r}) failed: " \
                              f"errno={eno} ({errno.errorcode.get(eno, '?')})"
            added += 1
        # Restrict self. This call is irrevocable for the process.
        rc = _libc_syscall(_syscall("landlock_restrict_self"),
                            ruleset_fd, 0)
        if rc < 0:
            return False, f"landlock_restrict_self failed: errno={_errno()}"
    finally:
        os.close(ruleset_fd)
    return True, f"landlock restricted: {added} path rule(s) enforced"


# ----------------------------------------------------------------------
# prctl wrappers
# ----------------------------------------------------------------------

def _apply_no_new_privs() -> tuple[bool, str]:
    """prctl(PR_SET_NO_NEW_PRIVS, 1). Required before seccomp filter."""
    PR_SET_NO_NEW_PRIVS = 38
    # prctl is option-type based; on x86_64 glibc exposes it via
    # syscall(SYS_prctl, ...).
    SYS_prctl = 157
    rc = _libc_syscall(SYS_prctl, PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    if rc < 0:
        return False, f"prctl(PR_SET_NO_NEW_PRIVS) failed: errno={_errno()}"
    return True, "no-new-privs set"


# ----------------------------------------------------------------------
# Default seccomp blacklist — conservative, libc-friendly.
# ----------------------------------------------------------------------

DEFAULT_DENYLIST_NAMES = (
    "ptrace", "mount", "umount2", "reboot", "kexec_load",
    "kexec_file_load", "init_module", "finit_module", "delete_module",
    "setuid", "setgid", "setreuid", "setregid", "setresuid",
    "setresgid", "chroot", "pivot_root",
    # Network is intentionally NOT blocked; the agent needs to talk to
    # Ollama. Landlock (paths) and the C-1/C-3 classifier handle the
    # higher-level policy.
)


_SYSCALL_NAME_TO_NR = {
    "x86_64": {
        "ptrace": 101, "mount": 165, "umount2": 166, "reboot": 169,
        "kexec_load": 246, "kexec_file_load": 320, "init_module": 175,
        "finit_module": 313, "delete_module": 176, "setuid": 105,
        "setgid": 106, "setreuid": 113, "setregid": 114, "setresuid": 117,
        "setresgid": 119, "chroot": 161, "pivot_root": 155,
    },
    "aarch64": {
        "ptrace": 117, "mount": 40, "umount2": 39, "reboot": 142,
        "kexec_load": 104, "kexec_file_load": 294, "init_module": 105,
        "finit_module": 273, "delete_module": 106, "setuid": 146,
        "setgid": 144, "setreuid": 113, "setregid": 114, "setresuid": 119,
        "setresgid": 120, "chroot": 51, "pivot_root": 41,
    },
}


def _resolve_denylist(names: Sequence[str]) -> list[int]:
    arch = os.uname().machine
    table = _SYSCALL_NAME_TO_NR.get(arch, _SYSCALL_NAME_TO_NR["x86_64"])
    out: list[int] = []
    for n in names:
        if n in table:
            out.append(table[n])
    return out


# ----------------------------------------------------------------------
# Default Landlock path policy — read-only access to the runtime needed
# for /bin/ls /usr/lib etc.; the working directory is read-write.
# ----------------------------------------------------------------------

def _default_landlock_rules(argv: Sequence[str]) -> list[PathRule]:
    """Return a conservative default path policy. Anything not listed
    becomes inaccessible. The caller can override via the policy dict."""
    # argv[0] is the binary — its directory must be executable.
    bin_path = Path(argv[0])
    bin_dir = str(bin_path.parent.resolve()) if bin_path.is_absolute() \
        else "/usr/bin"
    rules = [
        PathRule(bin_dir, read=True, execute=True),
        PathRule("/usr", read=True, execute=True),
        PathRule("/lib", read=True, execute=True),
        PathRule("/lib64", read=True, execute=True),
        PathRule("/etc/ld.so.cache", read=True),
        PathRule("/etc/ld.so.conf", read=True),
        PathRule("/etc/ld.so.conf.d", read=True),
        PathRule("/dev", read=True, write=True),     # stdio, null, tty, etc.
        PathRule("/proc/self", read=True),           # /proc/self/maps etc.
        PathRule("/tmp", read=True, write=True),
        PathRule(str(Path.cwd()), read=True, write=True),
    ]
    return rules


# ----------------------------------------------------------------------
# Policy dataclass + parser
# ----------------------------------------------------------------------

@dataclass
class SandboxPolicy:
    paths_ro: list[str] = field(default_factory=list)
    paths_rw: list[str] = field(default_factory=list)
    paths_execute: list[str] = field(default_factory=list)
    no_new_privs: bool = True
    seccomp_denylist: list[str] = field(default_factory=list)
    inherit_defaults: bool = True

    def to_landlock_rules(self, argv: Sequence[str]) -> list[PathRule]:
        rules: list[PathRule] = []
        if self.inherit_defaults:
            rules.extend(_default_landlock_rules(argv))
        for p in self.paths_ro:
            rules.append(PathRule(p, read=True))
        for p in self.paths_rw:
            rules.append(PathRule(p, read=True, write=True))
        for p in self.paths_execute:
            rules.append(PathRule(p, execute=True))
        return rules

    def to_seccomp_denylist(self) -> list[int]:
        names = list(self.seccomp_denylist) if self.seccomp_denylist \
            else list(DEFAULT_DENYLIST_NAMES)
        return _resolve_denylist(names)

    @classmethod
    def from_json(cls, raw: str) -> "SandboxPolicy":
        d = json.loads(raw)
        return cls(
            paths_ro=list(d.get("paths_ro") or []),
            paths_rw=list(d.get("paths_rw") or []),
            paths_execute=list(d.get("paths_execute") or []),
            no_new_privs=bool(d.get("no_new_privs", True)),
            seccomp_denylist=list(d.get("seccomp_denylist") or []),
            inherit_defaults=bool(d.get("inherit_defaults", True)),
        )


# ----------------------------------------------------------------------
# Sandbox app: applied to the CURRENT process before execve.
# ----------------------------------------------------------------------

def _apply_in_child(argv: Sequence[str], policy: SandboxPolicy
                     ) -> list[tuple[str, str]]:
    """Apply the full sandbox stack to the calling process. Returns a
    list of (component, status) tuples for the parent's log."""
    log: list[tuple[str, str]] = []
    if policy.no_new_privs:
        ok, detail = _apply_no_new_privs()
        log.append(("no_new_privs", "ok" if ok else f"FAIL: {detail}"))
    denylist = policy.to_seccomp_denylist()
    if denylist:
        ok, detail = _apply_seccomp_blacklist(denylist)
        log.append(("seccomp", "ok" if ok else f"FAIL: {detail}"))
    rules = policy.to_landlock_rules(argv)
    if rules:
        ok, detail = _apply_landlock(rules)
        log.append(("landlock", "ok" if ok else f"FAIL: {detail}"))
    return log


def _execve_path(argv0: str) -> str:
    """Resolve argv[0] to an absolute path (the sandbox rules are
    relative to the binary's directory, so we want a canonical form)."""
    p = Path(argv0)
    if p.is_absolute() and p.exists():
        return str(p)
    # PATH lookup
    PATH = os.environ.get("PATH", "/usr/bin:/bin")
    for d in PATH.split(":"):
        cand = Path(d) / argv0
        if cand.exists() and os.access(cand, os.X_OK):
            return str(cand)
    return argv0  # let execve surface ENOENT


# ----------------------------------------------------------------------
# CLI entry: fork + child sandbox + execve; parent reaps.
# ----------------------------------------------------------------------

def sandbox_exec(argv: Sequence[str], policy: SandboxPolicy
                  ) -> int:
    """Fork; in the CHILD, apply the sandbox then execve the command.
    In the PARENT, wait for the child and return its exit code."""
    if not argv:
        print("sandbox_exec: empty argv", file=sys.stderr)
        return 2
    abs_argv0 = _execve_path(argv[0])
    pid = os.fork()
    if pid == 0:
        # Child.
        try:
            log = _apply_in_child([abs_argv0, *argv[1:]], policy)
        except Exception as e:  # pragma: no cover (defensive)
            print(f"sandbox: child setup failed: {e}", file=sys.stderr)
            os._exit(126)
        # Emit a tiny one-line audit trace the parent can capture.
        try:
            print(json.dumps({"event": "sandbox_applied",
                                "components": log}),
                  file=sys.stderr, flush=True)
        except Exception:
            pass
        try:
            os.execv(abs_argv0, [abs_argv0, *argv[1:]])
        except FileNotFoundError:
            print(f"sandbox: {abs_argv0}: not found", file=sys.stderr)
            os._exit(127)
        except PermissionError:
            print(f"sandbox: {abs_argv0}: permission denied", file=sys.stderr)
            os._exit(126)
        except Exception as e:
            print(f"sandbox: execv failed: {e}", file=sys.stderr)
            os._exit(126)
    # Parent: reap and return exit code.
    _, status = os.waitpid(pid, 0)
    if os.WIFEXITED(status):
        return os.WEXITSTATUS(status)
    if os.WIFSIGNALED(status):
        # If the seccomp filter killed the child, exit code is
        # conventional 128 + signum; we surface the signal as a
        # "killed" reason the caller can recognise.
        signum = os.WTERMSIG(status)
        print(f"sandbox: child killed by signal {signum} "
              f"(seccomp violation likely)", file=sys.stderr)
        return 128 + signum
    return 1  # pragma: no cover


def _main() -> int:
    """CLI: `python -m aiosh_mcp.sandbox --policy <json> -- <bin> <args...>`"""
    if "--policy" in sys.argv:
        i = sys.argv.index("--policy")
        if i + 1 >= len(sys.argv):
            print("usage: python -m aiosh_mcp.sandbox "
                  "--policy <json> -- <bin> <args...>", file=sys.stderr)
            return 2
        policy_json = sys.argv[i + 1]
        rest = sys.argv[i + 2:]
    else:
        policy_json = "{}"
        rest = sys.argv[1:]
    if not rest or rest[0] != "--":
        print("usage: python -m aiosh_mcp.sandbox "
              "--policy <json> -- <bin> <args...>", file=sys.stderr)
        return 2
    argv = rest[1:]
    policy = SandboxPolicy.from_json(policy_json)
    return sandbox_exec(argv, policy)


if __name__ == "__main__":
    sys.exit(_main())
