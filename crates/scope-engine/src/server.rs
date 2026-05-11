use crate::api::*;

pub fn dispatch(req: &JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        "open_project" => {
            let params: OpenProjectRequest = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::err(req.id.clone(), -32602, format!("Invalid params: {e}")),
            };
            let _ = params;
            JsonRpcResponse::ok(req.id.clone(), serde_json::json!({"status": "opened"}))
        }
        "read_code" => {
            let params: ReadCodeRequest = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::err(req.id.clone(), -32602, format!("Invalid params: {e}")),
            };
            JsonRpcResponse::ok(
                req.id.clone(),
                serde_json::to_value(ReadCodeResponse {
                    selector: params.selector,
                    content: "// TODO: implement read_code\n".to_string(),
                    language: "rust".to_string(),
                })
                .unwrap(),
            )
        }
        "search_code" => {
            let params: SearchCodeRequest = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::err(req.id.clone(), -32602, format!("Invalid params: {e}")),
            };
            JsonRpcResponse::ok(
                req.id.clone(),
                serde_json::to_value(SearchCodeResponse { selectors: vec![] }).unwrap(),
            )
        }
        "edit_code" => {
            let params: EditCodeRequest = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::err(req.id.clone(), -32602, format!("Invalid params: {e}")),
            };
            let _ = params; // TODO: apply patch via propagation engine
            JsonRpcResponse::ok(
                req.id.clone(),
                serde_json::to_value(AffectedResponse { affected_selectors: vec![] }).unwrap(),
            )
        }
        "delete_code" => {
            let params: DeleteCodeRequest = match serde_json::from_value(req.params.clone()) {
                Ok(p) => p,
                Err(e) => return JsonRpcResponse::err(req.id.clone(), -32602, format!("Invalid params: {e}")),
            };
            let _ = params; // TODO: apply deletion via propagation engine
            JsonRpcResponse::ok(
                req.id.clone(),
                serde_json::to_value(AffectedResponse { affected_selectors: vec![] }).unwrap(),
            )
        }
        "ack_next_event" => {
            JsonRpcResponse::ok(
                req.id.clone(),
                serde_json::to_value(NextReviewResponse { review: None }).unwrap(),
            )
        }
        _ => JsonRpcResponse::err(req.id.clone(), -32601, format!("Method not found: {}", req.method)),
    }
}
