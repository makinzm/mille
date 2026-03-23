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
    /// String literal content.
    StringLiteral,
    /// Identifier reference (e.g. attribute access segments like `gcp` in `cfg.gcp.bucket`).
    Identifier,
}

/// Parsed names grouped by kind.
///
/// `Default` is intentionally NOT derived so that adding a new field
/// causes a compile error in every parser that constructs this struct.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedNames {
    /// Function, struct, enum, class, trait, interface, type alias definition names.
    pub symbols: Vec<RawName>,
    /// Variable, const, let, static declaration names.
    pub variables: Vec<RawName>,
    /// Inline comment contents.
    pub comments: Vec<RawName>,
    /// String literal contents.
    pub string_literals: Vec<RawName>,
    /// Identifier references (e.g. attribute access segments).
    pub identifiers: Vec<RawName>,
}

impl ParsedNames {
    /// Flatten all parsed names into a single `Vec<RawName>`.
    pub fn into_all(self) -> Vec<RawName> {
        let mut out = Vec::with_capacity(
            self.symbols.len()
                + self.variables.len()
                + self.comments.len()
                + self.string_literals.len()
                + self.identifiers.len(),
        );
        out.extend(self.symbols);
        out.extend(self.variables);
        out.extend(self.comments);
        out.extend(self.string_literals);
        out.extend(self.identifiers);
        out
    }
}
