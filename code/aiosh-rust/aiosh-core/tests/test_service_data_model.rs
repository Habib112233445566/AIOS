//! Automated Unit & Integration Tests for Init & Service Supervision Data Model (SS1..SS5)

use aiosh_core::service::{
    validate_service_name, validate_service_spec, validate_service_status, ServiceAction,
    ServiceDependency, ServiceDependencyType, ServiceHealth, ServiceQuery, ServiceRestartPolicy,
    ServiceSpec, ServiceStartupMode, ServiceState, ServiceStatus, ServiceType,
};
use std::collections::BTreeMap;

fn base_valid_spec() -> ServiceSpec {
    let mut env = BTreeMap::new();
    env.insert("AIOS_ENV".to_string(), "production".to_string());
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    ServiceSpec {
        name: "aios-securityd.service".to_string(),
        description: "AIOS Security Daemon supervising platform PEP and audit ring".to_string(),
        exec_start: "/usr/bin/aios-securityd --daemon --config /etc/aios/security.json".to_string(),
        exec_stop: Some("/usr/bin/aios-securityd --stop".to_string()),
        exec_reload: Some("/usr/bin/aios-securityd --reload".to_string()),
        service_type: ServiceType::Simple,
        restart_policy: ServiceRestartPolicy::Always,
        startup_mode: ServiceStartupMode::Enabled,
        user: Some("aios".to_string()),
        group: Some("aios".to_string()),
        working_dir: Some("/var/lib/aios".to_string()),
        environment: env,
        dependencies: vec![ServiceDependency {
            name: "auditd.service".to_string(),
            dependency_type: ServiceDependencyType::Requires,
            optional: false,
        }],
        timeout_start_secs: 30,
        timeout_stop_secs: 30,
    }
}

#[test]
fn test_ss1_service_name_boundary_and_syntax() {
    // Valid standard service names
    assert!(validate_service_name("aios-securityd").is_ok());
    assert!(validate_service_name("aios-securityd.service").is_ok());
    assert!(validate_service_name("dbus").is_ok());
    assert!(validate_service_name("systemd-journald.socket").is_ok());
    assert!(validate_service_name("auditd.service").is_ok());
    assert!(validate_service_name("sshd_config").is_ok());
    assert!(validate_service_name("network-manager").is_ok());

    // Boundary: min length (1 char)
    assert!(validate_service_name("a").is_ok());
    assert!(validate_service_name("1").is_ok());

    // Boundary: max length (128 chars)
    let max_len_name = "a".repeat(128);
    assert!(validate_service_name(&max_len_name).is_ok());

    // Negative: oversized (129 chars)
    let over_len_name = "a".repeat(129);
    assert!(validate_service_name(&over_len_name).is_err());

    // Negative: empty
    assert!(validate_service_name("").is_err());

    // Negative: leading non-alphanumeric symbols
    assert!(validate_service_name("-service").is_err());
    assert!(validate_service_name(".service").is_err());
    assert!(validate_service_name("_service").is_err());

    // Negative: whitespace
    assert!(validate_service_name(" aios").is_err());
    assert!(validate_service_name("aios ").is_err());
    assert!(validate_service_name("aios service").is_err());
    assert!(validate_service_name("aios\tservice").is_err());

    // Negative: path separators
    assert!(validate_service_name("aios/securityd").is_err());
    assert!(validate_service_name("aios\\securityd").is_err());

    // Negative: shell metacharacters and null bytes
    assert!(validate_service_name("aios;rm").is_err());
    assert!(validate_service_name("aios&service").is_err());
    assert!(validate_service_name("aios|service").is_err());
    assert!(validate_service_name("aios>service").is_err());
    assert!(validate_service_name("aios<service").is_err());
    assert!(validate_service_name("aios$service").is_err());
    assert!(validate_service_name("aios\0service").is_err());
}

