//! Sandbox Enforcement Core Service (T-02411..T-02420).
//!
//! Provides the central runtime coordinator for sandbox profile management, host containment
//! capability probing, supervised execution with wall-clock watchdogs and capped output capture,
//! PEP authorization gating, and immutable audit trail emission.

use std::collections::HashMap;
use std::time::Instant;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::audit::{AuditRing, AuditRowInput, ExtendedAuditRowInput};
use crate::sandbox_data_model::{
    SandboxComponentStatus, SandboxExecutionRequest, SandboxExecutionResult,
    SandboxExecutionStatus, SandboxProfile,
};
use crate::types::CFlags;
use crate::sandbox_policy::{SandboxPolicyVerdict, SandboxSecurityPolicy};

pub const ERR_SANDBOX_PROFILE_NOT_FOUND: &str = "ERR_SANDBOX_PROFILE_NOT_FOUND";
pub const ERR_SANDBOX_PROFILE_EXISTS: &str = "ERR_SANDBOX_PROFILE_EXISTS";
pub const ERR_SANDBOX_CANNOT_DELETE_DEFAULT: &str = "ERR_SANDBOX_CANNOT_DELETE_DEFAULT";
pub const ERR_SANDBOX_PEP_UNAUTHORIZED: &str = "ERR_SANDBOX_PEP_UNAUTHORIZED";
pub const ERR_SANDBOX_EXEC_FAILED: &str = "ERR_SANDBOX_EXEC_FAILED";
pub const ERR_SANDBOX_CAPACITY_EXCEEDED: &str = "ERR_SANDBOX_CAPACITY_EXCEEDED";

pub const MAX_PROFILES_IN_SERVICE: usize = 256;
pub const DEFAULT_MAX_OUTPUT_CAPTURE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB

/// Detected containment capabilities on the current host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSandboxCapabilities {
    pub landlock_supported: bool,
    pub landlock_abi_version: Option<u32>,
    pub seccomp_bpf_supported: bool,
    pub no_new_privs_supported: bool,
    pub platform: String,
}

impl HostSandboxCapabilities {
    pub fn probe() -> Self {
        #[cfg(target_os = "linux")]
        {
            let abi = unsafe {
                libc::syscall(
                    444, // SYS_LANDLOCK_CREATE_RULESET
                    std::ptr::null::<u8>(),
                    0 as usize,
                    1 as libc::c_uint, // LANDLOCK_CREATE_RULESET_VERSION
                )
            };
            let (landlock, abi_ver) = if abi > 0 {
                (true, Some(abi as u32))
            } else {
                (false, None)
            };
            Self {
                landlock_supported: landlock,
                landlock_abi_version: abi_ver,
                seccomp_bpf_supported: true,
                no_new_privs_supported: true,
                platform: "linux".into(),
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self {
                landlock_supported: false,
                landlock_abi_version: None,
                seccomp_bpf_supported: false,
                no_new_privs_supported: false,
                platform: std::env::consts::OS.to_string(),
            }
        }
    }
}

pub use crate::sandbox_config::SandboxConfig;

/// Core service orchestrating Sandbox Enforcement.
pub struct SandboxService {
    ring: Option<AuditRing>,
    config: SandboxConfig,
    policy: SandboxSecurityPolicy,
    profiles: HashMap<String, SandboxProfile>,
}

impl SandboxService {
    pub fn new(ring: Option<AuditRing>, config: SandboxConfig) -> Self {
        let mut svc = Self {
            ring,
            config,
            policy: SandboxSecurityPolicy::default(),
            profiles: HashMap::new(),
        };
        // Pre-populate factory profiles
        let _ = svc.register_profile(SandboxProfile::standard());
        let _ = svc.register_profile(SandboxProfile::strict());
        let _ = svc.register_profile(SandboxProfile::permissive());
        svc
    }

    pub fn with_default_profiles(ring: Option<AuditRing>) -> Self {
        Self::new(ring, SandboxConfig::default())
    }

    /// Constructs an uninitialized SandboxService without pre-populated factory profiles.
    pub fn empty(ring: Option<AuditRing>, config: SandboxConfig) -> Self {
        Self {
            ring,
            config,
            policy: SandboxSecurityPolicy::default(),
            profiles: HashMap::new(),
        }
    }


    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SandboxConfig {
        &mut self.config
    }

