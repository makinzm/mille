pub mod go;
pub mod rust;

use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::ResolvedImport;
use crate::domain::repository::resolver::Resolver;
use go::GoResolver;
use rust::RustResolver;

/// Dispatches to the appropriate resolver based on file extension.
pub struct DispatchingResolver {
    rust: RustResolver,
    go: GoResolver,
}

impl DispatchingResolver {
    pub fn new(go: GoResolver) -> Self {
        DispatchingResolver {
            rust: RustResolver,
            go,
        }
    }
}

impl Resolver for DispatchingResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        if import.file.ends_with(".go") {
            self.go.resolve(import)
        } else {
            self.rust.resolve(import)
        }
    }

    fn resolve_for_project(&self, import: &RawImport, own_crate: &str) -> ResolvedImport {
        if import.file.ends_with(".go") {
            self.go.resolve_for_project(import, own_crate)
        } else {
            self.rust.resolve_for_project(import, own_crate)
        }
    }
}
