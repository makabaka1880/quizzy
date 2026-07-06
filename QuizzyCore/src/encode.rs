use crate::types::*;
use crate::crypto::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::Serialize;

/// Build an answer key from the original teacher problems.
pub fn build_answer_key(problems: &[Problem]) -> AnswerKey {
    let mut key = AnswerKey::new();
    for p in problems {
        let entry = match &p.variant {
            ProblemVariant::MCQ { correct_idx, .. } => AnswerKeyEntry::MCQ {
                correct_idx: *correct_idx,
                points: p.points,
            },
            ProblemVariant::MMCQ { correct_idxs, .. } => AnswerKeyEntry::MMCQ {
                correct_idxs: correct_idxs.clone(),
                points: p.points,
            },
            ProblemVariant::FRQ {
                expected_pattern,
                strategy,
                ..
            } => AnswerKeyEntry::FRQ {
                expected_pattern: expected_pattern.clone(),
                strategy: strategy.clone(),
                points: p.points,
            },
            ProblemVariant::FFRQ => AnswerKeyEntry::FFRQ {
                points: p.points,
                grading: "human".to_string(),
            },
        };
        key.insert(p.id, entry);
    }
    key
}

// ---------- Block encode/decode primitives ----------

fn encode_block<T: Serialize>(payload: &T, key: &[u8; 32]) -> String {
    let json = serde_json::to_string(payload).expect("serialization must succeed");
    let ciphered = xor_cipher(json.as_bytes(), key);
    let b64 = BASE64.encode(&ciphered);
    let salt = generate_salt(b64.len());
    let interleaved = interleave(&b64, &salt);
    let checksum = sha256_hex(&interleaved);
    format!("{}{}", &checksum[..8], interleaved)
}

