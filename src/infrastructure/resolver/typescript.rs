use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for TypeScript/JavaScript imports.
///
/// Classification rules:
/// - **Internal**: relative imports starting with `./` or `../`
/// - **External**: everything else (npm packages, Node.js built-ins, etc.)
pub struct TypeScriptResolver;

impl TypeScriptResolver {
    pub fn new() -> Self {
        TypeScriptResolver
    }
}

impl Default for TypeScriptResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver for TypeScriptResolver {
    fn resolve(&self, _import: &RawImport) -> ResolvedImport {
        todo!("TypeScriptResolver::resolve not implemented")
    }

    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        self.resolve(import)
    }
}
