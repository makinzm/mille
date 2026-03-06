pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

use self::go::GoResolver;
use self::python::PythonResolver;
use self::rust::RustResolver;
use self::typescript::TypeScriptResolver;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::ResolvedImport;
use crate::domain::repository::resolver::Resolver;

/// Dispatches to the appropriate resolver based on file extension.
pub struct DispatchingResolver {
    rust: RustResolver,
    go: GoResolver,
    python: PythonResolver,
    typescript: TypeScriptResolver,
}

impl DispatchingResolver {
    pub fn new(go: GoResolver, python: PythonResolver, typescript: TypeScriptResolver) -> Self {
        DispatchingResolver {
            rust: RustResolver,
            go,
            python,
            typescript,
        }
    }
}

fn is_ts_js(file: &str) -> bool {
    file.ends_with(".ts")
        || file.ends_with(".tsx")
        || file.ends_with(".js")
        || file.ends_with(".jsx")
}

impl Resolver for DispatchingResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        if import.file.ends_with(".go") {
            self.go.resolve(import)
        } else if import.file.ends_with(".py") {
            self.python.resolve(import)
        } else if is_ts_js(&import.file) {
            self.typescript.resolve(import)
        } else {
            self.rust.resolve(import)
        }
    }

    fn resolve_for_project(&self, import: &RawImport, own_crate: &str) -> ResolvedImport {
        if import.file.ends_with(".go") {
            self.go.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".py") {
            self.python.resolve_for_project(import, own_crate)
        } else if is_ts_js(&import.file) {
            self.typescript.resolve_for_project(import, own_crate)
        } else {
            self.rust.resolve_for_project(import, own_crate)
        }
    }
}
