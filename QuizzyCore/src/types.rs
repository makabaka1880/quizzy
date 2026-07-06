use serde::{Deserialize, Serialize};

// ---------- Teacher Problem Set (has answers) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSet {
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "quizKey")]
    pub quiz_key: String,
    pub problems: Vec<Problem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: u64,
    pub points: u64,
    pub prompt: String,
    pub variant: ProblemVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ProblemVariant {
    #[serde(rename = "mcq")]
    MCQ {
        options: Vec<String>,
        #[serde(rename = "correctIdx")]
        correct_idx: usize,
    },
    #[serde(rename = "mmcq")]
    MMCQ {
        options: Vec<String>,
        #[serde(rename = "correctIdxs")]
        correct_idxs: Vec<usize>,
    },
    #[serde(rename = "frq")]
    FRQ {
        #[serde(rename = "expectedPattern")]
        expected_pattern: String,
        strategy: MatchStrategy,
    },
    #[serde(rename = "ffrq")]
    FFRQ,
}

// ---------- Student Problem Set (no answers) ----------

/// Given to the student. Contains no correct answers — just questions.
/// The `encrypted_answer_key` is an opaque blob the frontend passes through
/// unchanged into the output. It can only be decrypted by the teacher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentProblemSet {
    #[serde(rename = "studentId")]
    pub student_id: String,
    /// Pre-encrypted answer key block (encrypted with deriveKey(quizKey, 0)).
    /// The frontend includes this verbatim in the output blob.
    #[serde(rename = "encryptedAnswerKey")]
    pub encrypted_answer_key: String,
    pub problems: Vec<StudentProblem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentProblem {
    pub id: u64,
    pub points: u64,
    pub prompt: String,
    pub variant: StudentProblemVariant,
}

/// Student-facing variants have NO correct answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StudentProblemVariant {
    #[serde(rename = "mcq")]
    MCQ { options: Vec<String> },
    #[serde(rename = "mmcq")]
    MMCQ { options: Vec<String> },
    #[serde(rename = "frq")]
    FRQ,
    #[serde(rename = "ffrq")]
    FFRQ,
}

// ---------- Shared enums ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchStrategy {
    Exact,
    Contains,
    Regex,
}

// ---------- Answers ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Answer {
    #[serde(rename = "mcq")]
    MCQ { value: usize },
    #[serde(rename = "mmcq")]
    MMCQ { value: Vec<usize> },
    #[serde(rename = "frq")]
    FRQ { value: String },
    #[serde(rename = "ffrq")]
    FFRQ { value: String },
}

// ---------- Session ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    #[serde(rename = "studentId")]
    pub student_id: String,
    pub timestamp: u64,
    pub entries: Vec<GradingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingEntry {
    #[serde(rename = "problemId")]
    pub problem_id: u64,
    #[serde(rename = "isCorrect")]
    pub is_correct: Option<bool>,
    #[serde(rename = "gradingStatus")]
    pub grading_status: GradingStatus,
    pub answer: Answer,
    #[serde(rename = "timeSpentMs")]
    pub time_spent_ms: u64,
    #[serde(rename = "pointsAwarded")]
    pub points_awarded: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GradingStatus {
    AutoGraded,
    #[serde(rename = "pending-human")]
    PendingHuman,
}

// ---------- Answer Key ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AnswerKeyEntry {
    #[serde(rename = "mcq")]
    MCQ {
        #[serde(rename = "correctIdx")]
        correct_idx: usize,
        points: u64,
    },
    #[serde(rename = "mmcq")]
    MMCQ {
        #[serde(rename = "correctIdxs")]
        correct_idxs: Vec<usize>,
        points: u64,
    },
    #[serde(rename = "frq")]
    FRQ {
        #[serde(rename = "expectedPattern")]
        expected_pattern: String,
        strategy: MatchStrategy,
        points: u64,
    },
    #[serde(rename = "ffrq")]
    FFRQ { points: u64, grading: String },
}

pub type AnswerKey = std::collections::BTreeMap<u64, AnswerKeyEntry>;

// ---------- Inner payloads for encoding ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionPayload {
    pub session: SessionState,
}

/// The full decoded result returned by decode_all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedResult {
    pub session: SessionState,
    #[serde(rename = "answerKey")]
    pub answer_key: AnswerKey,
}
