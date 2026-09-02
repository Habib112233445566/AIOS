# T-01012 — Distro Selection & Justification / Core Service: Specification

## 1. DistroStore Service Specification

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroStore {
    profiles: HashMap<String, DistroProfile>,
}

impl DistroStore {
    /// Instantiates store with default canonical profiles.
    pub fn new() -> Self;

    /// Registers a new profile after running validate_distro_profile.
    pub fn register_profile(&mut self, profile: DistroProfile) -> Result<(), String>;

    /// Retrieves a reference to a profile by its unique ID.
    pub fn get_profile(&self, id: &str) -> Option<&DistroProfile>;

    /// Returns a list of all registered profiles sorted by ID.
    pub fn list_profiles(&self) -> Vec<&DistroProfile>;

    /// Evaluates a single profile and returns its DistroEvaluation.
    pub fn evaluate_profile(&self, id: &str) -> Result<DistroEvaluation, String>;

    /// Evaluates all registered profiles and returns sorted evaluations.
    pub fn evaluate_all(&self) -> Vec<DistroEvaluation>;

    /// Returns the recommended production distribution profile.
    pub fn get_recommended_profile(&self) -> Option<&DistroProfile>;

    /// Persists registry atomically to JSON file.
    pub fn save_to_path(&self, path: &Path) -> Result<(), String>;

    /// Loads registry from JSON file.
    pub fn load_from_path(path: &Path) -> Result<Self, String>;

    /// Loads registry from JSON file or recovers with defaults on error.
    pub fn load_or_recover(path: &Path) -> Self;
}
```

## 2. Invariants & Guarantees
- Uniqueness: Duplicate profile IDs are overwritten only with valid updated definitions.
- Fallback Safety: `load_or_recover` guarantees at least the Debian 12 Minimal profile is always available.
- Deterministic Evaluation: `evaluate_all` sorts by `overall_score` descending.
