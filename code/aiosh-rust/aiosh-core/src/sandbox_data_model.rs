//! Sandbox Enforcement Data Model (T-02401..T-02410).
//!
//! Provides structured, type-safe representations for sandbox profiles, process isolation,
//! resource constraints, filesystem access rules, syscall filtering, and execution envelopes.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::canonical::canonical;
use crate::sandbox::SandboxPolicy;

pub const ERR_SANDBOX_INVALID_PATH: &str = "ERR_SANDBOX_INVALID_PATH";
pub const ERR_SANDBOX_INVALID_LIMIT: &str = "ERR_SANDBOX_INVALID_LIMIT";
pub const ERR_SANDBOX_POLICY_CONFLICT: &str = "ERR_SANDBOX_POLICY_CONFLICT";
pub const ERR_SANDBOX_EMPTY_COMMAND: &str = "ERR_SANDBOX_EMPTY_COMMAND";
pub const ERR_SANDBOX_BOUNDS_EXCEEDED: &str = "ERR_SANDBOX_BOUNDS_EXCEEDED";

pub const MIN_MEMORY_BYTES: u64 = 1_000_000;              // 1 MB
pub const MAX_MEMORY_BYTES: u64 = 68_719_476_736;         // 64 GB
pub const MIN_WALL_TIME_MS: u64 = 10;                     // 10 ms
pub const MAX_WALL_TIME_MS: u64 = 3_600_000;              // 1 hour
pub const MAX_PROCESSES_LIMIT: u32 = 4096;
pub const MAX_OPEN_FILES_LIMIT: u32 = 65536;

pub const MAX_PATHS_PER_POLICY: usize = 256;
pub const MAX_PATH_LEN: usize = 4096;
pub const MAX_PROFILE_NAME_LEN: usize = 128;
pub const MAX_ARGS_COUNT: usize = 1024;
pub const MAX_ARG_LEN: usize = 65536;
pub const MAX_ENV_VARS_COUNT: usize = 256;
pub const MAX_ENV_KEY_LEN: usize = 256;
pub const MAX_ENV_VAL_LEN: usize = 32768;

/// High-level predefined or custom sandbox profile type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxProfileType {
    Strict,
    Standard,
    Permissive,
    IsolatedDev,
    Custom,
}

/// Level of isolation enforced on the execution substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    ProcessOnly,
    RestrictedNamespaces,
    FullLandlockSeccomp,
}

/// Resource constraints imposed on the sandboxed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_time_ms: u64,
    pub max_wall_time_ms: u64,
    pub max_processes: u32,
    pub max_open_files: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 536_870_912, // 512 MB
            max_cpu_time_ms: 10_000,       // 10 s
            max_wall_time_ms: 30_000,      // 30 s
            max_processes: 32,
            max_open_files: 256,
        }
    }
}

impl ResourceLimits {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_memory_bytes < MIN_MEMORY_BYTES || self.max_memory_bytes > MAX_MEMORY_BYTES {
            return Err(format!("{}: memory limit {} out of bounds [{}, {}]",
                ERR_SANDBOX_INVALID_LIMIT, self.max_memory_bytes, MIN_MEMORY_BYTES, MAX_MEMORY_BYTES));
        }
        if self.max_wall_time_ms < MIN_WALL_TIME_MS || self.max_wall_time_ms > MAX_WALL_TIME_MS {
            return Err(format!("{}: wall time limit {} out of bounds [{}, {}]",
                ERR_SANDBOX_INVALID_LIMIT, self.max_wall_time_ms, MIN_WALL_TIME_MS, MAX_WALL_TIME_MS));
        }
        if self.max_processes == 0 || self.max_processes > MAX_PROCESSES_LIMIT {
            return Err(format!("{}: max processes {} out of bounds [1, {}]",
                ERR_SANDBOX_INVALID_LIMIT, self.max_processes, MAX_PROCESSES_LIMIT));
        }
        if self.max_open_files < 16 || self.max_open_files > MAX_OPEN_FILES_LIMIT {
            return Err(format!("{}: max open files {} out of bounds [16, {}]",
                ERR_SANDBOX_INVALID_LIMIT, self.max_open_files, MAX_OPEN_FILES_LIMIT));
        }
        Ok(())
    }
}

