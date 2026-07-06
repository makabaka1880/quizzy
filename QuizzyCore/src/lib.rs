pub mod types;
pub mod crypto;
pub mod encode;
pub mod decode;
pub mod problem;

// Re-export key public API
pub use encode::{encode_full, decode_session, decode_answer_key, decode_all, build_answer_key, package_student_safe, encrypt_problem_set, decrypt_problem_set};
pub use problem::{create_problem_set, mcq, mmcq, frq, ffrq, to_json, from_json};

// ---------- WASM bindings ----------

#[cfg(feature = "wasm")]
mod wasm_bindings {
    use wasm_bindgen::prelude::*;
    use crate::types::*;
    use crate::encode;

    /// Encode a quiz result.
    /// `encrypted_answer_key` is the opaque blob from the StudentProblemSet
    /// (pre-encrypted by the teacher at package time). The frontend passes it through.
    #[wasm_bindgen]
    pub fn encode_quiz_result(
        student_id: &str,
        timestamp: f64,
        entries_json: &str,
        encrypted_answer_key: &str,
    ) -> Result<String, JsValue> {
        let entries: Vec<GradingEntry> = serde_json::from_str(entries_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid entries JSON: {}", e)))?;

        let session = SessionState {
            student_id: student_id.to_string(),
            timestamp: timestamp as u64,
            entries,
        };

        Ok(encode::encode_full(
            student_id,
            timestamp as u64,
            &session,
            encrypted_answer_key,
        ))
    }

    /// Student-facing: decode just the session (answers + timing) from a result blob.
    #[wasm_bindgen]
    pub fn decode_my_session(encoded: &str) -> Result<String, JsValue> {
        let session =
            encode::decode_session(encoded).map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&session)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Teacher-facing: decode both session and answer key.
    #[wasm_bindgen]
    pub fn decode_all_wasm(encoded: &str, quiz_key: &str) -> Result<String, JsValue> {
        let decoded = encode::decode_all(encoded, quiz_key)
            .map_err(|e| JsValue::from_str(&e))?;
        serde_json::to_string(&decoded)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Teacher-facing: produce a student-safe problem set from a full teacher problem set.
    /// Input: JSON string of ProblemSet (with answers).
    /// Output: JSON string of StudentProblemSet (no answers, with encryptedAnswerKey embedded).
    #[wasm_bindgen]
    pub fn package_student_safe_wasm(problem_set_json: &str) -> Result<String, JsValue> {
        let ps: ProblemSet = serde_json::from_str(problem_set_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid problem set JSON: {}", e)))?;
        let sps = encode::package_student_safe(&ps);
        serde_json::to_string(&sps)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

#[cfg(feature = "wasm")]
pub use wasm_bindings::*;
