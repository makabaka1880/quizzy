// ---------- Student-safe problem set (no answers) ----------

export type StudentProblemVariant =
  | { kind: 'mcq';  options: string[] }
  | { kind: 'mmcq'; options: string[] }
  | { kind: 'frq' }
  | { kind: 'ffrq' }

export interface StudentProblem {
  id: number
  points: number
  prompt: string
  variant: StudentProblemVariant
}

export interface StudentProblemSet {
  studentId: string
  encryptedAnswerKey: string
  problems: StudentProblem[]
}

// ---------- Answers ----------

export type Answer =
  | { kind: 'mcq';  value: number }
  | { kind: 'mmcq'; value: number[] }
  | { kind: 'frq';  value: string }
  | { kind: 'ffrq'; value: string }

// ---------- Session ----------

export type GradingStatus = 'auto-graded' | 'pending-human'

export interface GradingEntry {
  problemId: number
  isCorrect: boolean | null
  gradingStatus: GradingStatus
  answer: Answer
  timeSpentMs: number
  pointsAwarded: number
}

export interface SessionState {
  studentId: string
  timestamp: number
  entries: GradingEntry[]
}
