//! Canonical JSON + hashing primitives.
//!
//! The audit-ring chain hash is
//! `sha256(prev_hash || canonical_json(proto))`. The canonical form must
//! be byte-identical to the form the old TS (`canonicalJson`) and Python
//! (`json.dumps(sort_keys=True, separators=(",",":"))`) substrates
//! produced, so rows written by either legacy substrate still verify.
//!
//! Rules (shared contract, `docs/SPEC-CONSTITUTION-CLASSIFIER.md` §4):
//!   - object keys sorted lexicographically
//!   - no whitespace anywhere (`:` and `,` separators only)
//!   - `undefined`/`None` → JSON `null`
//!   - strings quoted with `"`, standard JSON escapes
//!   - numbers: integers as-is; floats with `.0` preserved (JSON
//!     `1.0` stays `1.0`, not `1`)

use serde_json::Value;

/// Serialize a JSON value canonically: sorted keys, no whitespace.
pub fn canonical(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", escape_json_string(s)),
        Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", inner.join(","))
        }
        Value::Object(map) => {
            // serde_json's default Map is a BTreeMap → keys already sorted.
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .iter()
                .map(|k| {
                    format!(
                        "\"{}\":{}",
                        escape_json_string(k),
                        canonical(&map[*k])
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
    }
}

/// Escape a string per JSON rules (no surrounding quotes).
fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// SHA-256 of a string, hex-encoded lowercase (64 chars).
pub fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex_lower(&hasher.finalize())
}

/// SHA-256 of bytes, hex-encoded lowercase.
pub fn sha256_hex_bytes(input: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex_lower(&hasher.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Current UTC time as ISO-8601 with microseconds and `Z` suffix —
/// matches the legacy Python `%Y-%m-%dT%H:%M:%S.%fZ` format.
pub fn utcnow_iso() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
}

/// The all-zero genesis hash for the audit ring.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_sorts_keys_and_minimizes() {
        let v = json!({"b": 2, "a": 1, "c": null});
        assert_eq!(canonical(&v), "{\"a\":1,\"b\":2,\"c\":null}");
    }

    #[test]
    fn canonical_nested_and_arrays() {
        let v = json!({"z": [3, {"y": true, "x": "s"}], "a": {"d": 1.0}});
        assert_eq!(
            canonical(&v),
            "{\"a\":{\"d\":1.0},\"z\":[3,{\"x\":\"s\",\"y\":true}]}"
        );
    }

    #[test]
    fn canonical_escapes_strings() {
        let v = json!({"q": "a\"b\\c\nd"});
        assert_eq!(canonical(&v), "{\"q\":\"a\\\"b\\\\c\\nd\"}");
    }

    #[test]
    fn sha256_hex_is_64_lowercase() {
        let h = sha256_hex("abc");
        assert_eq!(h.len(), 64);
        // All hex digits; any letter must be lowercase (digits are
        // neither lowercase nor uppercase in ASCII).
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()
            && (!c.is_ascii_alphabetic() || c.is_ascii_lowercase())));
        // Known vector: sha256("abc")
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn utcnow_iso_has_z_and_microseconds() {
        let t = utcnow_iso();
        assert!(t.ends_with('Z'));
        assert!(t.contains('T'));
        // pattern: YYYY-MM-DDTHH:MM:SS.ffffffZ
        assert_eq!(t.len(), 27);
    }
}
