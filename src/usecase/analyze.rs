use crate::domain::repository::config_repository::ConfigRepository;
use crate::domain::repository::parser::Parser;
use crate::domain::repository::resolver::Resolver;
use crate::domain::repository::source_file_repository::SourceFileRepository;

pub struct AnalyzeResult {
    pub nodes: Vec<LayerNode>,
    pub edges: Vec<LayerEdge>,
}

pub struct LayerNode {
    pub name: String,
    pub file_count: usize,
}

pub struct LayerEdge {
    pub from: String,
    pub to: String,
    pub import_count: usize,
}

pub fn analyze(
    config_path: &str,
    config_repo: &dyn ConfigRepository,
    file_repo: &dyn SourceFileRepository,
    parser: &dyn Parser,
    resolver: &dyn Resolver,
) -> Result<AnalyzeResult, String> {
    todo!()
}
