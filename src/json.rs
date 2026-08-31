//! JSON envelope — `{"status":"success","data":…}` / `{"status":"error","code":N,"message":"…"}`.

use serde_json::{json, Value};

/// Success envelope.
#[must_use]
pub fn ok(data: &Value) -> Value {
    json!({ "status": "success", "data": data })
}

/// Error envelope; `code` MUST equal the process's numeric exit code.
#[must_use]
pub fn err(code: i32, message: &str) -> Value {
    json!({ "status": "error", "code": code, "message": message })
}
