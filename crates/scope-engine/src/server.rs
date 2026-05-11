use crate::api::*;
use axum::{
    routing::{get, post},
    Json, Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/open", post(open_project))
        .route("/v1/read", post(read_code))
        .route("/v1/search", post(search_code))
        .route("/v1/edit", post(notify_edit))
        .route("/v1/reviews/next", get(next_review))
}

async fn health() -> &'static str {
    "SCOPE engine OK"
}

async fn open_project(Json(_req): Json<OpenProjectRequest>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "opened" }))
}

async fn read_code(Json(req): Json<ReadCodeRequest>) -> Json<ReadCodeResponse> {
    Json(ReadCodeResponse {
        selector: req.selector,
        content: "// TODO: implement read_code\n".to_string(),
        language: "rust".to_string(),
    })
}

async fn search_code(Json(_req): Json<SearchCodeRequest>) -> Json<SearchCodeResponse> {
    Json(SearchCodeResponse { selectors: vec![] })
}

async fn notify_edit(Json(_req): Json<EditNotifyRequest>) -> Json<AffectedResponse> {
    Json(AffectedResponse {
        affected_selectors: vec![],
    })
}

async fn next_review() -> Json<NextReviewResponse> {
    Json(NextReviewResponse { review: None })
}
