use crate::analyzer::Analyzer;
use crate::api::AffectedSelector;

pub struct TreeSitterAnalyzer;

impl TreeSitterAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Analyzer for TreeSitterAnalyzer {
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
