//! Core service for enforcing Dependency & Toolchain Pinning (T-00323).
//!
//! Contract: `docs/tasks/evidence/T-00322-core-service-specification.md`.

use crate::toolchain_config::ToolchainManifest;

use std::time::{Duration, Instant};
use std::process::{Command, Stdio};

fn run_with_timeout(cmd: &mut Command, timeout_ms: u64) -> Result<std::process::Output, String> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    let start = Instant::now();
    
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed().as_millis() as u64 > timeout_ms {
                    let _ = child.kill(); // Try to cleanup the child
                    let _ = child.wait(); // Reap zombie
                    return Err(format!("process timed out after {} ms", timeout_ms));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("wait error: {e}"));
            }
        }
    }
    child.wait_with_output().map_err(|e| format!("failed to read output: {e}"))
}

/// Verifies that the host environment matches the provided ToolchainManifest.
///
/// Executes ecosystem-specific binaries (`rustc`, `python`, `node`) and
/// compares their version outputs to the pinned versions in the manifest.
pub fn enforce_toolchain(manifest: &ToolchainManifest) -> Result<(), String> {
    let timeout = 15000; // 15 seconds allows for cold start/rustup shim on Windows

    // 1. Check Rust
    let mut cmd = Command::new("rustc");
    cmd.arg("-V");
    let rustc_output = run_with_timeout(&mut cmd, timeout)
        .map_err(|e| format!("toolchain binary not found or failed: rustc ({e})"))?;
    if !rustc_output.status.success() {
        return Err("failed to execute rustc".into());
    }
    let rustc_str = String::from_utf8_lossy(&rustc_output.stdout);
    if !rustc_str.contains(&manifest.rust_version) {
        return Err(format!("toolchain mismatch: expected rustc {}, found {}", manifest.rust_version, rustc_str.trim()));
    }

    // 2. Check Python
    // We try `python3` first, then `python`.
    let mut cmd_py3 = Command::new("python3");
    cmd_py3.arg("-V");
    let python_output = run_with_timeout(&mut cmd_py3, timeout).or_else(|_| {
        let mut cmd_py = Command::new("python");
        cmd_py.arg("-V");
        run_with_timeout(&mut cmd_py, timeout)
    }).map_err(|e| format!("toolchain binary not found or failed: python/python3 ({e})"))?;
    
    if !python_output.status.success() {
        return Err("failed to execute python".into());
    }
    let python_str = String::from_utf8_lossy(&python_output.stdout);
    if !python_str.contains(&manifest.python_version) {
        return Err(format!("toolchain mismatch: expected python {}, found {}", manifest.python_version, python_str.trim()));
    }

    // 3. Check Node (Optional)
    if let Some(node_req) = &manifest.node_version {
        let mut cmd_node = Command::new("node");
        cmd_node.arg("-v");
        let node_output = run_with_timeout(&mut cmd_node, timeout)
            .map_err(|e| format!("toolchain binary not found or failed: node ({e})"))?;
        if !node_output.status.success() {
            return Err("failed to execute node".into());
        }
        let node_str = String::from_utf8_lossy(&node_output.stdout);
        if !node_str.contains(node_req) {
            return Err(format!("toolchain mismatch: expected node {}, found {}", node_req, node_str.trim()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn get_real_rustc() -> String {
        let out = Command::new("rustc").arg("-V").output().unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn get_real_python() -> String {
        let out = Command::new("python3").arg("-V").output().unwrap_or_else(|_| {
            Command::new("python").arg("-V").output().unwrap()
        });
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[test]
    fn test_enforce_toolchain_valid() {
        let manifest = ToolchainManifest {
            rust_version: get_real_rustc(),
            python_version: get_real_python(),
            node_version: None,
            enforce_hashes: true,
        };
        let result = enforce_toolchain(&manifest);
        assert!(result.is_ok(), "Valid toolchains should pass, got {:?}", result.err());
    }

    #[test]
    fn test_enforce_toolchain_mismatch_fails() {
        let manifest = ToolchainManifest {
            rust_version: "999.99.99".into(), // Will never match
            python_version: get_real_python(),
            node_version: None,
            enforce_hashes: true,
        };
        let result = enforce_toolchain(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected rustc 999.99.99"));
    }

    #[test]
    fn test_enforce_toolchain_python_mismatch_fails() {
        let manifest = ToolchainManifest {
            rust_version: get_real_rustc(),
            python_version: "Python 999.99.99".into(), // Will never match
            node_version: None,
            enforce_hashes: true,
        };
        let result = enforce_toolchain(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected python Python 999.99.99"));
    }

    #[test]
    fn test_enforce_toolchain_node_mismatch_fails() {
        let manifest = ToolchainManifest {
            rust_version: get_real_rustc(),
            python_version: get_real_python(),
            node_version: Some("v999.99.99".into()),
            enforce_hashes: true,
        };
        let result = enforce_toolchain(&manifest);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        // It could either fail because node is missing, or version mismatch
        assert!(err_msg.contains("expected node v999.99.99") || err_msg.contains("binary not found"));
    }

    #[test]
    fn test_check_toolchain_policy_enforcement() {
        // Read-only actions pass without a grant
        assert!(check_toolchain_policy(None, "aios.toolchain.check").is_ok());
        assert!(check_toolchain_policy(None, "aios.toolchain.config.get").is_ok());
        assert!(check_toolchain_policy(None, "toolchain.show").is_ok());
        assert!(check_toolchain_policy(None, "toolchain.check").is_ok());

        // Irreversible actions fail without a grant
        let err = check_toolchain_policy(None, "aios.toolchain.set");
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("requires an active PEP grant"));

        let err_empty = check_toolchain_policy(Some(""), "aios.toolchain.set");
        assert!(err_empty.is_err());

        // Irreversible actions pass with a valid grant token
        assert!(check_toolchain_policy(Some("gr_12345678"), "aios.toolchain.set").is_ok());
    }

    #[test]
    fn test_collect_toolchain_telemetry_captures_details() {
        let manifest = ToolchainManifest {
            rust_version: get_real_rustc(),
            python_version: get_real_python(),
            node_version: None,
            enforce_hashes: false,
        };
        let telemetry = collect_toolchain_telemetry(&manifest).unwrap();
        assert!(telemetry.detected_rust.is_some());
        assert!(telemetry.detected_python.is_some());
        assert!(telemetry.check_passed);
    }

    #[test]
    fn test_collect_toolchain_telemetry_negative_case() {
        let manifest = ToolchainManifest {
            rust_version: "999.99.99".into(),
            python_version: "3.14".into(),
            node_version: None,
            enforce_hashes: false,
        };
        let telemetry = collect_toolchain_telemetry(&manifest).unwrap();
        assert!(telemetry.detected_rust.is_some());
        assert!(!telemetry.check_passed);
    }

    #[test]
    fn test_validate_toolchain_manifest_happy_and_error() {
        let manifest = ToolchainManifest {
            rust_version: "1.99.0".into(),
            python_version: "3.14".into(),
            node_version: Some("v24.18".into()),
            enforce_hashes: false,
        };
        let temp_dir = std::env::temp_dir();
        let valid_path = temp_dir.join("test_val_manifest.json");
        std::fs::write(&valid_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let loaded = validate_toolchain_manifest(valid_path.to_str().unwrap());
        assert!(loaded.is_ok());
        let res = loaded.unwrap();
        assert_eq!(res.rust_version, "1.99.0");

        let missing = validate_toolchain_manifest("/path/that/does/not/exist_98765.json");
        assert!(missing.is_err());

        let corrupted_path = temp_dir.join("test_val_corrupted.json");
        std::fs::write(&corrupted_path, "{ broken json").unwrap();
        let corrupted = validate_toolchain_manifest(corrupted_path.to_str().unwrap());
        assert!(corrupted.is_err());
    }

    #[test]
    fn test_recover_default_toolchain() {
        let default_manifest = recover_default_toolchain();
        assert_eq!(default_manifest.rust_version, "1.99.0");
        assert_eq!(default_manifest.python_version, "3.14");
        assert_eq!(default_manifest.node_version, Some("v24.18".into()));
        assert!(!default_manifest.enforce_hashes);
    }

    #[test]
    fn test_reconcile_toolchain_report() {
        let conforming_manifest = ToolchainManifest {
            rust_version: get_real_rustc(),
            python_version: get_real_python(),
            node_version: None,
            enforce_hashes: false,
        };
        let report = reconcile_toolchain(&conforming_manifest).unwrap();
        assert_eq!(report.rust_status, "conforming");
        assert_eq!(report.python_status, "conforming");
        assert!(report.is_conforming);

        let drifted_manifest = ToolchainManifest {
            rust_version: "999.99.99".into(),
            python_version: "3.14".into(),
            node_version: None,
            enforce_hashes: false,
        };
        let drifted_report = reconcile_toolchain(&drifted_manifest).unwrap();
        assert_eq!(drifted_report.rust_status, "drifted");
        assert!(!drifted_report.is_conforming);
        assert!(!drifted_report.remediation_steps.is_empty());
    }
}

/// Checks security policy for toolchain operations (T-00374).
pub fn check_toolchain_policy(grant: Option<&str>, action: &str) -> Result<(), String> {
    if crate::pep::is_irreversible(action) {
        match grant {
            Some(g) if !g.is_empty() => Ok(()),
            _ => Err(format!(
                "Action '{}' is irreversible and requires an active PEP grant",
                action
            )),
        }
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolchainTelemetry {
    pub manifest: ToolchainManifest,
    pub detected_rust: Option<String>,
    pub detected_python: Option<String>,
    pub detected_node: Option<String>,
    pub check_passed: bool,
}

fn clamp_str(s: &str, max_len: usize) -> String {
    let trimmed = s.trim();
    if trimmed.len() > max_len {
        format!("{}...[TRUNCATED]", &trimmed[..max_len])
    } else {
        trimmed.to_string()
    }
}

/// Collects runtime toolchain telemetry (T-00384 / T-00388).
pub fn collect_toolchain_telemetry(manifest: &ToolchainManifest) -> Result<ToolchainTelemetry, String> {
    let timeout = 15000;
    let mut detected_rust = None;
    let mut detected_python = None;
    let mut detected_node = None;

    let mut cmd_rust = Command::new("rustc");
    cmd_rust.arg("-V");
    if let Ok(out) = run_with_timeout(&mut cmd_rust, timeout) {
        if out.status.success() {
            detected_rust = Some(clamp_str(&String::from_utf8_lossy(&out.stdout), 512));
        }
    }

    let mut cmd_py3 = Command::new("python3");
    cmd_py3.arg("-V");
    if let Ok(out) = run_with_timeout(&mut cmd_py3, timeout) {
        if out.status.success() {
            detected_python = Some(clamp_str(&String::from_utf8_lossy(&out.stdout), 512));
        }
    } else {
        let mut cmd_py = Command::new("python");
        cmd_py.arg("-V");
        if let Ok(out) = run_with_timeout(&mut cmd_py, timeout) {
            if out.status.success() {
                detected_python = Some(clamp_str(&String::from_utf8_lossy(&out.stdout), 512));
            }
        }
    }

    let mut cmd_node = Command::new("node");
    cmd_node.arg("-v");
    if let Ok(out) = run_with_timeout(&mut cmd_node, timeout) {
        if out.status.success() {
            detected_node = Some(clamp_str(&String::from_utf8_lossy(&out.stdout), 512));
        }
    }

    let check_passed = enforce_toolchain(manifest).is_ok();

    Ok(ToolchainTelemetry {
        manifest: manifest.clone(),
        detected_rust,
        detected_python,
        detected_node,
        check_passed,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolchainReconciliationReport {
    pub is_conforming: bool,
    pub rust_status: String,
    pub python_status: String,
    pub node_status: String,
    pub remediation_steps: Vec<String>,
}

/// Validates toolchain manifest syntax and schema without executing compiler binaries (T-00404).
pub fn validate_toolchain_manifest(path: &str) -> Result<ToolchainManifest, String> {
    ToolchainManifest::from_path(path)
}

/// Recovers in-memory canonical default toolchain configuration (T-00404).
pub fn recover_default_toolchain() -> ToolchainManifest {
    ToolchainManifest::default()
}

/// Reconciles host toolchain against desired manifest, providing drift remediation steps (T-00404).
pub fn reconcile_toolchain(manifest: &ToolchainManifest) -> Result<ToolchainReconciliationReport, String> {
    let telemetry = collect_toolchain_telemetry(manifest)?;
    let mut remediation_steps = Vec::new();
    
    let rust_status = match &telemetry.detected_rust {
        Some(v) if v.contains(&manifest.rust_version) => "conforming".to_string(),
        Some(v) => {
            remediation_steps.push(format!(
                "Rust mismatch: detected '{}', expected '{}'. Run 'rustup default {}'",
                v, manifest.rust_version, manifest.rust_version
            ));
            "drifted".to_string()
        }
        None => {
            remediation_steps.push(format!(
                "Rust binary not found. Install rustup and toolchain {}",
                manifest.rust_version
            ));
            "missing".to_string()
        }
    };

    let python_status = match &telemetry.detected_python {
        Some(v) if v.contains(&manifest.python_version) => "conforming".to_string(),
        Some(v) => {
            remediation_steps.push(format!(
                "Python mismatch: detected '{}', expected '{}'. Configure virtualenv with Python {}",
                v, manifest.python_version, manifest.python_version
            ));
            "drifted".to_string()
        }
        None => {
            remediation_steps.push(format!(
                "Python binary not found. Install Python {}",
                manifest.python_version
            ));
            "missing".to_string()
        }
    };

    let node_status = match (&manifest.node_version, &telemetry.detected_node) {
        (None, _) => "unconstrained".to_string(),
        (Some(req), Some(v)) if v.contains(req) => "conforming".to_string(),
        (Some(req), Some(v)) => {
            remediation_steps.push(format!(
                "Node mismatch: detected '{}', expected '{}'. Run 'nvm use {}'",
                v, req, req
            ));
            "drifted".to_string()
        }
        (Some(req), None) => {
            remediation_steps.push(format!("Node binary not found. Install Node {}", req));
            "missing".to_string()
        }
    };

    let is_conforming = telemetry.check_passed;

    Ok(ToolchainReconciliationReport {
        is_conforming,
        rust_status,
        python_status,
        node_status,
        remediation_steps,
    })
}





