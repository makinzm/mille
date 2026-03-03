use crate::domain::entity::import::RawImport;

/// A `RawImport` after path resolution and category classification.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResolvedImport {
    pub raw: RawImport,
    pub category: ImportCategory,
    /// Normalised file-system path for `Internal` imports (e.g. `src/domain/entity/config`).
    /// `None` when the path could not be resolved (wildcards, grouped imports, `super::` etc.).
    pub resolved_path: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ImportCategory {
    /// `crate::*`, `super::*`, `self::*` — lives inside this crate.
    Internal,
    /// Third-party crates (serde, toml, …).
    External,
    /// `std::*`, `core::*`, `alloc::*`.
    Stdlib,
    /// Could not be determined.
    Unknown,
}
