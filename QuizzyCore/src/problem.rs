use crate::types::*;

/// Create a new ProblemSet from parts.
pub fn create_problem_set(student_id: &str, quiz_key: &str, problems: Vec<Problem>) -> ProblemSet {
    ProblemSet {
        student_id: student_id.to_string(),
        quiz_key: quiz_key.to_string(),
        problems,
    }
}

/// Build a multiple-choice problem.
pub fn mcq(id: u64, points: u64, prompt: &str, options: Vec<&str>, correct_idx: usize) -> Problem {
    Problem {
        id,
        points,
        prompt: prompt.to_string(),
        variant: ProblemVariant::MCQ {
            options: options.iter().map(|s| s.to_string()).collect(),
            correct_idx,
        },
    }
}

/// Build a multi-select multiple-choice problem.
pub fn mmcq(
    id: u64,
    points: u64,
    prompt: &str,
    options: Vec<&str>,
    correct_idxs: Vec<usize>,
) -> Problem {
    Problem {
        id,
        points,
        prompt: prompt.to_string(),
        variant: ProblemVariant::MMCQ {
            options: options.iter().map(|s| s.to_string()).collect(),
            correct_idxs,
        },
    }
}

/// Build a free-response (auto-graded) problem.
pub fn frq(
    id: u64,
    points: u64,
    prompt: &str,
    expected_pattern: &str,
    strategy: MatchStrategy,
) -> Problem {
    Problem {
        id,
        points,
        prompt: prompt.to_string(),
        variant: ProblemVariant::FRQ {
            expected_pattern: expected_pattern.to_string(),
            strategy,
        },
    }
}

/// Build a free-form response (human-graded) problem.
pub fn ffrq(id: u64, points: u64, prompt: &str) -> Problem {
    Problem {
        id,
        points,
        prompt: prompt.to_string(),
        variant: ProblemVariant::FFRQ,
    }
}

/// Serialize a ProblemSet to pretty JSON.
pub fn to_json(problem_set: &ProblemSet) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(problem_set)
}

/// Parse a ProblemSet from JSON.
pub fn from_json(json: &str) -> Result<ProblemSet, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_serialize() {
        let problems = vec![
            mcq(1, 5, "Q1?", vec!["A", "B", "C", "D"], 1),
            mmcq(2, 10, "Q2?", vec!["X", "Y", "Z"], vec![0, 2]),
            frq(3, 15, "Q3?", "hello", MatchStrategy::Contains),
            ffrq(4, 25, "Essay?"),
        ];
        let ps = create_problem_set("student1", "quiz-key-123", problems);
        let json = to_json(&ps).expect("serialization should work");
        let ps2 = from_json(&json).expect("deserialization should work");
        assert_eq!(ps.student_id, ps2.student_id);
        assert_eq!(ps.quiz_key, ps2.quiz_key);
        assert_eq!(ps.problems.len(), ps2.problems.len());
    }
}