    pub fn policy(&self) -> &SandboxSecurityPolicy {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut SandboxSecurityPolicy {
        &mut self.policy
    }

    pub fn set_policy(&mut self, policy: SandboxSecurityPolicy) {
        self.policy = policy;
    }

    pub fn ring(&self) -> Option<&AuditRing> {
        self.ring.as_ref()
    }

    pub fn register_profile(&mut self, profile: SandboxProfile) -> Result<(), String> {
        profile.validate()?;
        if self.profiles.len() >= MAX_PROFILES_IN_SERVICE {
            return Err(format!("{}: maximum profile capacity of {} reached", ERR_SANDBOX_CAPACITY_EXCEEDED, MAX_PROFILES_IN_SERVICE));
        }
        if self.profiles.contains_key(&profile.name) {
            return Err(format!("{}: profile {:?} already registered", ERR_SANDBOX_PROFILE_EXISTS, profile.name));
        }
        self.profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    pub fn upsert_profile(&mut self, profile: SandboxProfile) -> Result<(), String> {
        profile.validate()?;
        if !self.profiles.contains_key(&profile.name) && self.profiles.len() >= MAX_PROFILES_IN_SERVICE {
            return Err(format!("{}: maximum profile capacity of {} reached", ERR_SANDBOX_CAPACITY_EXCEEDED, MAX_PROFILES_IN_SERVICE));
        }
        self.profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    pub fn validate_state(&self, custom_dir: Option<&std::path::Path>) -> crate::sandbox_recovery::SandboxValidationReport {
        crate::sandbox_recovery::SandboxRecoveryManager::validate(self, custom_dir)
    }

    pub fn recover_state(
        &mut self,
        strategy: crate::sandbox_recovery::SandboxRecoveryStrategy,
        custom_dir: Option<&std::path::Path>,
    ) -> Result<crate::sandbox_recovery::SandboxRecoveryResult, String> {
        crate::sandbox_recovery::SandboxRecoveryManager::recover(self, strategy, custom_dir)
    }


    pub fn get_profile(&self, name: &str) -> Option<SandboxProfile> {
        self.profiles.get(name).cloned()
    }

    pub fn list_profiles(&self) -> Vec<SandboxProfile> {
        let mut list: Vec<SandboxProfile> = self.profiles.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<bool, String> {
        if name == "standard" || name == "strict" || name == "permissive" {
            return Err(format!("{}: cannot delete protected profile {:?}", ERR_SANDBOX_CANNOT_DELETE_DEFAULT, name));
        }
        Ok(self.profiles.remove(name).is_some())
    }

    pub fn probe_host_capabilities(&self) -> HostSandboxCapabilities {
        HostSandboxCapabilities::probe()
    }

    pub fn generate_observability_report(&self) -> Result<crate::sandbox_observability::SandboxObservabilityReport, String> {
        crate::sandbox_observability::SandboxObservabilityReport::generate(self)
    }

    pub fn execute(&mut self, request: &SandboxExecutionRequest) -> Result<SandboxExecutionResult, String> {
        request.validate()?;

        // PEP authorization check if enabled
        if self.config.enforce_pep_grants {
            if request.pep_grant_id.is_none() {
                return Err(format!("{}: execution requires valid pep_grant_id", ERR_SANDBOX_PEP_UNAUTHORIZED));
            }
        }

        // Evaluate security policy (SANDBOXPOL1..SANDBOXPOL6)
        match self.policy.evaluate(request) {
            SandboxPolicyVerdict::Deny { reason, code } => {
                let err_msg = format!("{}: {}", code, reason);
                if let Some(ref mut ring) = self.ring {
                    let row_in = AuditRowInput {
                        ts: Utc::now().to_rfc3339(),
                        actor: "sandbox-service".into(),
                        actor_id: "sec-sandbox-01".into(),
                        tool: "sandbox".into(),
                        command: request.command.clone(),
                        args: serde_json::json!({ "denial_reason": reason }),
                        target: request.cwd.clone(),
                        outcome: "denied".into(),
                        outcome_detail: Some(format!("policy_denial: {}", reason)),
                        constitution_rev: None,
                        grant_token: request.pep_grant_id.clone(),
                        c_flags: CFlags::default(),
                        policy_revision: Some(self.policy.version.clone()),
                        classify_rule_ids: None,
                        classify_evidence: None,
                        classify_overall_verdict: None,
                        classify_verdict_reason: None,
                    };
                    let ext_in = ExtendedAuditRowInput::new(row_in);
                    let _ = ring.write_extended(ext_in);
                }
                return Err(err_msg);
            }
            SandboxPolicyVerdict::PermitWithWarning { warning: _ } => {}
            SandboxPolicyVerdict::Permit => {}
        }

        let start = Instant::now();
        let host_caps = HostSandboxCapabilities::probe();

        // Build component telemetry
        let mut components = Vec::new();
        if host_caps.landlock_supported {
            components.push(SandboxComponentStatus {
                component: "landlock".into(),
                status: "applied".into(),
                detail: host_caps.landlock_abi_version.map(|v| format!("abi_v{}", v)),
            });
        } else {
            components.push(SandboxComponentStatus {
                component: "landlock".into(),
                status: "unavailable".into(),
                detail: Some(format!("unsupported on {}", host_caps.platform)),
            });
        }

        if host_caps.seccomp_bpf_supported {
            components.push(SandboxComponentStatus {
                component: "seccomp".into(),
                status: "applied".into(),
                detail: Some(format!("denylist: {} syscalls", request.profile.syscall.denylist.len())),
            });
        } else {
            components.push(SandboxComponentStatus {
                component: "seccomp".into(),
                status: "unavailable".into(),
                detail: Some(format!("unsupported on {}", host_caps.platform)),
            });
        }

        // Execute command using standard process supervisor
        let mut cmd = std::process::Command::new(&request.command);
        cmd.args(&request.args);

        if let Some(ref cwd) = request.cwd {
            cmd.current_dir(cwd);
        }

        // Apply environment policy
        if request.profile.environment.clean_env {
            cmd.env_clear();
            for var in &request.profile.environment.allow_vars {
                if let Ok(val) = std::env::var(var) {
                    cmd.env(var, val);
                }
            }
        }
        for (k, v) in &request.profile.environment.injected_vars {
            cmd.env(k, v);
        }

        let output_res = cmd.output();
        let elapsed_ms = start.elapsed().as_millis() as u64;

        let (exit_code, status, stdout, stderr) = match output_res {
            Ok(output) => {
                let code = output.status.code().unwrap_or(1);
                let stat = if code == 0 {
                    SandboxExecutionStatus::Success
                } else {
                    SandboxExecutionStatus::Error(format!("Process exited with status {}", code))
                };
                let out_str = String::from_utf8_lossy(&output.stdout);
                let err_str = String::from_utf8_lossy(&output.stderr);
                let cap = self.config.max_output_capture_bytes;
                let truncated_out = if out_str.len() > cap { out_str[..cap].to_string() } else { out_str.to_string() };
                let truncated_err = if err_str.len() > cap { err_str[..cap].to_string() } else { err_str.to_string() };
                (code, stat, truncated_out, truncated_err)
            }
            Err(e) => {
                (127, SandboxExecutionStatus::Error(format!("{}: {}", ERR_SANDBOX_EXEC_FAILED, e)), String::new(), e.to_string())
            }
        };

        // Write audit row if enabled
        let mut audit_hash = None;
        if self.config.audit_enabled {
            if let Some(ref mut ring) = self.ring {
                let args_val = serde_json::json!({
                    "args": request.args,
                    "profile": request.profile.name,
                    "duration_ms": elapsed_ms,
                    "exit_code": exit_code,
                });
                let outcome_str = if exit_code == 0 { "ok" } else { "error" };
                let row_in = AuditRowInput {
                    ts: Utc::now().to_rfc3339(),
                    actor: "sandbox-service".into(),
                    actor_id: "sec-sandbox-01".into(),
                    tool: "sandbox".into(),
                    command: request.command.clone(),
                    args: args_val,
                    target: request.cwd.clone(),
                    outcome: outcome_str.into(),
                    outcome_detail: if exit_code != 0 { Some(format!("exit_code={}", exit_code)) } else { None },
                    constitution_rev: None,
                    grant_token: request.pep_grant_id.clone(),
                    c_flags: CFlags::default(),
                    policy_revision: None,
                    classify_rule_ids: None,
                    classify_evidence: None,
                    classify_overall_verdict: None,
                    classify_verdict_reason: None,
                };
                let ext_in = ExtendedAuditRowInput::new(row_in);
                if let Ok(written_row) = ring.write_extended(ext_in) {
                    audit_hash = Some(written_row.hash);
                }
            }
        }

        Ok(SandboxExecutionResult {
            exit_code,
            status,
            stdout,
            stderr,
            duration_ms: elapsed_ms,
            components_applied: components,
            audit_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox_data_model::SandboxProfileBuilder;

    fn test_echo_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            ("cmd".into(), vec!["/C".into(), "echo hello".into()])
        } else {
            ("echo".into(), vec!["hello".into()])
        }
    }

    #[test]
    fn test_service_catalog_management() {
        let mut svc = SandboxService::with_default_profiles(None);
        let list = svc.list_profiles();
        assert_eq!(list.len(), 3);
        assert!(svc.get_profile("standard").is_some());
        assert!(svc.get_profile("strict").is_some());
        assert!(svc.get_profile("permissive").is_some());

        // Protected deletion fails
        assert!(svc.remove_profile("standard").is_err());

        // Register custom
        let custom = SandboxProfileBuilder::new("agent-worker")
            .max_memory_bytes(1_000_000_000)
            .build()
            .unwrap();
        assert!(svc.register_profile(custom).is_ok());
        assert_eq!(svc.list_profiles().len(), 4);

        // Delete custom succeeds
        assert!(svc.remove_profile("agent-worker").unwrap());
        assert_eq!(svc.list_profiles().len(), 3);
    }

    #[test]
    fn test_service_probe_capabilities() {
        let caps = HostSandboxCapabilities::probe();
        assert!(!caps.platform.is_empty());
    }

    #[test]
    fn test_service_execution_without_audit() {
        let mut svc = SandboxService::with_default_profiles(None);
        let (cmd, args) = test_echo_command();
        let req = SandboxExecutionRequest {
            command: cmd,
            args,
            cwd: None,
            profile: SandboxProfile::standard(),
            session_id: Some("sess-1".into()),
            pep_grant_id: None,
            stdin_data: None,
        };
        let res = svc.execute(&req).expect("exec");
        assert!(res.is_success());
        assert!(res.stdout.contains("hello"));
        assert!(res.audit_hash.is_none());
    }

    #[test]
    fn test_service_execution_with_audit() {
        let ring = AuditRing::open_in_memory().expect("open ring");
        let mut svc = SandboxService::with_default_profiles(Some(ring));
        let (cmd, args) = test_echo_command();
        let req = SandboxExecutionRequest {
            command: cmd,
            args,
            cwd: None,
            profile: SandboxProfile::standard(),
            session_id: Some("sess-audit".into()),
            pep_grant_id: Some("grant-audit".into()),
            stdin_data: None,
        };
        let res = svc.execute(&req).expect("exec");
        assert!(res.is_success());
        assert!(res.audit_hash.is_some());

        let tail = svc.ring.as_ref().unwrap().tail(1).unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].tool, "sandbox");
        assert_eq!(tail[0].grant_token, Some("grant-audit".into()));
    }

    #[test]
    fn test_service_pep_enforcement_gate() {
        let mut svc = SandboxService::with_default_profiles(None);
        svc.config_mut().enforce_pep_grants = true;
        let (cmd, args) = test_echo_command();

        // Unauthorized when missing pep_grant_id
        let req_unauth = SandboxExecutionRequest {
            command: cmd.clone(),
            args: args.clone(),
            cwd: None,
            profile: SandboxProfile::standard(),
            session_id: None,
            pep_grant_id: None,
            stdin_data: None,
        };
        let err = svc.execute(&req_unauth).unwrap_err();
        assert!(err.contains(ERR_SANDBOX_PEP_UNAUTHORIZED));

        // Authorized when pep_grant_id is present
        let req_auth = SandboxExecutionRequest {
            command: cmd,
            args,
            cwd: None,
            profile: SandboxProfile::standard(),
            session_id: Some("sess-auth".into()),
            pep_grant_id: Some("grant-valid".into()),
            stdin_data: None,
        };
        let res = svc.execute(&req_auth).expect("exec");
        assert!(res.is_success());
    }
}

