//! Observability and telemetry reports for User Session Bootstrap Subsystem (SSO1..SSO6).
//!
//! Provides comprehensive reporting of tracked sessions, runtime state distributions,
//! seat arbitration, idle activity, user concurrency distributions, and security policy compliance.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::session::{SessionClass, SessionScope, SessionState, SessionType, UserSessionStore};
use crate::session_policy::UserSessionSecurityPolicy;

/// Canonical string representation for session state.
pub fn session_state_to_str(state: SessionState) -> &'static str {
    match state {
        SessionState::Initializing => "initializing",
        SessionState::Authenticating => "authenticating",
        SessionState::Active => "active",
        SessionState::Locked => "locked",
        SessionState::Terminating => "terminating",
        SessionState::Terminated => "terminated",
    }
}

/// Canonical string representation for session type.
pub fn session_type_to_str(st: SessionType) -> &'static str {
    match st {
        SessionType::Tty => "tty",
        SessionType::X11 => "x11",
        SessionType::Wayland => "wayland",
        SessionType::AiAgent => "ai_agent",
    }
}

/// Canonical string representation for session class.
pub fn session_class_to_str(sc: SessionClass) -> &'static str {
    match sc {
        SessionClass::User => "user",
        SessionClass::Greeter => "greeter",
        SessionClass::LockScreen => "lock_screen",
        SessionClass::Background => "background",
        SessionClass::Agent => "agent",
    }
}

/// Canonical string representation for session scope.
pub fn session_scope_to_str(scope: SessionScope) -> &'static str {
    match scope {
        SessionScope::Foreground => "foreground",
        SessionScope::Background => "background",
    }
}

/// Comprehensive observability and telemetry report for the User Session Bootstrap subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionObservabilityReport {
    /// Total number of tracked sessions in store.
    pub total_sessions: usize,
    /// Number of unique usernames active/tracked in store.
    pub distinct_users_count: usize,
    /// Histogram breakdown of sessions by runtime state.
    pub state_breakdown: BTreeMap<String, usize>,
    /// Breakdown of sessions by assigned physical seat.
    pub seat_breakdown: BTreeMap<String, usize>,
    /// Breakdown of sessions by focus scope (foreground, background).
    pub scope_breakdown: BTreeMap<String, usize>,
    /// Breakdown of sessions by execution environment type.
    pub session_type_breakdown: BTreeMap<String, usize>,
    /// Breakdown of sessions by functional classification.
    pub session_class_breakdown: BTreeMap<String, usize>,
    /// Mapping of username to count of active/tracked sessions.
    pub user_breakdown: BTreeMap<String, usize>,
    /// Total number of sessions currently in locked state.
    pub locked_count: usize,
    /// Total number of sessions with idle_seconds > 0.
    pub idle_sessions_count: usize,
    /// Maximum idle duration observed across sessions (in seconds).
    pub max_idle_seconds: u64,
    /// Aggregate idle time across all sessions (in seconds).
    pub total_idle_seconds: u64,
    /// Number of sessions passing security policy constraints.
    pub policy_compliant_count: usize,
    /// Number of sessions with one or more security policy violations.
    pub policy_violations_count: usize,
    /// Session IDs with active policy violations.
    pub violating_sessions: Vec<String>,
    /// RFC-3339 generation timestamp.
    pub generated_at: String,
}

impl SessionObservabilityReport {
    /// Generates an observability report from the provided store and optional security policy (SSO1..SSO6).
    pub fn generate(
        store: &UserSessionStore,
        policy_opt: Option<&UserSessionSecurityPolicy>,
    ) -> Self {
        let total_sessions = store.sessions.len();

        let mut state_breakdown = BTreeMap::new();
        state_breakdown.insert("initializing".into(), 0);
        state_breakdown.insert("authenticating".into(), 0);
        state_breakdown.insert("active".into(), 0);
        state_breakdown.insert("locked".into(), 0);
        state_breakdown.insert("terminating".into(), 0);
        state_breakdown.insert("terminated".into(), 0);

        let mut seat_breakdown = BTreeMap::new();
        let mut scope_breakdown = BTreeMap::new();
        scope_breakdown.insert("foreground".into(), 0);
        scope_breakdown.insert("background".into(), 0);

        let mut session_type_breakdown = BTreeMap::new();
        session_type_breakdown.insert("tty".into(), 0);
        session_type_breakdown.insert("x11".into(), 0);
        session_type_breakdown.insert("wayland".into(), 0);
        session_type_breakdown.insert("ai_agent".into(), 0);

        let mut session_class_breakdown = BTreeMap::new();
        session_class_breakdown.insert("user".into(), 0);
        session_class_breakdown.insert("greeter".into(), 0);
        session_class_breakdown.insert("lock_screen".into(), 0);
        session_class_breakdown.insert("background".into(), 0);
        session_class_breakdown.insert("agent".into(), 0);

        let mut user_breakdown = BTreeMap::new();
        let mut locked_count = 0;
        let mut idle_sessions_count = 0;
        let mut max_idle_seconds = 0;
        let mut total_idle_seconds = 0;

        for (id, status) in &store.sessions {
            // State
            let state_str = session_state_to_str(status.state);
            *state_breakdown.entry(state_str.to_string()).or_insert(0) += 1;

            // Scope
            let scope_str = session_scope_to_str(status.scope);
            *scope_breakdown.entry(scope_str.to_string()).or_insert(0) += 1;

            // User
            *user_breakdown.entry(status.username.clone()).or_insert(0) += 1;

            // Locked & Idle
            if status.locked || status.state == SessionState::Locked {
                locked_count += 1;
            }
            if status.idle_seconds > 0 {
                idle_sessions_count += 1;
            }
            if status.idle_seconds > max_idle_seconds {
                max_idle_seconds = status.idle_seconds;
            }
            total_idle_seconds += status.idle_seconds;

            // Seat, Type, Class from Spec if present
            if let Some(spec) = store.specs.get(id) {
                *seat_breakdown.entry(spec.seat.clone()).or_insert(0) += 1;

                let type_str = session_type_to_str(spec.session_type);
                *session_type_breakdown.entry(type_str.to_string()).or_insert(0) += 1;

                let class_str = session_class_to_str(spec.session_class);
                *session_class_breakdown.entry(class_str.to_string()).or_insert(0) += 1;
            }
        }

        let distinct_users_count = user_breakdown.len();

        // Policy evaluation
        let mut policy_compliant_count = total_sessions;
        let mut policy_violations_count = 0;
        let mut violating_sessions = Vec::new();

        if let Some(policy) = policy_opt {
            let verdicts = policy.evaluate_store(store);
            let mut failed_ids = Vec::new();
            for v in &verdicts {
                if !v.allowed || !v.violations.is_empty() {
                    failed_ids.push(v.session_id.clone());
                }
            }
            policy_violations_count = failed_ids.len();
            if policy_violations_count > total_sessions {
                policy_compliant_count = 0;
            } else {
                policy_compliant_count = total_sessions - policy_violations_count;
            }
            violating_sessions = failed_ids;
        }

        SessionObservabilityReport {
            total_sessions,
            distinct_users_count,
            state_breakdown,
            seat_breakdown,
            scope_breakdown,
            session_type_breakdown,
            session_class_breakdown,
            user_breakdown,
            locked_count,
            idle_sessions_count,
            max_idle_seconds,
            total_idle_seconds,
            policy_compliant_count,
            policy_violations_count,
            violating_sessions,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
