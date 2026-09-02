//! Secrets scanning and ingestion service (T-00721..T-00730).
//!
//! Scans workspace files for private keys, AWS credentials, API tokens,
//! and embedded configuration secrets without leaking unredacted values.

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::canonical::sha256_hex;
use crate::secrets::{redact_secret_value, SecretFinding, SecretPatternKind, SecretScanReport, SecretSeverity};

/// Default maximum file size scanned for secrets (16 MiB).
pub const DEFAULT_MAX_SECRET_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Default line length limit to protect against minified ReDoS/spikes (4096 bytes).
pub const MAX_LINE_SCAN_LENGTH: usize = 4096;

/// Standard ignored directories.
pub const DEFAULT_IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".venv", "dist"];

/// Scan a single file for secret patterns.
pub fn scan_file_for_secrets(
    path: &Path,
    base_dir: &Path,
    max_file_bytes: u64,
) -> Result<Vec<SecretFinding>, String> {
    if !path.is_file() {
        return Ok(Vec::new());
    }

    let meta = match path.metadata() {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to read file metadata {}: {}", path.display(), e)),
    };

    if meta.len() > max_file_bytes {
        return Ok(Vec::new());
    }

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to open file {}: {}", path.display(), e)),
    };

    // Check first 512 bytes for binary null byte indicator
    let mut header = [0u8; 512];
    let n = file.read(&mut header).unwrap_or(0);
    if header[..n].contains(&0) {
        return Ok(Vec::new());
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("Failed to re-open file {}: {}", path.display(), e)),
    };

    let rel_path = path.strip_prefix(base_dir).unwrap_or(path).to_string_lossy().to_string();
    let reader = BufReader::new(file);
    let mut findings = Vec::new();

    for (idx, line_res) in reader.lines().enumerate() {
        let line_number = idx + 1;
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue, // Skip unreadable/non-UTF8 lines
        };

        let scan_text = if line.len() > MAX_LINE_SCAN_LENGTH {
            &line[..MAX_LINE_SCAN_LENGTH]
        } else {
            &line
        };

        // Pattern 1: Private Key
        if scan_text.contains("-----BEGIN") && scan_text.contains("PRIVATE KEY-----") {
            let snippet = redact_secret_value(scan_text);
            let fp = sha256_hex(&format!("SEC-001:{}:{}:{}", rel_path, line_number, snippet));
            findings.push(SecretFinding {
                rule_id: "SEC-001".into(),
                path: rel_path.clone(),
                line_number,
                severity: SecretSeverity::Critical,
                pattern_kind: SecretPatternKind::PrivateKey,
                description: "Private Key block detected".into(),
                redacted_snippet: snippet,
                fingerprint: fp,
            });
        }

        // Pattern 2: AWS Access Key ID
        if let Some(pos) = scan_text.find("AKIA") {
            if scan_text.len() >= pos + 20 {
                let candidate = &scan_text[pos..pos + 20];
                if candidate.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
                    let snippet = redact_secret_value(candidate);
                    let fp = sha256_hex(&format!("SEC-002:{}:{}:{}", rel_path, line_number, snippet));
                    findings.push(SecretFinding {
                        rule_id: "SEC-002".into(),
                        path: rel_path.clone(),
                        line_number,
                        severity: SecretSeverity::Critical,
                        pattern_kind: SecretPatternKind::AwsCredentials,
                        description: "AWS Access Key ID detected".into(),
                        redacted_snippet: snippet,
                        fingerprint: fp,
                    });
                }
            }
        }

        // Pattern 3: GitHub PAT
        if let Some(pos) = scan_text.find("ghp_") {
            if scan_text.len() >= pos + 40 {
                let candidate = &scan_text[pos..pos + 40];
                if candidate.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    let snippet = redact_secret_value(candidate);
                    let fp = sha256_hex(&format!("SEC-003:{}:{}:{}", rel_path, line_number, snippet));
                    findings.push(SecretFinding {
                        rule_id: "SEC-003".into(),
                        path: rel_path.clone(),
                        line_number,
                        severity: SecretSeverity::High,
                        pattern_kind: SecretPatternKind::ApiToken,
                        description: "GitHub Personal Access Token detected".into(),
                        redacted_snippet: snippet,
                        fingerprint: fp,
                    });
                }
            }
        }

        // Pattern 4: Generic API / Bearer Token
        let lower = scan_text.to_ascii_lowercase();
        if lower.contains("api_key") || lower.contains("bearer_token") || lower.contains("secret_token") {
            if let Some(eq_pos) = scan_text.find('=') {
                let val = scan_text[eq_pos + 1..].trim().trim_matches(|c| c == '"' || c == '\'');
                if val.len() >= 20 && !val.contains(' ') {
                    let snippet = redact_secret_value(val);
                    let fp = sha256_hex(&format!("SEC-004:{}:{}:{}", rel_path, line_number, snippet));
                    findings.push(SecretFinding {
                        rule_id: "SEC-004".into(),
                        path: rel_path.clone(),
                        line_number,
                        severity: SecretSeverity::High,
                        pattern_kind: SecretPatternKind::ApiToken,
                        description: "Generic API/Bearer token assignment detected".into(),
                        redacted_snippet: snippet,
                        fingerprint: fp,
                    });
                }
            }
        }

        // Pattern 5: Password in Config / .env
        if lower.contains("password") || lower.contains("db_pass") || lower.contains("secret_key") {
            if let Some(eq_pos) = scan_text.find('=') {
                let val = scan_text[eq_pos + 1..].trim().trim_matches(|c| c == '"' || c == '\'');
                if val.len() >= 8 && !val.contains(' ') && val != "test" && val != "example" {
                    let snippet = redact_secret_value(val);
                    let fp = sha256_hex(&format!("SEC-005:{}:{}:{}", rel_path, line_number, snippet));
                    findings.push(SecretFinding {
                        rule_id: "SEC-005".into(),
                        path: rel_path.clone(),
                        line_number,
                        severity: SecretSeverity::High,
                        pattern_kind: SecretPatternKind::PasswordInConfig,
                        description: "Hardcoded password/key assignment detected".into(),
                        redacted_snippet: snippet,
                        fingerprint: fp,
                    });
                }
            }
        }
    }

    Ok(findings)
}

