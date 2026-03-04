use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::ResolvedImport;

/// Port for resolving a raw import path into a categorised, normalised import.
/// Concrete implementations live in `infrastructure::resolver`.
pub trait Resolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport;
}