/// Path permissions and filesystem isolation rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilesystemPolicy {
    pub paths_ro: Vec<String>,
    pub paths_rw: Vec<String>,
    pub paths_execute: Vec<String>,
    pub paths_denied: Vec<String>,
    pub allow_cwd_rw: bool,
    pub allow_tmp_rw: bool,
}

impl FilesystemPolicy {
    pub fn validate(&self) -> Result<(), String> {
        let total_paths = self.paths_ro.len() + self.paths_rw.len()
            + self.paths_execute.len() + self.paths_denied.len();
        if total_paths > MAX_PATHS_PER_POLICY {
            return Err(format!("{}: total paths count {} exceeds maximum {}",
                ERR_SANDBOX_BOUNDS_EXCEEDED, total_paths, MAX_PATHS_PER_POLICY));
        }
        let all_paths = self.paths_ro.iter().chain(&self.paths_rw)
            .chain(&self.paths_execute).chain(&self.paths_denied);
        for p in all_paths {
            if p.trim().is_empty() {
                return Err(format!("{}: path cannot be empty", ERR_SANDBOX_INVALID_PATH));
            }
            if p.len() > MAX_PATH_LEN {
                return Err(format!("{}: path length {} exceeds maximum {}",
                    ERR_SANDBOX_BOUNDS_EXCEEDED, p.len(), MAX_PATH_LEN));
            }
            if p.contains("..") {
                return Err(format!("{}: path {:?} contains directory traversal '..'", ERR_SANDBOX_INVALID_PATH, p));
            }
        }

        // Check for direct conflicts between read-only and read-write
        for ro in &self.paths_ro {
            if self.paths_rw.contains(ro) {
                return Err(format!("{}: path {:?} cannot be both read-only and read-write", ERR_SANDBOX_POLICY_CONFLICT, ro));
            }
            if self.paths_denied.contains(ro) {
                return Err(format!("{}: path {:?} cannot be both read-only and denied", ERR_SANDBOX_POLICY_CONFLICT, ro));
            }
        }
        for rw in &self.paths_rw {
            if self.paths_denied.contains(rw) {
                return Err(format!("{}: path {:?} cannot be both read-write and denied", ERR_SANDBOX_POLICY_CONFLICT, rw));
            }
        }
        Ok(())
    }
}

/// Network restriction mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkIsolationMode {
    Disabled,
    LoopbackOnly,
    FilteredEgress { allowed_ports: Vec<u16> },
    Unrestricted,
}

impl Default for NetworkIsolationMode {
    fn default() -> Self {
        Self::LoopbackOnly
    }
}

/// Action to take on syscall filter match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyscallAction {
    KillProcess,
    ReturnErrno(i32),
    Log,
    Allow,
}

/// Syscall filtering configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyscallPolicy {
    pub no_new_privs: bool,
    pub default_action: SyscallAction,
    pub denylist: Vec<String>,
    pub allowlist: Vec<String>,
}

impl Default for SyscallPolicy {
    fn default() -> Self {
        Self {
            no_new_privs: true,
            default_action: SyscallAction::Allow,
            denylist: vec![
                "ptrace".into(), "mount".into(), "umount2".into(), "reboot".into(),
                "kexec_load".into(), "kexec_file_load".into(), "init_module".into(),
                "finit_module".into(), "delete_module".into(), "setuid".into(),
                "setgid".into(), "setreuid".into(), "setregid".into(), "setresuid".into(),
                "setresgid".into(), "chroot".into(), "pivot_root".into(),
            ],
            allowlist: vec![],
        }
    }
}

/// Environment variable filtering and injection policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EnvironmentPolicy {
    pub clean_env: bool,
    pub allow_vars: Vec<String>,
    pub injected_vars: BTreeMap<String, String>,
}

impl EnvironmentPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.allow_vars.len() > MAX_ENV_VARS_COUNT {
            return Err(format!("{}: allow_vars count {} exceeds maximum {}",
                ERR_SANDBOX_BOUNDS_EXCEEDED, self.allow_vars.len(), MAX_ENV_VARS_COUNT));
        }
        if self.injected_vars.len() > MAX_ENV_VARS_COUNT {
            return Err(format!("{}: injected_vars count {} exceeds maximum {}",
                ERR_SANDBOX_BOUNDS_EXCEEDED, self.injected_vars.len(), MAX_ENV_VARS_COUNT));
        }
        for (k, v) in &self.injected_vars {
            if k.len() > MAX_ENV_KEY_LEN {
                return Err(format!("{}: env var key length {} exceeds maximum {}",
                    ERR_SANDBOX_BOUNDS_EXCEEDED, k.len(), MAX_ENV_KEY_LEN));
            }
            if v.len() > MAX_ENV_VAL_LEN {
                return Err(format!("{}: env var value length {} exceeds maximum {}",
                    ERR_SANDBOX_BOUNDS_EXCEEDED, v.len(), MAX_ENV_VAL_LEN));
            }
        }
        Ok(())
    }
}

