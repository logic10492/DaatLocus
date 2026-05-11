mod language;
mod analyzer;
mod api;
mod lsp;
mod server;
mod state;
mod treesitter;

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: api::JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": -32700, "message": format!("Parse error: {e}")}
                });
                let _ = writeln!(stdout.lock(), "{err}");
                continue;
            }
        };

        let resp = server::dispatch(&req);
        let json = serde_json::to_string(&resp).unwrap_or_default();
        let _ = writeln!(stdout.lock(), "{json}");
    }
}
