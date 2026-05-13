//! SCOPE engine in-process client.
//!
//! Provides direct access to scope-engine functionality (tree-sitter parsing,
//! symbol lookup, code editing, propagation analysis) without spawning a
//! separate JSON-RPC process. The scope-engine crate is linked directly as
//! a library dependency.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use scope_engine::api;
use scope_engine::language::LanguageRegistry;
use scope_engine::selector;
use scope_engine::state::PropagationState;
use scope_engine::treesitter::TreeSitterAnalyzer;

/// In-process SCOPE engine client.
///
/// Wraps the scope-engine library to provide:
/// - Symbol-based code reading via selector
/// - Text-based code search
/// - Selector-based code editing and deletion
/// - Propagation review events
/// - Tree-sitter symbol lookup
/// - Config hints for language servers
pub struct ScopeClient {
    project_root: Option<PathBuf>,
    propagation_state: Mutex<PropagationState>,
    tree_sitter: TreeSitterAnalyzer,
}

impl ScopeClient {
    /// Create a new scope client (no project opened yet).
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            project_root: None,
            propagation_state: Mutex::new(PropagationState::new()),
            tree_sitter: TreeSitterAnalyzer::new(),
        }
    }

    /// Open a project, setting the root directory for subsequent operations.
    #[allow(dead_code)]
    pub fn open_project(&mut self, project_root: impl Into<PathBuf>, _language: Option<&str>) {
        self.project_root = Some(project_root.into());
        // LSP initialization can be added later via scope_engine::lsp
    }

    /// The project root path, if a project has been opened.
    #[allow(dead_code)]
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Accumulate propagation results and get the next review event, if any.
    #[allow(dead_code)]
    pub fn next_review_event(
        &self,
        results: Vec<api::PropagationResult>,
    ) -> Option<api::ReviewEvent> {
        let mut state = self
            .propagation_state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.accumulate(results);
        state.next_review()
    }

    /// Find the containing symbol for a given file and line number using tree-sitter.
    #[allow(dead_code)]
    pub fn find_containing_symbol(&self, file_path: &Path, line_number: usize) -> Option<String> {
        let root = self.project_root.as_deref()?;
        self.tree_sitter
            .find_containing_symbol(file_path, line_number, root)
    }

    /// Parse a selector string into a structured `ParsedSelector`.
    #[allow(dead_code)]
    pub fn parse_selector(selector_str: &str) -> Result<selector::ParsedSelector, String> {
        selector::parse_selector(selector_str)
    }

    /// Resolve a selector's file path against the project root.
    #[allow(dead_code)]
    pub fn resolve_file(
        &self,
        parsed: &selector::ParsedSelector,
    ) -> Result<(PathBuf, String), String> {
        let root = self.project_root.as_deref().ok_or("no project opened")?;
        selector::resolve_file(parsed, root)
    }

    /// Get config hints for language servers and tree-sitter languages.
    ///
    /// Returns a JSON-RPC response containing `tree_sitter_languages` and `lsp_languages`.
    #[allow(dead_code)]
    pub fn get_config_hints() -> api::JsonRpcResponse {
        let fake_id = serde_json::json!(1);
        let fake_req = api::JsonRpcRequest {
            _jsonrpc: "2.0".to_string(),
            id: fake_id,
            method: "get_config_hints".to_string(),
            params: serde_json::Value::Null,
        };
        scope_engine::server::dispatch_get_config_hints(&fake_req)
    }

    /// Get the list of supported tree-sitter languages.
    #[allow(dead_code)]
    pub fn supported_languages() -> Vec<(String, Vec<String>)> {
        let registry = LanguageRegistry::new();
        registry
            .list_languages()
            .into_iter()
            .map(|(name, exts)| (name.to_string(), exts.iter().map(|e| e.to_string()).collect()))
            .collect()
    }
}

impl Default for ScopeClient {
    fn default() -> Self {
        Self::new()
    }
}
