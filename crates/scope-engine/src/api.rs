use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize)]
pub struct EditNotifyRequest {
    pub selector: String,
    pub project_root: String,
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
