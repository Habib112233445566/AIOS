use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultRecord {
    pub suite: String,
    pub index: usize,
    pub status: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub started_at: String,
    pub finished_at: String,
    pub log_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunSummary {
    pub tool: String,
    pub schema_version: u32,
    pub started_at: String,
    pub finished_at: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub all_pass: bool,
    pub results: Vec<ResultRecord>,
}

pub fn load_summary_with_retry(path: &Path, retries: usize) -> Result<RunSummary, String> {
    // Hardening: Bounded retries where external files are involved
    let mut attempts = 0;
    loop {
        attempts += 1;
        // Hardening: Size cap
        match fs::metadata(path) {
            Ok(meta) => {
                if meta.len() > 1024 * 1024 {
                    return Err(format!("summary file {} exceeds 1MB size cap", path.display()));
                }
                break;
            }
            Err(e) => {
                if attempts > retries {
                    return Err(format!("could not stat {}: {}", path.display(), e));
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }

    let content = fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;
    
    // Strict JSON parsing
    let summary: RunSummary = serde_json::from_str(&content).map_err(|e| format!("parse error: {}", e))?;

    // Hardening: Explicit validation (no silent failures)
    if summary.schema_version != 1 {
        return Err(format!("schema_version is {}, expected 1", summary.schema_version));
    }
    if summary.passed + summary.failed != summary.total {
        return Err(format!("arithmetic incoherence: {} + {} != {}", summary.passed, summary.failed, summary.total));
    }

    Ok(summary)
}

pub fn human_report(summary: &RunSummary) -> String {
    let verdict = if summary.all_pass { "PASS" } else { "FAIL" };
    let mut lines = vec![
        format!("CI run {} .. {}: {}", summary.started_at, summary.finished_at, verdict),
        format!("suites: {} run, {} passed, {} failed", summary.total, summary.passed, summary.failed),
    ];
    for r in &summary.results {
        if r.status == "pass" {
            lines.push(format!("  [ok ] {} {} ({} ms)", r.index, r.suite, r.duration_ms));
        } else {
            let rc = r.exit_code.map_or("-".to_string(), |c| c.to_string());
            lines.push(format!("  [FAIL] {} {} ({} ms) exit={} log={}", r.index, r.suite, r.duration_ms, rc, r.log_path));
        }
    }
    lines.join("\n") + "\n"
}
