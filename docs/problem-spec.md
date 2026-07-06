# Problem Set Specification v1.0

## Overview

A **Problem Set** is a JSON document that defines a quiz. It contains the student's identity, a list of problems, and for each problem: its point value, a Markdown prompt, and a variant describing how the student answers and (for auto-graded variants) how the answer is evaluated.

## Top-Level Structure

```jsonc
{
  "studentId": string,    // Unique identifier for the student
  "quizKey":   string,    // Instructor-provided key; encrypts the answer key in the output
  "problems": Problem[]   // Ordered list of problems (presented in this order)
}
```

## The `Problem` Object

| Field     | Type           | Required | Description                                      |
|-----------|----------------|----------|--------------------------------------------------|
| `id`      | `number`       | Yes      | Unique problem identifier within this set         |
| `points`  | `number`       | Yes      | Maximum points awarded for a correct answer       |
| `prompt`  | `string`       | Yes      | Question text in Markdown                         |
| `variant` | `ProblemVariant` | Yes    | The answer/interaction model for this question    |

## `ProblemVariant` — The Four Kinds

A variant is a tagged union discriminated by the `kind` field. Exactly four kinds exist.

---

### 1. `mcq` — Multiple Choice (Single Answer)

The student selects exactly one option.

```jsonc
{
  "kind": "mcq",
  "options": ["Option A text", "Option B text", "Option C text", "Option D text"],
  "correctIdx": 1    // Zero-based index of the correct option
}
```

| Field         | Type       | Required | Description                               |
|---------------|------------|----------|-------------------------------------------|
| `kind`        | `"mcq"`    | Yes      | Discriminant                              |
| `options`     | `string[]` | Yes      | Ordered list of choice texts              |
| `correctIdx`  | `number`   | Yes      | Zero-based index of the single correct answer |

**Grading:** Student's selected index must equal `correctIdx` exactly. Full points or zero.

---

### 2. `mmcq` — Multiple Multiple-Choice (Select All That Apply)

The student selects zero or more options.

```jsonc
{
  "kind": "mmcq",
  "options": ["2", "4", "6", "7"],
  "correctIdxs": [0, 3]   // Zero-based indices of all correct options
}
```

| Field          | Type       | Required | Description                                     |
|----------------|------------|----------|-------------------------------------------------|
| `kind`         | `"mmcq"`   | Yes      | Discriminant                                    |
| `options`      | `string[]` | Yes      | Ordered list of choice texts                    |
| `correctIdxs`  | `number[]` | Yes      | Zero-based indices of all correct options       |

**Grading:** Student's selected set must exactly match `correctIdxs` (order-independent, set equality). Partial credit is NOT awarded — all or nothing.

---

### 3. `frq` — Free Response Question (Auto-Graded)

The student types a short text answer, which is matched against a pattern.

```jsonc
{
  "kind": "frq",
  "expectedPattern": "polymorphism",
  "strategy": "contains"       // "exact" | "contains" | "regex"
}
```

| Field             | Type      | Required | Description                                           |
|-------------------|-----------|----------|-------------------------------------------------------|
| `kind`            | `"frq"`   | Yes      | Discriminant                                          |
| `expectedPattern` | `string`  | Yes      | The pattern to match against                          |
| `strategy`        | `string`  | Yes      | Matching strategy: `"exact"`, `"contains"`, or `"regex"` |

#### Matching Strategies

| Strategy     | Behavior                                                                 |
|--------------|--------------------------------------------------------------------------|
| `exact`      | Student answer trimmed & lowercased must equal pattern trimmed & lowercased |
| `contains`   | Student answer lowercased must contain pattern lowercased as a substring   |
| `regex`      | Pattern is compiled as a case-insensitive `RegExp` and tested against answer |

If the regex is malformed, grading returns **incorrect** (no error is surfaced to the student).

**Grading:** Match → full points. No match → zero points.

---

### 4. `ffrq` — Free-Form Response Question (Human-Graded)

The student writes a long-form response. This question is **never auto-graded** — the answer is preserved verbatim in the encoded output for human evaluation.

```jsonc
{
  "kind": "ffrq"
}
```

| Field  | Type     | Required | Description     |
|--------|----------|----------|-----------------|
| `kind` | `"ffrq"` | Yes     | Discriminant    |

No other fields are needed. The `points` field on the parent `Problem` indicates the maximum possible score, but the student always receives **0 points** in the auto-graded output — a human grader must override this.

**Grading:** Always `pending-human`. `isCorrect` is `null`. `pointsAwarded` is `0`.

---

## Complete Example

```json
{
  "studentId": "jsmith42",
  "quizKey": "teacher-secret-42",
  "problems": [
    {
      "id": 1,
      "points": 5,
      "prompt": "What is the capital of France?",
      "variant": {
        "kind": "mcq",
        "options": ["London", "Paris", "Berlin", "Madrid"],
        "correctIdx": 1
      }
    },
    {
      "id": 2,
      "points": 10,
      "prompt": "Select all numbers that are **prime**:",
      "variant": {
        "kind": "mmcq",
        "options": ["2", "4", "7", "9", "11"],
        "correctIdxs": [0, 2, 4]
      }
    },
    {
      "id": 3,
      "points": 15,
      "prompt": "## Algorithms\n\nWhat is the time complexity of **binary search**? Explain briefly.",
      "variant": {
        "kind": "frq",
        "expectedPattern": "O\\(log n\\)",
        "strategy": "regex"
      }
    },
    {
      "id": 4,
      "points": 25,
      "prompt": "## Essay\n\nCompare and contrast **functional programming** with **object-oriented programming**. Use specific examples from languages you have studied.",
      "variant": {
        "kind": "ffrq"
      }
    }
  ]
}
```

## Markdown in Prompts

The `prompt` field supports a lightweight Markdown subset:

- `**bold**` / `__bold__`
- `*italic*` / `_italic_`
- `` `inline code` ``
- ` ``` ` fenced code blocks (optional language tag)
- `[link text](url)`
- `#`–`####` headings
- `- ` / `* ` unordered list items
- `---` horizontal rules

HTML in prompts is escaped. Images (`![alt](url)`) are rendered.
