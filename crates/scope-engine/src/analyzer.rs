use crate::api::AffectedSelector;

pub trait Analyzer: Send + Sync {
    fn find_references(&self, selector: &str) -> Vec<AffectedSelector>;
    fn find_callers(&self, selector: &str) -> Vec<AffectedSelector>;
    fn find_definition(&self, selector: &str) -> Option<AffectedSelector>;
}
