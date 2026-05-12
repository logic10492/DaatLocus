use crate::api::{PropagationResult, PropagationSource, ReviewEvent};
use std::collections::HashSet;

pub struct PropagationState {
    pending: Vec<PropagationResult>,
    seen: HashSet<String>,
}

impl PropagationState {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn accumulate(&mut self, results: Vec<PropagationResult>) {
        for r in results {
            if self.seen.insert(r.selector.clone()) {
                self.pending.push(r);
            }
        }
    }

    pub fn next_review(&mut self) -> Option<ReviewEvent> {
        let r = self.pending.pop()?;
        let suggested_action = match &r.source {
            PropagationSource::Lsp => "read_and_verify".to_string(),
            PropagationSource::OpenEnded => "investigate_impact".to_string(),
        };
        Some(ReviewEvent {
            selector: r.selector,
            reason: r.reason,
            suggested_action,
            source: r.source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lsp_result(selector: &str, reason: &str) -> PropagationResult {
        PropagationResult {
            selector: selector.to_string(),
            reason: reason.to_string(),
            source: PropagationSource::Lsp,
        }
    }

    fn open_result(selector: &str, reason: &str) -> PropagationResult {
        PropagationResult {
            selector: selector.to_string(),
            reason: reason.to_string(),
            source: PropagationSource::OpenEnded,
        }
    }

    #[test]
    fn accumulate_deduplicates_selectors() {
        let mut state = PropagationState::new();
        state.accumulate(vec![
            lsp_result("src/a.rs::fn foo", "modified"),
            lsp_result("src/a.rs::fn foo", "modified again"),
        ]);
        assert_eq!(state.pending.len(), 1);
    }

    #[test]
    fn accumulate_keeps_distinct_selectors() {
        let mut state = PropagationState::new();
        state.accumulate(vec![
            lsp_result("src/a.rs::fn foo", "modified"),
            lsp_result("src/b.rs::fn bar", "modified"),
        ]);
        assert_eq!(state.pending.len(), 2);
    }

    #[test]
    fn next_review_returns_lsp_event_with_read_and_verify() {
        let mut state = PropagationState::new();
        state.accumulate(vec![lsp_result("src/a.rs::fn foo", "referenced")]);
        let event = state.next_review().unwrap();
        assert_eq!(event.selector, "src/a.rs::fn foo");
        assert_eq!(event.reason, "referenced");
        assert_eq!(event.suggested_action, "read_and_verify");
        assert_eq!(event.source, PropagationSource::Lsp);
    }

    #[test]
    fn next_review_returns_open_ended_event_with_investigate_impact() {
        let mut state = PropagationState::new();
        state.accumulate(vec![open_result("src/a.rs::fn foo", "modified")]);
        let event = state.next_review().unwrap();
        assert_eq!(event.selector, "src/a.rs::fn foo");
        assert_eq!(event.suggested_action, "investigate_impact");
        assert_eq!(event.source, PropagationSource::OpenEnded);
    }

    #[test]
    fn next_review_returns_none_when_empty() {
        let mut state = PropagationState::new();
        assert!(state.next_review().is_none());
    }

    #[test]
    fn next_review_pops_in_lifo_order() {
        let mut state = PropagationState::new();
        state.accumulate(vec![
            lsp_result("src/a.rs::fn foo", "first"),
            lsp_result("src/b.rs::fn bar", "second"),
        ]);
        let e1 = state.next_review().unwrap();
        assert_eq!(e1.selector, "src/b.rs::fn bar");
        let e2 = state.next_review().unwrap();
        assert_eq!(e2.selector, "src/a.rs::fn foo");
    }

    #[test]
    fn mixed_sources_accumulate_independently() {
        let mut state = PropagationState::new();
        state.accumulate(vec![
            lsp_result("src/a.rs::fn foo", "lsp ref"),
            open_result("src/b.rs::fn bar", "open ref"),
        ]);
        assert_eq!(state.pending.len(), 2);
        let e1 = state.next_review().unwrap();
        assert_eq!(e1.source, PropagationSource::OpenEnded);
        let e2 = state.next_review().unwrap();
        assert_eq!(e2.source, PropagationSource::Lsp);
    }
}
