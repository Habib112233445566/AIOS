//! Task Ledger Control — configuration layer (T-00053 SCAFFOLD).
//!
//! Typed interfaces only; bodies fail loudly until T-00054.
//! Contract: `docs/tasks/evidence/T-00052-spec.md`.
//!
//! Twelve-Factor III alignment: configuration lives in ENVIRONMENT
//! VARIABLES with built-in defaults equal to today's constants — no
//! config files (E2 names their weaknesses). Loaded fresh at each
//! operation start; invalid values fail loudly naming the variable.
//!
//! Knobs (env var -> default):
//!   AIOSH_LEDGER_LOCK_TIMEOUT_SECS  -> 5
//!   AIOSH_LEDGER_MAX_LEDGER_BYTES   -> 67108864
//!   AIOSH_LEDGER_MAX_EVENTS_BYTES   -> 16777216
//!   AIOSH_LEDGER_MAX_STATE_BYTES    -> 4194304
//!   AIOSH_LEDGER_MAX_TEXT           -> 4096
//!   AIOSH_LEDGER_MAX_EVIDENCE_ITEMS -> 16

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerConfig {
    pub lock_timeout_secs: u64,
    pub max_ledger_bytes: u64,
    pub max_events_bytes: u64,
    pub max_state_bytes: u64,
    pub max_text: usize,
    pub max_evidence_items: usize,
}

impl Default for LedgerConfig {
    /// Defaults == the constants shipped through T-00028/T-00034.
    fn default() -> Self {
        Self {
            lock_timeout_secs: 5,
            max_ledger_bytes: 64 * 1024 * 1024,
            max_events_bytes: 16 * 1024 * 1024,
            max_state_bytes: 4 * 1024 * 1024,
            max_text: 4096,
            max_evidence_items: 16,
        }
    }
}

impl LedgerConfig {
    /// Read the six `AIOSH_LEDGER_*` variables. Precedence:
    /// env var > default. Unparseable/out-of-range values produce
    /// `Err("invalid AIOSH_LEDGER_<NAME>='<raw>': <why>")`.
    /// Constraints: lock >=1; byte caps >=1024; text >=64; items >=1.
    pub fn from_env() -> Result<LedgerConfig, String> {
        Self::from_source(&|name| std::env::var(name).ok())
    }

    /// Dependency-injected variant for tests/diagnostics: `get` returns
    /// None for "unset". Same precedence/constraint rules as from_env.
    pub fn from_source(
        get: &dyn Fn(&str) -> Option<String>,
    ) -> Result<LedgerConfig, String> {
        let d = LedgerConfig::default();
        #[allow(clippy::let_and_return)]
        let get = |name: &str, default: u64, min: u64| -> Result<u64, String> {
            match get(name) {
                None => Ok(default),
                Some(raw) => {
                    let parsed: u64 = raw.trim().parse().map_err(|_| {
                        format!("invalid {name}='{raw}': not a decimal integer")
                    })?;
                    if parsed < min {
                        return Err(format!(
                            "invalid {name}='{parsed}': must be >= {min}"
                        ));
                    }
                    Ok(parsed)
                }
            }
        };
        // T-00058 hardening: explicit CEILING on the lock wait so no
        // platform/toolchain arithmetic edge can turn a typo into a
        // multi-decade hang (T-00057 S4 caveat closed by construction).
        let lock_timeout_secs = {
            let v = get("AIOSH_LEDGER_LOCK_TIMEOUT_SECS", d.lock_timeout_secs, 1)?;
            const LOCK_CEILING_SECS: u64 = 86_400; // 24h
            if v > LOCK_CEILING_SECS {
                return Err(format!(
                    "invalid AIOSH_LEDGER_LOCK_TIMEOUT_SECS='{v}': must be <= {LOCK_CEILING_SECS}"
                ));
            }
            v
        };
        Ok(LedgerConfig {
            lock_timeout_secs,
            max_ledger_bytes: get("AIOSH_LEDGER_MAX_LEDGER_BYTES", d.max_ledger_bytes, 1024)?,
            max_events_bytes: get("AIOSH_LEDGER_MAX_EVENTS_BYTES", d.max_events_bytes, 1024)?,
            max_state_bytes: get("AIOSH_LEDGER_MAX_STATE_BYTES", d.max_state_bytes, 1024)?,
            max_text: get("AIOSH_LEDGER_MAX_TEXT", d.max_text as u64, 64)? as usize,
            max_evidence_items: get(
                "AIOSH_LEDGER_MAX_EVIDENCE_ITEMS",
                d.max_evidence_items as u64,
                1,
            )? as usize,
        })
    }

