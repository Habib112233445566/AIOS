//! Unit Tests for Secrets Handling Data Model (T-02605).

use aiosh_core::secret_data_model::*;

#[test]
fn test_secret_kind_parsing_and_display() {
    assert_eq!(SecretKind::parse_kind("api_key"), Some(SecretKind::ApiKey));
    assert_eq!(SecretKind::parse_kind("token"), Some(SecretKind::OAuthToken));
    assert_eq!(SecretKind::parse_kind("db"), Some(SecretKind::DatabaseCredential));
    assert_eq!(SecretKind::parse_kind("privatekey"), Some(SecretKind::PrivateKey));
    assert_eq!(SecretKind::parse_kind("cert"), Some(SecretKind::TlsCertificate));
    assert_eq!(SecretKind::parse_kind("symmetrickey"), Some(SecretKind::SymmetricKey));
    assert_eq!(SecretKind::parse_kind("generic"), Some(SecretKind::Generic));
    assert_eq!(SecretKind::parse_kind("nonexistent_kind"), None);

    assert_eq!(SecretKind::ApiKey.as_str(), "api_key");
    assert_eq!(format!("{}", SecretKind::TlsCertificate), "tls_certificate");
}

#[test]
fn test_secret_scope_hierarchy_and_parsing() {
    let global = SecretScope::parse_scope("global", None).unwrap();
    assert_eq!(global, SecretScope::Global);

    let env = SecretScope::parse_scope("env", Some("production")).unwrap();
    assert_eq!(env, SecretScope::Environment("production".into()));

    let actor = SecretScope::parse_scope("actor", Some("agent_007")).unwrap();
    assert_eq!(actor, SecretScope::Actor("agent_007".into()));

    let session = SecretScope::parse_scope("session", Some("sess_xyz")).unwrap();
    assert_eq!(session, SecretScope::Session("sess_xyz".into()));

    // Target missing errors
    assert!(SecretScope::parse_scope("env", None).is_err());
    assert!(SecretScope::parse_scope("actor", None).is_err());
    assert!(SecretScope::parse_scope("session", None).is_err());

    // Access hierarchy
    assert!(global.allows(&env));
    assert!(global.allows(&actor));
    assert!(global.allows(&session));
    assert!(env.allows(&SecretScope::Environment("production".into())));
    assert!(!env.allows(&SecretScope::Environment("staging".into())));
    assert!(!env.allows(&actor));
}

#[test]
fn test_secret_state_transitions() {
    let s = SecretState::Active;
    assert!(s.is_accessible());

    // Active -> Rotated
    let s2 = s.transition_to(SecretState::Rotated).unwrap();
    assert_eq!(s2, SecretState::Rotated);
    assert!(s2.is_accessible());

    // Rotated -> Revoked
    let s3 = s2.transition_to(SecretState::Revoked).unwrap();
    assert_eq!(s3, SecretState::Revoked);
    assert!(!s3.is_accessible());

    // Revoked is terminal
    assert!(s3.transition_to(SecretState::Active).is_err());
    assert!(s3.transition_to(SecretState::Rotated).is_err());

    // Active -> Expired
    let s_exp = s.transition_to(SecretState::Expired).unwrap();
    assert_eq!(s_exp, SecretState::Expired);
    assert!(!s_exp.is_accessible());

    // Expired is terminal
    assert!(s_exp.transition_to(SecretState::Active).is_err());
}

#[test]
fn test_secret_metadata_validation() {
    // Valid metadata
    let mut meta = SecretMetadata::new(
        "sec_openai_prod",
        "OpenAI API Key",
        SecretKind::ApiKey,
        SecretScope::Global,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ).unwrap();
    assert!(meta.validate().is_ok());

    // Empty ID
    meta.id = "".into();
    assert!(meta.validate().is_err());

    // ID too long
    meta.id = "a".repeat(MAX_SECRET_ID_LEN + 1);
    assert!(meta.validate().is_err());

    // Invalid ID characters
    meta.id = "sec$invalid!".into();
    assert!(meta.validate().is_err());

    // Empty name
    meta.id = "valid_id".into();
    meta.name = "".into();
    assert!(meta.validate().is_err());

    // Name with control char
    meta.name = "Bad\nName".into();
    assert!(meta.validate().is_err());
}

#[test]
fn test_secret_value_bounds_and_fingerprint() {
    // Exceeding 64 KiB
    let huge = vec![0x42u8; MAX_SECRET_PAYLOAD_SIZE + 1];
    assert!(SecretValue::new(&huge).is_err());

    // Valid secret
    let val = SecretValue::from_str_slice("test_secret_payload_12345").unwrap();
    assert_eq!(val.as_bytes(), b"test_secret_payload_12345");
    assert_eq!(val.as_str().unwrap(), "test_secret_payload_12345");

    let fp = val.compute_fingerprint();
    assert_eq!(fp.len(), 64); // SHA-256 hex string
}

#[test]
fn test_secret_value_constant_time_eq_and_masking() {
    let v1 = SecretValue::from_str_slice("super_secret_token_123").unwrap();
    let v2 = SecretValue::from_str_slice("super_secret_token_123").unwrap();
    let v3 = SecretValue::from_str_slice("different_token_123456").unwrap();

    assert!(v1.constant_time_eq(&v2));
    assert!(!v1.constant_time_eq(&v3));

    // Masking long secret (>= 12 chars)
    let masked = v1.masked_display();
    assert_eq!(masked, "supe..._123");

    // Masking short secret (< 12 chars)
    let short = SecretValue::from_str_slice("short").unwrap();
    assert_eq!(short.masked_display(), "********");
}

#[test]
fn test_secret_entry_lifecycle() {
    let mut entry = SecretEntry::new(
        "db_pass_main",
        "Primary DB Password",
        SecretKind::DatabaseCredential,
        SecretScope::Environment("production".into()),
        b"initial_password_123",
    ).unwrap();

    assert_eq!(entry.metadata.version, 1);
    assert_eq!(entry.metadata.state, SecretState::Active);
    let initial_fp = entry.metadata.fingerprint.clone();

    // Rotate secret
    entry.rotate(b"new_rotated_password_456").unwrap();
    assert_eq!(entry.metadata.version, 2);
    assert_eq!(entry.metadata.state, SecretState::Rotated);
    assert_ne!(entry.metadata.fingerprint, initial_fp);
    assert_eq!(entry.value.as_bytes(), b"new_rotated_password_456");

    // Revoke secret
    entry.revoke().unwrap();
    assert_eq!(entry.metadata.state, SecretState::Revoked);
    assert_eq!(entry.value.as_bytes(), b"");
}

#[test]
fn test_secret_metadata_label_bounds() {
    let mut meta = SecretMetadata::new(
        "sec_label_test",
        "Label Test Secret",
        SecretKind::Generic,
        SecretScope::Global,
        "dummy_fingerprint",
    ).unwrap();

    // 1. Label count limit (> 32)
    for i in 0..33 {
        meta.labels.insert(format!("key_{}", i), "val".into());
    }
    assert!(meta.validate().is_err());

    meta.labels.clear();

    // 2. Label key too long (> 64)
    meta.labels.insert("k".repeat(65), "val".into());
    assert!(meta.validate().is_err());

    meta.labels.clear();

    // 3. Label val too long (> 256)
    meta.labels.insert("valid_key".into(), "v".repeat(257));
    assert!(meta.validate().is_err());

    meta.labels.clear();

    // 4. Valid labels
    meta.labels.insert("valid_key".into(), "valid_val".into());
    assert!(meta.validate().is_ok());
}

