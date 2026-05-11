use crate::api::{AffectedSelector, ReviewEvent};
use std::collections::HashSet;

pub struct AffectedState {
    pending: Vec<AffectedSelector>,
    seen: HashSet<String>,
}

impl AffectedState {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn accumulate(&mut self, selectors: Vec<AffectedSelector>) {
        for sel in selectors {
            if self.seen.insert(sel.selector.clone()) {
                self.pending.push(sel);
            }
        }
    }

    pub fn next_review(&mut self) -> Option<ReviewEvent> {
        let sel = self.pending.pop()?;
        Some(ReviewEvent {
            selector: sel.selector,
            reason: sel.reason,
            suggested_action: "read_and_verify".to_string(),
        })
    }
}