    /// Effective-config view for `aiosh task config`: one object entry
    /// per knob: {"value": n, "source": "env"|"default"}.
    pub fn to_json_with_sources(&self) -> Value {
        self.to_json_with_sources_from(&|name| std::env::var(name).is_ok())
    }

    /// Injected-source variant of the sources view.
    pub fn to_json_with_sources_from(
        &self,
        is_set: &dyn Fn(&str) -> bool,
    ) -> Value {
        let src = |name: &str| if is_set(name) { "env" } else { "default" };
        json!({
            "lock_timeout_secs": {"value": self.lock_timeout_secs,
                "source": src("AIOSH_LEDGER_LOCK_TIMEOUT_SECS")},
            "max_ledger_bytes": {"value": self.max_ledger_bytes,
                "source": src("AIOSH_LEDGER_MAX_LEDGER_BYTES")},
            "max_events_bytes": {"value": self.max_events_bytes,
                "source": src("AIOSH_LEDGER_MAX_EVENTS_BYTES")},
            "max_state_bytes": {"value": self.max_state_bytes,
                "source": src("AIOSH_LEDGER_MAX_STATE_BYTES")},
            "max_text": {"value": self.max_text,
                "source": src("AIOSH_LEDGER_MAX_TEXT")},
            "max_evidence_items": {"value": self.max_evidence_items,
                "source": src("AIOSH_LEDGER_MAX_EVIDENCE_ITEMS")},
        })
    }
}

/// Convenience: defaults as JSON (used by tests/call sites).
pub fn defaults_json() -> Value {
    let d = LedgerConfig::default();
    json!({
        "lock_timeout_secs": d.lock_timeout_secs,
        "max_ledger_bytes": d.max_ledger_bytes,
        "max_events_bytes": d.max_events_bytes,
        "max_state_bytes": d.max_state_bytes,
        "max_text": d.max_text,
        "max_evidence_items": d.max_evidence_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Precedence + loud errors via INJECTED source (no process-env
    /// mutation -> no parallel-test races).
    #[test]
    fn from_source_precedence_and_loud_errors() {
        let mut vars: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        // defaults (closure rebuilt after each mutation below)
        let c = LedgerConfig::from_source(&|n: &str| vars.get(n).cloned()).unwrap();
        assert_eq!(c.lock_timeout_secs, 5);
        assert_eq!(c.max_text, 4096);
        // valid override
        vars.insert("AIOSH_LEDGER_LOCK_TIMEOUT_SECS".into(), "9".into());
        assert_eq!(
            LedgerConfig::from_source(&|n: &str| vars.get(n).cloned())
                .unwrap()
                .lock_timeout_secs,
            9
        );
        // invalid: non-numeric names the variable
        vars.insert("AIOSH_LEDGER_LOCK_TIMEOUT_SECS".into(), "soon".into());
        let e =
            LedgerConfig::from_source(&|n: &str| vars.get(n).cloned()).unwrap_err();
        assert!(e.contains("invalid AIOSH_LEDGER_LOCK_TIMEOUT_SECS='soon'"), "{e}");
        // range violation
        vars.insert("AIOSH_LEDGER_LOCK_TIMEOUT_SECS".into(), "0".into());
        assert!(LedgerConfig::from_source(&|n: &str| vars.get(n).cloned())
            .unwrap_err()
            .contains("must be >= 1"));
        // ceiling (T-00058): absurd timeouts refused loudly
        vars.insert("AIOSH_LEDGER_LOCK_TIMEOUT_SECS".into(), "86401".into());
        assert!(LedgerConfig::from_source(&|n: &str| vars.get(n).cloned())
            .unwrap_err()
            .contains("must be <= 86400"));
        vars.remove("AIOSH_LEDGER_LOCK_TIMEOUT_SECS"); // back to defaults
        // sources view
        vars.insert("AIOSH_LEDGER_MAX_TEXT".into(), "8192".into());
        let c = LedgerConfig::from_source(&|n: &str| vars.get(n).cloned()).unwrap();
        let v = c.to_json_with_sources_from(&|n| vars.contains_key(n));
        assert_eq!(v["max_text"]["value"], json!(8192));
        assert_eq!(v["max_text"]["source"], json!("env"));
        assert_eq!(v["max_state_bytes"]["source"], json!("default"));
    }

    /// Type-level wiring proof without invoking todo bodies.
    #[test]
    fn scaffold_defaults_compose() {
        let d = LedgerConfig::default();
        assert_eq!(d.lock_timeout_secs, 5);
        assert_eq!(d.max_text, 4096);
        assert_eq!(d.max_evidence_items, 16);
        let v: Value = defaults_json();
        assert_eq!(v["max_ledger_bytes"], json!(64 * 1024 * 1024));
    }
}
