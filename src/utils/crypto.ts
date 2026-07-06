import initWasm, {
  encode_quiz_result,
  decode_my_session,
  decode_all_wasm,
  package_student_safe_wasm,
} from '@/wasm/quizzy_core'
import type { SessionState, GradingEntry } from '@/types/quiz'

let initialized = false

async function ensureInit(): Promise<void> {
  if (!initialized) {
    await initWasm()
    initialized = true
  }
}

// ---------- Student-facing ----------

/**
 * Encode the quiz submission.
 * `encrypted_answer_key` is the opaque blob from the StudentProblemSet — passed through verbatim.
 */
export async function encodeSubmission(
  studentId: string,
  timestamp: number,
  entries: GradingEntry[],
  encryptedAnswerKey: string,
): Promise<string> {
  await ensureInit()
  const entriesJson = JSON.stringify(entries)
  return encode_quiz_result(studentId, timestamp, entriesJson, encryptedAnswerKey)
}

/**
 * Student-facing: decode your own session (answers + timing) from a result blob.
 */
export async function decodeMySession(
  encoded: string,
): Promise<SessionState | null> {
  await ensureInit()
  try {
    const json = decode_my_session(encoded)
    return JSON.parse(json) as SessionState
  } catch {
    return null
  }
}

// ---------- Teacher-facing ----------

export interface DecodedAll {
  session: SessionState
  answerKey: Record<number, {
    kind: 'mcq' | 'mmcq' | 'frq' | 'ffrq'
    correctIdx?: number
    correctIdxs?: number[]
    expectedPattern?: string
    strategy?: string
    points: number
    grading?: string
  }>
}

/**
 * Teacher: decode both session + answer key from a student's result blob.
 */
export async function decodeAll(
  encoded: string,
  quizKey: string,
): Promise<DecodedAll | null> {
  await ensureInit()
  try {
    const json = decode_all_wasm(encoded, quizKey)
    return JSON.parse(json) as DecodedAll
  } catch {
    return null
  }
}

/**
 * Teacher: produce a student-safe problem set from a full teacher problem set JSON.
 */
export async function packageStudentSafe(
  problemSetJson: string,
): Promise<string | null> {
  await ensureInit()
  try {
    return package_student_safe_wasm(problemSetJson)
  } catch {
    return null
  }
}