#[test]
fn test_ss2_exec_commands_and_working_dir() {
    // Happy path
    let spec = base_valid_spec();
    assert!(validate_service_spec(&spec).is_ok());

    // Boundary: exec_start 4096 chars is ok
    let mut spec = base_valid_spec();
    spec.exec_start = format!("/usr/bin/aios {}", "x".repeat(4096 - 14));
    assert_eq!(spec.exec_start.len(), 4096);
    assert!(validate_service_spec(&spec).is_ok());

    // Negative: empty exec_start
    let mut spec = base_valid_spec();
    spec.exec_start = String::new();
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("exec_start cannot be empty")));

    // Negative: exec_start > 4096 chars
    let mut spec = base_valid_spec();
    spec.exec_start = "a".repeat(4097);
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("exec_start exceeds 4096")));

    // Negative: empty exec_stop when specified
    let mut spec = base_valid_spec();
    spec.exec_stop = Some(String::new());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("exec_stop cannot be empty")));

    // Negative: empty exec_reload when specified
    let mut spec = base_valid_spec();
    spec.exec_reload = Some(String::new());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("exec_reload cannot be empty")));

    // Working directory: Unix absolute path ok
    let mut spec = base_valid_spec();
    spec.working_dir = Some("/opt/aios/app".to_string());
    assert!(validate_service_spec(&spec).is_ok());

    // Working directory: Windows absolute path ok
    let mut spec = base_valid_spec();
    spec.working_dir = Some("C:\\aios\\data".to_string());
    assert!(validate_service_spec(&spec).is_ok());

    // Negative: relative working directory
    let mut spec = base_valid_spec();
    spec.working_dir = Some("relative/path".to_string());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("must be an absolute path")));

    // Negative: path traversal sequence '..'
    let mut spec = base_valid_spec();
    spec.working_dir = Some("/var/lib/../etc".to_string());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("path traversal sequence '..'")));
}

#[test]
fn test_ss3_dependency_hygiene() {
    // Happy path: diverse dependency types
    let mut spec = base_valid_spec();
    spec.dependencies = vec![
        ServiceDependency {
            name: "network.target".to_string(),
            dependency_type: ServiceDependencyType::After,
            optional: false,
        },
        ServiceDependency {
            name: "syslog.service".to_string(),
            dependency_type: ServiceDependencyType::Wants,
            optional: true,
        },
        ServiceDependency {
            name: "legacy-security.service".to_string(),
            dependency_type: ServiceDependencyType::Conflicts,
            optional: false,
        },
    ];
    assert!(validate_service_spec(&spec).is_ok());

    // Negative: self-dependency
    let mut spec = base_valid_spec();
    spec.dependencies.push(ServiceDependency {
        name: "aios-securityd.service".to_string(),
        dependency_type: ServiceDependencyType::Requires,
        optional: false,
    });
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("service cannot depend on itself")));

    // Negative: duplicate dependency
    let mut spec = base_valid_spec();
    spec.dependencies.push(ServiceDependency {
        name: "auditd.service".to_string(),
        dependency_type: ServiceDependencyType::Wants,
        optional: true,
    });
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("duplicate dependency detected")));

    // Negative: invalid dependency name syntax
    let mut spec = base_valid_spec();
    spec.dependencies.push(ServiceDependency {
        name: "invalid/dep".to_string(),
        dependency_type: ServiceDependencyType::Requires,
        optional: false,
    });
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("invalid dependency name")));

    // Boundary: 128 dependencies ok
    let mut spec = base_valid_spec();
    spec.dependencies.clear();
    for i in 0..128 {
        spec.dependencies.push(ServiceDependency {
            name: format!("dep-{}.service", i),
            dependency_type: ServiceDependencyType::After,
            optional: false,
        });
    }
    assert!(validate_service_spec(&spec).is_ok());

    // Negative: > 128 dependencies
    spec.dependencies.push(ServiceDependency {
        name: "dep-128.service".to_string(),
        dependency_type: ServiceDependencyType::After,
        optional: false,
    });
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("dependencies list exceeds 128")));
}