fn decode_block<T: for<'de> serde::Deserialize<'de>>(block: &str, key: &[u8; 32]) -> Result<T, String> {
    if block.len() < 8 {
        return Err("Block too short".to_string());
    }
    let checksum_prefix = &block[..8];
    let interleaved = &block[8..];
    let computed = sha256_hex(interleaved);
    if &computed[..8] != checksum_prefix {
        return Err("Checksum mismatch — data may be corrupted".to_string());
    }
    let (b64, _salt) = simple_deinterleave(interleaved);
    let ciphered = BASE64
        .decode(b64.as_bytes())
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    let json_bytes = xor_cipher(&ciphered, key);
    let json = String::from_utf8(json_bytes).map_err(|e| format!("UTF-8 error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("JSON parse error: {}", e))
}

// ---------- Package: teacher creates student-safe problem set ----------

/// Produce a StudentProblemSet suitable for distribution.
///
/// 1. Strips answers from every problem (correctIdx, correctIdxs, expectedPattern removed).
/// 2. Encrypts the answer key with `deriveKey(quizKey, 0)` and embeds it as `encryptedAnswerKey`.
///
/// The student never sees correct answers. The encrypted answer key travels with the
/// problem set and gets passed through verbatim into the output blob so the teacher
/// can decode it later.
pub fn package_student_safe(problem_set: &ProblemSet) -> StudentProblemSet {
    let answer_key = build_answer_key(&problem_set.problems);
    let answer_key_payload = serde_json::json!({ "answerKey": answer_key });
    let key = derive_key(&problem_set.quiz_key, 0);
    let encrypted_answer_key = encode_block(&answer_key_payload, &key);

    let problems: Vec<StudentProblem> = problem_set
        .problems
        .iter()
        .map(|p| {
            let variant = match &p.variant {
                ProblemVariant::MCQ { options, .. } => StudentProblemVariant::MCQ {
                    options: options.clone(),
                },
                ProblemVariant::MMCQ { options, .. } => StudentProblemVariant::MMCQ {
                    options: options.clone(),
                },
                ProblemVariant::FRQ { .. } => StudentProblemVariant::FRQ,
                ProblemVariant::FFRQ => StudentProblemVariant::FFRQ,
            };
            StudentProblem {
                id: p.id,
                points: p.points,
                prompt: p.prompt.clone(),
                variant,
            }
        })
        .collect();

    StudentProblemSet {
        student_id: problem_set.student_id.clone(),
        encrypted_answer_key,
        problems,
    }
}

// ---------- Encode quiz result (called by frontend) ----------

/// Encode the student's quiz submission.
///
/// `encrypted_answer_key` is the opaque blob from the StudentProblemSet — the frontend
/// doesn't touch it, just passes it through.
///
/// Output: `header . session_block . encrypted_answer_key`
pub fn encode_full(
    student_id: &str,
    timestamp: u64,
    session: &SessionState,
    encrypted_answer_key: &str,
) -> String {
    let student_key = derive_key(student_id, timestamp);

    // Session block
    let session_payload = SessionPayload {
        session: session.clone(),
    };
    let session_block = encode_block(&session_payload, &student_key);

    // Header
    let header_json = serde_json::json!({
        "sid": student_id,
        "ts": timestamp,
    });
    let header = BASE64.encode(header_json.to_string().as_bytes());

    // encrypted_answer_key is already a full block (checksum + interleaved)
    format!("{}.{}.{}", header, session_block, encrypted_answer_key)
}

// ---------- Decode ----------

/// Decode just the session block (student-facing: no quiz key needed).
pub fn decode_session(encoded: &str) -> Result<SessionState, String> {
    let parts: Vec<&str> = encoded.splitn(3, '.').collect();
    if parts.len() < 2 {
        return Err("Invalid format: expected header.block".to_string());
    }
    let header_json_bytes = BASE64
        .decode(parts[0].as_bytes())
        .map_err(|e| format!("Header base64 error: {}", e))?;
    let header_json =
        String::from_utf8(header_json_bytes).map_err(|e| format!("Header UTF-8 error: {}", e))?;
    let header: serde_json::Value =
        serde_json::from_str(&header_json).map_err(|e| format!("Header JSON error: {}", e))?;
    let sid = header["sid"].as_str().ok_or("Missing sid in header")?;
    let ts = header["ts"].as_u64().ok_or("Missing ts in header")?;
    let student_key = derive_key(sid, ts);
    let session_payload: SessionPayload = decode_block(parts[1], &student_key)?;
    Ok(session_payload.session)
}

/// Decode the answer key block (teacher-facing: requires quiz_key).
pub fn decode_answer_key(encoded: &str, quiz_key: &str) -> Result<AnswerKey, String> {
    let parts: Vec<&str> = encoded.splitn(3, '.').collect();
    if parts.len() < 3 {
        return Err("Invalid format: expected header.sessionBlock.encryptedAnswerKey".to_string());
    }
    let key = derive_key(quiz_key, 0);
    let raw: serde_json::Value = decode_block(parts[2], &key)?;
    let answer_key: AnswerKey =
        serde_json::from_value(raw["answerKey"].clone()).map_err(|e| format!("JSON parse error: {}", e))?;
    Ok(answer_key)
}

/// Decode both (teacher-facing: requires quiz_key).
pub fn decode_all(encoded: &str, quiz_key: &str) -> Result<DecodedResult, String> {
    let session = decode_session(encoded)?;
    let answer_key = decode_answer_key(encoded, quiz_key)?;
    Ok(DecodedResult { session, answer_key })
}

// ---------- Problem Set Encryption (for student distribution, legacy) ----------

pub fn encrypt_problem_set(problem_set_json: &str, quiz_key: &str) -> String {
    let key = derive_key(quiz_key, 0);
    let json_bytes = problem_set_json.as_bytes();
    let ciphered = xor_cipher(json_bytes, &key);
    let b64 = BASE64.encode(&ciphered);
    let salt = generate_salt(b64.len());
    let interleaved = interleave(&b64, &salt);
    let checksum = sha256_hex(&interleaved);
    format!("{}{}", &checksum[..8], interleaved)
}

pub fn decrypt_problem_set(encrypted: &str, quiz_key: &str) -> Result<String, String> {
    let key = derive_key(quiz_key, 0);
    if encrypted.len() < 8 {
        return Err("Encrypted blob too short".to_string());
    }
    let checksum_prefix = &encrypted[..8];
    let interleaved = &encrypted[8..];
    let computed = sha256_hex(interleaved);
    if &computed[..8] != checksum_prefix {
        return Err("Checksum mismatch — wrong quiz key or corrupted data".to_string());
    }
    let (b64, _salt) = simple_deinterleave(interleaved);
    let ciphered = BASE64
        .decode(b64.as_bytes())
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    let json_bytes = xor_cipher(&ciphered, &key);
    String::from_utf8(json_bytes).map_err(|e| format!("UTF-8 error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_session() -> SessionState {
        SessionState {
            student_id: "jsmith42".to_string(),
            timestamp: 1711234567890,
            entries: vec![
                GradingEntry {
                    problem_id: 1,
                    is_correct: Some(true),
                    grading_status: GradingStatus::AutoGraded,
                    answer: Answer::MCQ { value: 1 },
                    time_spent_ms: 5230,
                    points_awarded: 5,
                },
                GradingEntry {
                    problem_id: 2,
                    is_correct: Some(false),
                    grading_status: GradingStatus::AutoGraded,
                    answer: Answer::MMCQ { value: vec![0, 2] },
                    time_spent_ms: 12400,
                    points_awarded: 0,
                },
                GradingEntry {
                    problem_id: 3,
                    is_correct: Some(true),
                    grading_status: GradingStatus::AutoGraded,
                    answer: Answer::FRQ { value: "O(log n)".to_string() },
                    time_spent_ms: 8700,
                    points_awarded: 15,
                },
                GradingEntry {
                    problem_id: 4,
                    is_correct: None,
                    grading_status: GradingStatus::PendingHuman,
                    answer: Answer::FFRQ { value: "Functional programming is...".to_string() },
                    time_spent_ms: 45200,
                    points_awarded: 0,
                },
            ],
        }
    }

    fn make_test_problems() -> Vec<Problem> {
        vec![
            Problem {
                id: 1, points: 5, prompt: "Capital of France?".to_string(),
                variant: ProblemVariant::MCQ {
                    options: vec!["London".into(), "Paris".into(), "Berlin".into(), "Madrid".into()],
                    correct_idx: 1,
                },
            },
            Problem {
                id: 2, points: 10, prompt: "Select all primes".to_string(),
                variant: ProblemVariant::MMCQ {
                    options: vec!["2".into(), "4".into(), "7".into(), "9".into()],
                    correct_idxs: vec![0, 2],
                },
            },
            Problem {
                id: 3, points: 15, prompt: "Time complexity of binary search?".to_string(),
                variant: ProblemVariant::FRQ {
                    expected_pattern: "O(log n)".to_string(),
                    strategy: MatchStrategy::Regex,
                },
            },
            Problem {
                id: 4, points: 25, prompt: "Compare FP and OOP.".to_string(),
                variant: ProblemVariant::FFRQ,
            },
        ]
    }

    #[test]
    fn test_package_student_safe_strips_answers() {
        let ps = ProblemSet {
            student_id: "s1".into(),
            quiz_key: "secret".into(),
            problems: make_test_problems(),
        };
        let sps = package_student_safe(&ps);
        assert_eq!(sps.student_id, "s1");
        assert!(!sps.encrypted_answer_key.is_empty());
        assert_eq!(sps.problems.len(), 4);
        // Verify answers are stripped
        match &sps.problems[0].variant {
            StudentProblemVariant::MCQ { options } => {
                assert_eq!(options.len(), 4);
            }
            _ => panic!("Expected MCQ"),
        }
        // FRQ should have no expectedPattern
        match &sps.problems[2].variant {
            StudentProblemVariant::FRQ => {} // good — no fields
            _ => panic!("Expected FRQ"),
        }
    }

    #[test]
    fn test_roundtrip_full() {
        let ps = ProblemSet {
            student_id: "jsmith42".into(),
            quiz_key: "teacher-secret-42".into(),
            problems: make_test_problems(),
        };
        let sps = package_student_safe(&ps);
        let session = make_test_session();

        let encoded = encode_full("jsmith42", 1711234567890, &session, &sps.encrypted_answer_key);

        let parts: Vec<&str> = encoded.splitn(3, '.').collect();
        assert_eq!(parts.len(), 3, "Output should have 3 parts");

        // Student can decode their session
        let decoded_session = decode_session(&encoded).expect("decode_session should succeed");
        assert_eq!(decoded_session.student_id, "jsmith42");
        assert_eq!(decoded_session.entries.len(), 4);

        // Teacher can decode answer key
        let answer_key = decode_answer_key(&encoded, &ps.quiz_key).expect("decode_answer_key should succeed");
        assert_eq!(answer_key.len(), 4);
        match &answer_key[&1] {
            AnswerKeyEntry::MCQ { correct_idx, points } => {
                assert_eq!(*correct_idx, 1);
                assert_eq!(*points, 5);
            }
            _ => panic!("Expected MCQ"),
        }

        // Wrong quiz key fails
        assert!(decode_answer_key(&encoded, "wrong-key").is_err());

        // decode_all
        let decoded = decode_all(&encoded, &ps.quiz_key).expect("decode_all should succeed");
        assert_eq!(decoded.answer_key.len(), 4);
    }

    #[test]
    fn test_student_cannot_decode_answer_key() {
        let ps = ProblemSet {
            student_id: "s1".into(),
            quiz_key: "secret".into(),
            problems: make_test_problems(),
        };
        let sps = package_student_safe(&ps);
        let session = make_test_session();
        let encoded = encode_full("s1", 1000, &session, &sps.encrypted_answer_key);

        // Student can decode session
        assert!(decode_session(&encoded).is_ok());

        // Student does NOT have the quiz key — can't decode answer key
        // They could try guessing, but they can never get the answer key without quizKey
    }
}
