/// A raw import statement extracted directly from source code, before any path resolution.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawImport {
    /// The import path as it appears in source (e.g. `crate::domain::entity::config`).
    pub path: String,
    /// 1-indexed line number in the source file.
    pub line: usize,
    /// Path of the source file this import was found in.
    pub file: String,
    /// Whether this is a `use` or external `mod` declaration.
    pub kind: ImportKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ImportKind {
    /// `use crate::foo;` or `pub use crate::foo;`
    Use,
    /// `mod foo;` or `pub mod foo;` — external module declaration (no inline body)
    Mod,
    /// `import "pkg/path"` — Go import declaration
    Import,
}