/// Unified Sandbox Profile specifying complete containment envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxProfile {
    pub name: String,
    pub profile_type: SandboxProfileType,
    pub isolation_level: IsolationLevel,
    pub resources: ResourceLimits,
    pub filesystem: FilesystemPolicy,
    pub network: NetworkIsolationMode,
    pub syscall: SyscallPolicy,
    pub environment: EnvironmentPolicy,
}

impl Default for SandboxProfile {
    fn default() -> Self {
        Self::standard()
    }
}

impl SandboxProfile {
    pub fn standard() -> Self {
        Self {
            name: "standard".into(),
            profile_type: SandboxProfileType::Standard,
            isolation_level: IsolationLevel::FullLandlockSeccomp,
            resources: ResourceLimits::default(),
            filesystem: FilesystemPolicy {
                paths_ro: vec!["/usr".into(), "/lib".into(), "/lib64".into(), "/etc".into()],
                paths_rw: vec!["/tmp".into()],
                paths_execute: vec!["/usr/bin".into(), "/bin".into()],
                paths_denied: vec![],
                allow_cwd_rw: true,
                allow_tmp_rw: true,
            },
            network: NetworkIsolationMode::LoopbackOnly,
            syscall: SyscallPolicy::default(),
            environment: EnvironmentPolicy {
                clean_env: false,
                allow_vars: vec!["PATH".into(), "USER".into(), "HOME".into(), "LANG".into()],
                injected_vars: BTreeMap::new(),
            },
        }
    }

    pub fn strict() -> Self {
        let mut prof = Self::standard();
        prof.name = "strict".into();
        prof.profile_type = SandboxProfileType::Strict;
        prof.resources.max_memory_bytes = 268_435_456; // 256 MB
        prof.resources.max_wall_time_ms = 10_000;      // 10 s
        prof.network = NetworkIsolationMode::Disabled;
        prof.environment.clean_env = true;
        prof
    }

    pub fn permissive() -> Self {
        let mut prof = Self::standard();
        prof.name = "permissive".into();
        prof.profile_type = SandboxProfileType::Permissive;
        prof.network = NetworkIsolationMode::Unrestricted;
        prof.syscall.denylist.clear();
        prof
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("ERR_SANDBOX_INVALID_NAME: profile name cannot be empty".into());
        }
        if self.name.len() > MAX_PROFILE_NAME_LEN {
            return Err(format!("{}: profile name length {} exceeds maximum {}",
                ERR_SANDBOX_BOUNDS_EXCEEDED, self.name.len(), MAX_PROFILE_NAME_LEN));
        }
        self.resources.validate()?;
        self.filesystem.validate()?;
        self.environment.validate()?;
        Ok(())
    }

    pub fn to_legacy_policy(&self) -> SandboxPolicy {
        SandboxPolicy {
            paths_ro: self.filesystem.paths_ro.clone(),
            paths_rw: self.filesystem.paths_rw.clone(),
            paths_execute: self.filesystem.paths_execute.clone(),
            no_new_privs: self.syscall.no_new_privs,
            seccomp_denylist: self.syscall.denylist.clone(),
            inherit_defaults: true,
        }
    }

    pub fn from_legacy_policy(legacy: &SandboxPolicy) -> Self {
        let mut prof = Self::standard();
        prof.filesystem.paths_ro = legacy.paths_ro.clone();
        prof.filesystem.paths_rw = legacy.paths_rw.clone();
        prof.filesystem.paths_execute = legacy.paths_execute.clone();
        prof.syscall.no_new_privs = legacy.no_new_privs;
        prof.syscall.denylist = legacy.seccomp_denylist.clone();
        prof
    }

    pub fn canonical_hash(&self) -> Result<String, String> {
        let val = serde_json::to_value(self).map_err(|e| format!("failed to serialize profile: {}", e))?;
        Ok(crate::canonical::sha256_hex(&canonical(&val)))
    }
}

