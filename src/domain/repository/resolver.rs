use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::ResolvedImport;

/// Port for resolving a raw import path into a classified, normalised import.
/// Concrete implementations live in `infrastructure::resolver`.
pub trait Resolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport;

    /// Like `resolve`, but also treats `<own_crate>::` paths as Internal.
    /// This is needed when the binary crate (`main.rs`) imports from the library crate
    /// using the published crate name (e.g. `use mille::infrastructure::…`) rather than
    /// the `crate::` alias.  The default delegates to `resolve` (ignores own_crate).
    /// Override this in concrete implementations to handle the own-crate prefix.
    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        self.resolve(import)
    }
}
