use crate::analyzer::Analyzer;
use crate::api::AffectedSelector;

pub struct LspAnalyzer;

impl LspAnalyzer {
    pub async fn new(_project_root: &str, _language: &str) -> Self {
        Self
    }
}

impl Analyzer for LspAnalyzer {
    fn find_references(&self, _selector: &str) -> Vec<AffectedSelector> {
        vec![]
    }
    fn find_callers(&self, _selector: &str) -> Vec<AffectedSelector> {
        vec![]
    }
    fn find_definition(&self, _selector: &str) -> Option<AffectedSelector> {
        None
    }
}