/// Recursively scan workspace directories for secrets.
pub fn scan_workspace_for_secrets(
    root: &Path,
    max_file_bytes: u64,
    ignored_dirs: &[&str],
) -> Result<SecretScanReport, String> {
    if !root.is_dir() {
        return Err(format!("Workspace root does not exist: {}", root.display()));
    }

    let mut scanned_files_count = 0;
    let mut all_findings = Vec::new();

    fn walk_dir(
        dir: &Path,
        root: &Path,
        max_bytes: u64,
        ignored_dirs: &[&str],
        scanned_count: &mut u32,
        findings: &mut Vec<SecretFinding>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if path.is_dir() {
                if ignored_dirs.iter().any(|&d| d == name_str) {
                    continue;
                }
                walk_dir(&path, root, max_bytes, ignored_dirs, scanned_count, findings)?;
            } else if path.is_file() {
                *scanned_count += 1;
                if let Ok(file_findings) = scan_file_for_secrets(&path, root, max_bytes) {
                    findings.extend(file_findings);
                }
            }
        }
        Ok(())
    }

    if let Err(e) = walk_dir(root, root, max_file_bytes, ignored_dirs, &mut scanned_files_count, &mut all_findings) {
        return Err(format!("Failed during workspace traversal: {}", e));
    }

    Ok(SecretScanReport::new(root.to_string_lossy().to_string(), all_findings, scanned_files_count))
}

/// Scan a workspace directory tree using configuration specified in `SecretsConfig`.
pub fn scan_workspace_with_config(
    root: &Path,
    config: &crate::secrets_config::SecretsConfig,
) -> Result<SecretScanReport, String> {
    config.validate()?;
    let ignored_slice: Vec<&str> = config.ignored_dirs.iter().map(|s| s.as_str()).collect();
    scan_workspace_for_secrets(root, config.max_file_bytes, &ignored_slice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_scan_file_clean() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("clean.rs");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "// Just normal rust code\nfn main() {{ println!(\"hello\"); }}").unwrap();

        let findings = scan_file_for_secrets(&file_path, dir.path(), DEFAULT_MAX_SECRET_FILE_BYTES).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_file_private_key() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("id_rsa");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...\n-----END RSA PRIVATE KEY-----").unwrap();

        let findings = scan_file_for_secrets(&file_path, dir.path(), DEFAULT_MAX_SECRET_FILE_BYTES).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "SEC-001");
        assert_eq!(findings[0].severity, SecretSeverity::Critical);
        assert!(!findings[0].fingerprint.is_empty());
    }

    #[test]
    fn test_scan_file_aws_key_and_ghp() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(".env");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "AWS_KEY=AKIA1234567890ABCDEF\nGITHUB_TOKEN=ghp_123456789012345678901234567890123456").unwrap();

        let findings = scan_file_for_secrets(&file_path, dir.path(), DEFAULT_MAX_SECRET_FILE_BYTES).unwrap();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule_id, "SEC-002");
        assert_eq!(findings[1].rule_id, "SEC-003");
    }

    #[test]
    fn test_scan_workspace() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("src");
        std::fs::create_dir(&sub).unwrap();
        let file_path = sub.join("secrets.txt");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "api_key=\"abcdef1234567890abcdef1234567890\"").unwrap();

        let report = scan_workspace_for_secrets(dir.path(), DEFAULT_MAX_SECRET_FILE_BYTES, DEFAULT_IGNORED_DIRS).unwrap();
        assert!(!report.is_clean);
        assert_eq!(report.total_findings, 1);
        assert_eq!(report.findings[0].rule_id, "SEC-004");
    }

    #[test]
    fn test_scan_file_password_in_config() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("database.conf");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "db_pass = \"SuperSecretP@ssword123!\"").unwrap();

        let findings = scan_file_for_secrets(&file_path, dir.path(), DEFAULT_MAX_SECRET_FILE_BYTES).unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "SEC-005");
        assert_eq!(findings[0].severity, SecretSeverity::High);
    }

    #[test]
    fn test_scan_binary_file_skipped() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("blob.bin");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(&[0x00, 0x01, 0x02, 0x00, 0x41, 0x4b, 0x49, 0x41]).unwrap(); // Contains null bytes

        let findings = scan_file_for_secrets(&file_path, dir.path(), DEFAULT_MAX_SECRET_FILE_BYTES).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_workspace_nonexistent() {
        let non_existent = Path::new("/path/that/does/not/exist/for/sure/aios");
        let res = scan_workspace_for_secrets(non_existent, DEFAULT_MAX_SECRET_FILE_BYTES, DEFAULT_IGNORED_DIRS);
        assert!(res.is_err());
    }

    #[test]
    fn test_scan_workspace_with_config_execution() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "let token = \"ghp_123456789012345678901234567890123456\";").unwrap();

        let config = crate::secrets_config::SecretsConfig::default();
        let report = scan_workspace_with_config(dir.path(), &config).unwrap();
        assert!(!report.is_clean);
        assert_eq!(report.total_findings, 1);
        assert_eq!(report.findings[0].rule_id, "SEC-003");
    }
}
