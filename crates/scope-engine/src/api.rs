use serde::{Deserialize, Serialize};

// ── JSON-RPC 2.0 types ──────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn ok(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }
    pub fn err(id: serde_json::Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError { code, message: message.into() }),
        }
    }
}

// ── Domain types ────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct OpenProjectRequest {
    pub project_root: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReadCodeRequest {
    pub selector: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadCodeResponse {
    pub selector: String,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchCodeRequest {
    pub query: String,
    pub project_root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchCodeResponse {
    pub selectors: Vec<String>,
}

/// edit_code parameters: selector + stripped v4a hunk-only patch
#[derive(Debug, Clone, Deserialize)]
pub struct EditCodeRequest {
    pub selector: String,
    /// Stripped v4a patch (hunk-only, no file header)
    pub patch: String,
}

/// delete_code parameters
#[derive(Debug, Clone, Deserialize)]
pub struct DeleteCodeRequest {
    pub selector: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AffectedResponse {
    pub affected_selectors: Vec<AffectedSelector>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AffectedSelector {
    pub selector: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NextReviewResponse {
    pub review: Option<ReviewEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewEvent {
    pub selector: String,
    pub reason: String,
    pub suggested_action: String,
}