/// Request to execute an arbitrary command within a defined sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxExecutionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub profile: SandboxProfile,
    pub session_id: Option<String>,
    pub pep_grant_id: Option<String>,
    pub stdin_data: Option<String>,
}

impl SandboxExecutionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.command.trim().is_empty() {
            return Err(format!("{}: command cannot be empty", ERR_SANDBOX_EMPTY_COMMAND));
        }
        if self.command.len() > MAX_PATH_LEN {
            return Err(format!("{}: command path length exceeds maximum {}",
                ERR_SANDBOX_BOUNDS_EXCEEDED, MAX_PATH_LEN));
        }
        if self.args.len() > MAX_ARGS_COUNT {
            return Err(format!("{}: args count {} exceeds maximum {}",
                ERR_SANDBOX_BOUNDS_EXCEEDED, self.args.len(), MAX_ARGS_COUNT));
        }
        for a in &self.args {
            if a.len() > MAX_ARG_LEN {
                return Err(format!("{}: arg length {} exceeds maximum {}",
                    ERR_SANDBOX_BOUNDS_EXCEEDED, a.len(), MAX_ARG_LEN));
            }
        }
        self.profile.validate()?;
        if let Some(ref cwd) = self.cwd {
            if cwd.len() > MAX_PATH_LEN {
                return Err(format!("{}: cwd length exceeds maximum {}",
                    ERR_SANDBOX_BOUNDS_EXCEEDED, MAX_PATH_LEN));
            }
            if cwd.contains("..") {
                return Err(format!("{}: cwd cannot contain '..'", ERR_SANDBOX_INVALID_PATH));
            }
        }
        Ok(())
    }
}

/// Component application status reported from sandbox runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxComponentStatus {
    pub component: String,
    pub status: String,
    pub detail: Option<String>,
}

/// Outcome status of the sandboxed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxExecutionStatus {
    Success,
    TimedOut,
    Signaled(i32),
    Violation(String),
    Error(String),
}

/// Result envelope returned from sandboxed process execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    pub exit_code: i32,
    pub status: SandboxExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub components_applied: Vec<SandboxComponentStatus>,
    pub audit_hash: Option<String>,
}

impl SandboxExecutionResult {
    pub fn is_success(&self) -> bool {
        self.exit_code == 0 && matches!(self.status, SandboxExecutionStatus::Success)
    }
}

/// Fluent builder for constructing customized SandboxProfile instances safely.
#[derive(Debug, Default)]
pub struct SandboxProfileBuilder {
    name: String,
    profile_type: Option<SandboxProfileType>,
    isolation_level: Option<IsolationLevel>,
    resources: ResourceLimits,
    filesystem: FilesystemPolicy,
    network: NetworkIsolationMode,
    syscall: SyscallPolicy,
    environment: EnvironmentPolicy,
}

