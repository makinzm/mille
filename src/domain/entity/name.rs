/// A raw name extracted from source code for naming convention checks.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawName {
    /// The name string as it appears in source (e.g. `AwsClient`, `aws_url`).
    pub name: String,
    /// 1-indexed line number (0 for file-level checks like filename).
    pub line: usize,
    /// The kind of name target this represents.
    pub kind: NameKind,
    /// Path of the source file this name was found in.
    pub file: String,
}

/// The kind of naming target.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NameKind {
    /// File basename (no extension).
    File,
    /// Function, struct, enum, class, trait, interface, type alias definition name.
    Symbol,
    /// Variable, const, let, static declaration name.
    Variable,
    /// Inline comment content.
    Comment,
}
