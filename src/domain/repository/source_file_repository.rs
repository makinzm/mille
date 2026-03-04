/// Port for collecting source file paths that match a set of glob patterns.
/// Concrete implementations live in `infrastructure::repository`.
pub trait SourceFileRepository {
    fn collect(&self, patterns: &[String]) -> Vec<String>;
}