impl SandboxProfileBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let mut builder = Self::default();
        builder.name = name.into();
        builder.profile_type = Some(SandboxProfileType::Custom);
        builder.isolation_level = Some(IsolationLevel::FullLandlockSeccomp);
        builder
    }

    pub fn profile_type(mut self, p_type: SandboxProfileType) -> Self {
        self.profile_type = Some(p_type);
        self
    }

    pub fn isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = Some(level);
        self
    }

    pub fn max_memory_bytes(mut self, bytes: u64) -> Self {
        self.resources.max_memory_bytes = bytes;
        self
    }

    pub fn max_wall_time_ms(mut self, ms: u64) -> Self {
        self.resources.max_wall_time_ms = ms;
        self
    }

    pub fn add_ro_path(mut self, path: impl Into<String>) -> Self {
        self.filesystem.paths_ro.push(path.into());
        self
    }

    pub fn add_rw_path(mut self, path: impl Into<String>) -> Self {
        self.filesystem.paths_rw.push(path.into());
        self
    }

    pub fn add_execute_path(mut self, path: impl Into<String>) -> Self {
        self.filesystem.paths_execute.push(path.into());
        self
    }

    pub fn add_denied_path(mut self, path: impl Into<String>) -> Self {
        self.filesystem.paths_denied.push(path.into());
        self
    }

    pub fn network_mode(mut self, mode: NetworkIsolationMode) -> Self {
        self.network = mode;
        self
    }

    pub fn add_denied_syscall(mut self, syscall: impl Into<String>) -> Self {
        self.syscall.denylist.push(syscall.into());
        self
    }

    pub fn set_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.injected_vars.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<SandboxProfile, String> {
        let profile = SandboxProfile {
            name: if self.name.trim().is_empty() { "custom".into() } else { self.name },
            profile_type: self.profile_type.unwrap_or(SandboxProfileType::Custom),
            isolation_level: self.isolation_level.unwrap_or(IsolationLevel::FullLandlockSeccomp),
            resources: self.resources,
            filesystem: self.filesystem,
            network: self.network,
            syscall: self.syscall,
            environment: self.environment,
        };
        profile.validate()?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_profile_validates() {
        let p = SandboxProfile::standard();
        assert!(p.validate().is_ok());
        assert_eq!(p.profile_type, SandboxProfileType::Standard);
    }

    #[test]
    fn test_strict_profile_validates() {
        let p = SandboxProfile::strict();
        assert!(p.validate().is_ok());
        assert_eq!(p.network, NetworkIsolationMode::Disabled);
    }

    #[test]
    fn test_permissive_profile_validates() {
        let p = SandboxProfile::permissive();
        assert!(p.validate().is_ok());
        assert_eq!(p.network, NetworkIsolationMode::Unrestricted);
    }

    #[test]
    fn test_path_traversal_detection() {
        let mut p = SandboxProfile::standard();
        p.filesystem.paths_ro.push("/etc/../shadow".into());
        let res = p.validate();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains(ERR_SANDBOX_INVALID_PATH));
    }

    #[test]
    fn test_conflicting_paths_rejected() {
        let mut p = SandboxProfile::standard();
        p.filesystem.paths_ro.push("/tmp/shared".into());
        p.filesystem.paths_rw.push("/tmp/shared".into());
        let res = p.validate();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains(ERR_SANDBOX_POLICY_CONFLICT));
    }

    #[test]
    fn test_resource_limits_bounds() {
        let mut limits = ResourceLimits::default();
        limits.max_memory_bytes = 100; // Below 1MB
        assert!(limits.validate().is_err());

        limits.max_memory_bytes = 512 * 1024 * 1024;
        limits.max_wall_time_ms = 5; // Below 10ms
        assert!(limits.validate().is_err());
    }

    #[test]
    fn test_profile_builder() {
        let p = SandboxProfileBuilder::new("custom-worker")
            .max_memory_bytes(1_000_000_000)
            .add_ro_path("/usr")
            .add_rw_path("/var/app/scratch")
            .set_env_var("AIOS_SANDBOX", "1")
            .build();
        assert!(p.is_ok());
        let prof = p.unwrap();
        assert_eq!(prof.name, "custom-worker");
        assert_eq!(prof.environment.injected_vars.get("AIOS_SANDBOX").map(|s| s.as_str()), Some("1"));
    }

    #[test]
    fn test_execution_request_validation() {
        let req = SandboxExecutionRequest {
            command: "".into(),
            args: vec![],
            cwd: None,
            profile: SandboxProfile::standard(),
            session_id: None,
            pep_grant_id: None,
            stdin_data: None,
        };
        assert!(req.validate().is_err());

        let req_valid = SandboxExecutionRequest {
            command: "/usr/bin/echo".into(),
            args: vec!["hello".into()],
            cwd: Some("/tmp".into()),
            profile: SandboxProfile::standard(),
            session_id: Some("sess-1".into()),
            pep_grant_id: Some("grant-1".into()),
            stdin_data: None,
        };
        assert!(req_valid.validate().is_ok());
    }

    #[test]
    fn test_legacy_policy_roundtrip() {
        let prof = SandboxProfile::standard();
        let leg = prof.to_legacy_policy();
        assert_eq!(leg.paths_ro, prof.filesystem.paths_ro);
        let back = SandboxProfile::from_legacy_policy(&leg);
        assert_eq!(back.filesystem.paths_ro, leg.paths_ro);
    }

    #[test]
    fn test_canonical_hash_deterministic() {
        let p1 = SandboxProfile::standard();
        let p2 = SandboxProfile::standard();
        let h1 = p1.canonical_hash().unwrap();
        let h2 = p2.canonical_hash().unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }
}

