# T-01230: Verification Run Output

## Test Suite Execution Results

### 1. Master Package Test Suite (`tools/test_package_suites.py`)
```text
[+] PM1 package data model integrity & invariants (PM1..PM5)
[+] PM2 package core service integrity & invariants (CS1..CS5)
[+] PM3 package CLI surface commands & options (validate/list/show/search/plan/apply)

PASS: package_suites criteria (PM1..PM3)
```

### 2. Cargo Unit Tests (`test_cmd_package_flow`)
```text
running 1 test
test task_cli_tests::test_cmd_package_flow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.70s
```

### 3. Binary Execution Verification (`aiosh package list --json`)
```json
[
  {
    "name": "apk-tools",
    "version": "2.14.0-r5",
    "architecture": "x86_64",
    "format": "apk",
    "state": "installed",
    "description": "Alpine Package Keeper - package management tools",
    "installed_size_bytes": 524288,
    "sha256": "6666666666666666666666666666666666666666666666666666666666666666",
    "repository_url": "https://dl-cdn.alpinelinux.org/alpine/v3.19/main",
    "dependencies": [
      {
        "name": "musl",
        "version_constraint": ">= 1.2.4",
        "optional": false
      }
    ]
  },
  {
    "name": "bash",
    "version": "5.2.15-2+b2",
    "architecture": "amd64",
    "format": "deb",
    "state": "installed",
    "description": "GNU Bourne Again SHell",
    "installed_size_bytes": 6291456,
    "sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    "repository_url": "https://deb.debian.org/debian",
    "dependencies": [
      {
        "name": "libc6",
        "version_constraint": ">= 2.36",
        "optional": false
      }
    ]
  },
  {
    "name": "busybox",
    "version": "1.36.1-r15",
    "architecture": "x86_64",
    "format": "apk",
    "state": "installed",
    "description": "Size optimized toolbox of many common UNIX utilities",
    "installed_size_bytes": 1048576,
    "sha256": "4444444444444444444444444444444444444444444444444444444444444444",
    "repository_url": "https://dl-cdn.alpinelinux.org/alpine/v3.19/main",
    "dependencies": [
      {
        "name": "musl",
        "version_constraint": ">= 1.2.4",
        "optional": false
      }
    ]
  },
  {
    "name": "coreutils",
    "version": "9.1-1",
    "architecture": "amd64",
    "format": "deb",
    "state": "installed",
    "description": "GNU core utilities",
    "installed_size_bytes": 16777216,
    "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "repository_url": "https://deb.debian.org/debian",
    "dependencies": [
      {
        "name": "libc6",
        "version_constraint": ">= 2.36",
        "optional": false
      }
    ]
  },
  {
    "name": "curl",
    "version": "7.88.1-10+deb12u5",
    "architecture": "amd64",
    "format": "deb",
    "state": "available",
    "description": "command line tool for transferring data with URL syntax",
    "installed_size_bytes": 4194304,
    "sha256": "2222222222222222222222222222222222222222222222222222222222222222",
    "repository_url": "https://deb.debian.org/debian",
    "dependencies": [
      {
        "name": "libc6",
        "version_constraint": ">= 2.36",
        "optional": false
      },
      {
        "name": "libssl3",
        "version_constraint": ">= 3.0.0",
        "optional": false
      }
    ]
  },
  {
    "name": "libc6",
    "version": "2.36-9+deb12u7",
    "architecture": "amd64",
    "format": "deb",
    "state": "installed",
    "description": "GNU C Library: Shared libraries",
    "installed_size_bytes": 12582912,
    "sha256": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    "repository_url": "https://deb.debian.org/debian",
    "dependencies": []
  },
  {
    "name": "libssl3",
    "version": "3.0.13-1~deb12u1",
    "architecture": "amd64",
    "format": "deb",
    "state": "available",
    "description": "Secure Sockets Layer toolkit - shared libraries",
    "installed_size_bytes": 5242880,
    "sha256": "3333333333333333333333333333333333333333333333333333333333333333",
    "repository_url": "https://deb.debian.org/debian",
    "dependencies": [
      {
        "name": "libc6",
        "version_constraint": ">= 2.36",
        "optional": false
      }
    ]
  },
  {
    "name": "musl",
    "version": "1.2.4-r2",
    "architecture": "x86_64",
    "format": "apk",
    "state": "installed",
    "description": "the musl c library (libc) implementation",
    "installed_size_bytes": 629145,
    "sha256": "5555555555555555555555555555555555555555555555555555555555555555",
    "repository_url": "https://dl-cdn.alpinelinux.org/alpine/v3.19/main",
    "dependencies": []
  }
]
```

### 4. Task Documentation Validator (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```
