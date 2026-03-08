/// A detected architecture rule violation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Violation {
    /// Source file where the offending import lives.
    pub file: String,
    /// 1-indexed line number of the import.
    pub line: usize,
    /// Layer name of the importing file.
    pub from_layer: String,
    /// Layer name of the imported path.
    pub to_layer: String,
    /// The raw import path string.
    pub import_path: String,
    pub kind: ViolationKind,
    pub severity: Severity,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ViolationKind {
    /// `dependency_mode` rule was broken (opt-in: to_layer not in allow; opt-out: to_layer in deny).
    DependencyViolation,
    /// `external_mode` rule was broken: an external crate not in `external_allow` (opt-in) or in
    /// `external_deny` (opt-out) was imported. `to_layer` holds the crate name.
    ExternalViolation,
    /// `allow_call_patterns` rule was broken: a method not in `allow_methods` was called on a type
    /// from `callee_layer`.
    CallPatternViolation,
    /// Import could not be classified (Unknown category). `import_path` holds the raw import string.
    UnknownImport,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
}