#[test]
fn test_ss4_resource_and_field_limits() {
    // Boundary: timeout min = 1s
    let mut spec = base_valid_spec();
    spec.timeout_start_secs = 1;
    spec.timeout_stop_secs = 1;
    assert!(validate_service_spec(&spec).is_ok());

    // Boundary: timeout max = 86400s (24h)
    let mut spec = base_valid_spec();
    spec.timeout_start_secs = 86400;
    spec.timeout_stop_secs = 86400;
    assert!(validate_service_spec(&spec).is_ok());

    // Negative: timeout = 0
    let mut spec = base_valid_spec();
    spec.timeout_start_secs = 0;
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("timeout_start_secs out of range")));

    // Negative: timeout > 86400
    let mut spec = base_valid_spec();
    spec.timeout_stop_secs = 86401;
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("timeout_stop_secs out of range")));

    // Description: 4096 ok, 4097 error
    let mut spec = base_valid_spec();
    spec.description = "d".repeat(4096);
    assert!(validate_service_spec(&spec).is_ok());

    let mut spec = base_valid_spec();
    spec.description = "d".repeat(4097);
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("description exceeds 4096")));

    // Environment: empty key error
    let mut spec = base_valid_spec();
    spec.environment.insert(String::new(), "val".to_string());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("environment variable key cannot be empty")));

    // Environment: key containing '='
    let mut spec = base_valid_spec();
    spec.environment.insert("FOO=BAR".to_string(), "val".to_string());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("contains illegal characters")));

    // User/Group validation: valid Unix names
    let mut spec = base_valid_spec();
    spec.user = Some("root".to_string());
    spec.group = Some("wheel".to_string());
    assert!(validate_service_spec(&spec).is_ok());

    // User name with invalid characters
    let mut spec = base_valid_spec();
    spec.user = Some("user@domain".to_string());
    let errs = validate_service_spec(&spec).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("user name contains invalid characters")));
}

#[test]
fn test_ss5_service_status_and_lifecycle_consistency() {
    let status = ServiceStatus {
        name: "aios-securityd.service".to_string(),
        state: ServiceState::Active,
        startup_mode: ServiceStartupMode::Enabled,
        pid: Some(4242),
        health: ServiceHealth {
            healthy: true,
            exit_code: None,
            pid: Some(4242),
            uptime_seconds: Some(7200),
            restarts: 0,
            last_error: None,
        },
        started_at: Some("2026-09-05T00:00:00Z".to_string()),
    };
    assert!(validate_service_status(&status).is_ok());

    // Inactive stopped service ok
    let mut stopped = status.clone();
    stopped.state = ServiceState::Inactive;
    stopped.pid = None;
    stopped.health.healthy = false;
    assert!(validate_service_status(&stopped).is_ok());

    // Negative: Failed state but reports healthy == true
    let mut bad_status = status.clone();
    bad_status.state = ServiceState::Failed;
    bad_status.health.healthy = true;
    let errs = validate_service_status(&bad_status).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("service state is 'failed' but health reports healthy")));

    // Negative: Masked service reports active state
    let mut masked_status = status.clone();
    masked_status.startup_mode = ServiceStartupMode::Masked;
    masked_status.state = ServiceState::Active;
    let errs = validate_service_status(&masked_status).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("service is masked but reports state == 'active'")));

    // Negative: Invalid service name in status
    let mut bad_name = status.clone();
    bad_name.name = "invalid/service".to_string();
    let errs = validate_service_status(&bad_name).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("service name contains invalid character")));
}

#[test]
fn test_service_data_model_serde_roundtrip() {
    let spec = base_valid_spec();
    let json_spec = serde_json::to_string_pretty(&spec).expect("serialize spec");
    let deserialized_spec: ServiceSpec =
        serde_json::from_str(&json_spec).expect("deserialize spec");
    assert_eq!(spec, deserialized_spec);

    let query = ServiceQuery {
        name_pattern: Some("aios-*".to_string()),
        state: Some(ServiceState::Active),
        startup_mode: Some(ServiceStartupMode::Enabled),
        limit: Some(50),
    };
    let json_query = serde_json::to_string(&query).expect("serialize query");
    let deserialized_query: ServiceQuery =
        serde_json::from_str(&json_query).expect("deserialize query");
    assert_eq!(query, deserialized_query);

    let action = ServiceAction::Restart;
    let json_action = serde_json::to_string(&action).expect("serialize action");
    assert_eq!(json_action, "\"restart\"");
}
