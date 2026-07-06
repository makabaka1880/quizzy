<script setup lang="ts">
import { ref, computed } from 'vue'
import { decodeAll, packageStudentSafe } from '@/utils/crypto'
import type { DecodedAll } from '@/utils/crypto'

type Tab = 'decode' | 'package'

const activeTab = ref<Tab>('decode')

// ---- Decode tab ----
const decodeBlob = ref('')
const decodeKey = ref('')
const decodeLoading = ref(false)
const decodeError = ref('')
const decodedResult = ref<DecodedAll | null>(null)
const gradedRows = computed(() => decodedResult.value ? gradeAll(decodedResult.value) : [])

async function handleDecode() {
  decodeError.value = ''
  decodedResult.value = null
  decodeLoading.value = true
  try {
    const result = await decodeAll(decodeBlob.value.trim(), decodeKey.value.trim())
    if (!result) {
      decodeError.value = 'Decoding failed. Check quiz key and blob.'
      decodeLoading.value = false
      return
    }
    decodedResult.value = result
  } catch (e) {
    decodeError.value = e instanceof Error ? e.message : 'Decoding failed.'
  }
  decodeLoading.value = false
}

// ---- Package tab ----
const packageJson = ref('')
const packageOutput = ref('')
const packageError = ref('')
const packageCopied = ref(false)

async function handlePackage() {
  packageError.value = ''
  packageOutput.value = ''
  try {
    JSON.parse(packageJson.value)
  } catch {
    packageError.value = 'Invalid JSON.'
    return
  }
  const result = await packageStudentSafe(packageJson.value)
  if (!result) {
    packageError.value = 'Packaging failed. Check the JSON structure.'
    return
  }
  packageOutput.value = result
}

async function handleCopyPackaged() {
  try {
    await navigator.clipboard.writeText(packageOutput.value)
    packageCopied.value = true
    setTimeout(() => (packageCopied.value = false), 2000)
  } catch {
    const ta = document.querySelector('.blob-out') as HTMLTextAreaElement
    if (ta) { ta.select(); document.execCommand('copy') }
    packageCopied.value = true
    setTimeout(() => (packageCopied.value = false), 2000)
  }
}

// ---- Grading helpers ----

type AnyEntry = DecodedAll['session']['entries'][number]
type AnyKey = DecodedAll['answerKey'][number]

interface GradedRow {
  problemId: number
  kind: string
  studentAnswer: string
  expectedAnswer: string
  isCorrect: boolean | null  // null = pending human
  pointsEarned: number
  pointsMax: number
  timeSpentMs: number
}

function gradeAll(decoded: DecodedAll): GradedRow[] {
  return decoded.session.entries.map((entry) => {
    const ake = decoded.answerKey[entry.problemId]
    const result = gradeOne(entry, ake)
    return {
      problemId: entry.problemId,
      kind: entry.answer.kind,
      studentAnswer: formatStudentAnswer(entry.answer),
      expectedAnswer: formatExpected(ake),
      isCorrect: result.isCorrect,
      pointsEarned: result.pointsAwarded,
      pointsMax: ake?.points ?? 0,
      timeSpentMs: entry.timeSpentMs,
    }
  })
}

function gradeOne(
  entry: AnyEntry,
  ake: AnyKey | undefined,
): { isCorrect: boolean | null; pointsAwarded: number } {
  if (!ake || entry.gradingStatus === 'pending-human') {
    return { isCorrect: null, pointsAwarded: 0 }
  }

  const answer = entry.answer as { kind: string; value: unknown }

  switch (ake.kind) {
    case 'mcq': {
      const correct = ake.correctIdx !== undefined && (answer.value as number) === ake.correctIdx
      return { isCorrect: correct, pointsAwarded: correct ? ake.points : 0 }
    }
    case 'mmcq': {
      if (answer.kind !== 'mmcq') return { isCorrect: false, pointsAwarded: 0 }
      const given = [...(answer.value as number[])].sort((a, b) => a - b)
      const expected = [...(ake.correctIdxs ?? [])].sort((a, b) => a - b)
      const correct =
        given.length === expected.length && given.every((v, i) => v === expected[i])
      return { isCorrect: correct, pointsAwarded: correct ? ake.points : 0 }
    }
    case 'frq': {
      if (answer.kind !== 'frq') return { isCorrect: false, pointsAwarded: 0 }
      const studentAns = (answer.value as string).trim()
      const pattern = ake.expectedPattern ?? ''
      const strategy = ake.strategy ?? 'contains'
      let correct = false
      switch (strategy) {
        case 'exact':
          correct = studentAns.toLowerCase() === pattern.trim().toLowerCase()
          break
        case 'contains':
          correct = studentAns.toLowerCase().includes(pattern.toLowerCase())
          break
        case 'regex':
          try {
            correct = new RegExp(pattern, 'i').test(studentAns)
          } catch { /* invalid regex, mark wrong */ }
          break
      }
      return { isCorrect: correct, pointsAwarded: correct ? ake.points : 0 }
    }
    default:
      return { isCorrect: null, pointsAwarded: 0 }
  }
}

function formatStudentAnswer(answer: unknown): string {
  const a = answer as { kind: string; value: unknown }
  if (a.kind === 'mcq') return `Option ${(a.value as number) + 1}`
  if (a.kind === 'mmcq') return `[${(a.value as number[]).map((v) => v + 1).join(', ')}]`
  if (a.kind === 'frq') {
    const s = a.value as string
    return s.length > 60 ? s.slice(0, 60) + '…' : s
  }
  if (a.kind === 'ffrq') return '(see below)'
  return JSON.stringify(a)
}

function formatExpected(ake: AnyKey | undefined): string {
  if (!ake) return '—'
  switch (ake.kind) {
    case 'mcq': return `Option ${ake.correctIdx! + 1}`
    case 'mmcq': return `[${ake.correctIdxs!.map((v) => v + 1).join(', ')}]`
    case 'frq': {
      const s = ake.expectedPattern!
      return s.length > 40 ? s.slice(0, 40) + '…' : s
    }
    case 'ffrq': return 'Human graded'
  }
}

function resultSymbol(row: GradedRow): string {
  if (row.isCorrect === null) return '—'
  return row.isCorrect ? '✓' : '✗'
}

function totalAuto(rows: GradedRow[]): { earned: number; possible: number } {
  let earned = 0
  let possible = 0
  for (const r of rows) {
    if (r.isCorrect !== null) {
      earned += r.pointsEarned
      possible += r.pointsMax
    }
  }
  return { earned, possible }
}

function pendingCount(rows: GradedRow[]): number {
  return rows.filter((r) => r.isCorrect === null).length
}

// FFRQ items for human grading section
interface FfrqItem {
  problemId: number
  points: number
  answer: string
  timeSpentMs: number
}

function getFfrqItems(decoded: DecodedAll): FfrqItem[] {
  return decoded.session.entries
    .filter((e) => e.gradingStatus === 'pending-human')
    .map((e) => ({
      problemId: e.problemId,
      points: decoded.answerKey[e.problemId]?.points ?? 0,
      answer: (e.answer as { kind: string; value: string }).value,
      timeSpentMs: e.timeSpentMs,
    }))
}
</script>

<template>
  <div class="teacher-panel">
    <h1>quizzy <span class="role-badge">teacher</span></h1>

    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'decode' }]" @click="activeTab = 'decode'">Decode Results</button>
      <button :class="['tab', { active: activeTab === 'package' }]" @click="activeTab = 'package'">Package Problems</button>
    </div>

    <!-- DECODE -->
    <div v-if="activeTab === 'decode'" class="tab-content">
      <p class="desc">Decode a student's submission and grade it using the quiz key.</p>

      <textarea v-model="decodeBlob" class="input-area" placeholder="Paste the student's encoded blob…" rows="6" spellcheck="false"></textarea>

      <input v-model="decodeKey" type="text" class="key-field" placeholder="Quiz key" spellcheck="false" />

      <div v-if="decodeError" class="err">{{ decodeError }}</div>

      <button class="btn-primary" :disabled="!decodeBlob.trim() || !decodeKey.trim() || decodeLoading" @click="handleDecode">
        {{ decodeLoading ? 'Decoding…' : 'Decode & Grade' }}
      </button>

      <div v-if="decodedResult" class="report">
        <h2>Grading Report</h2>
        <div class="report-meta">
          <div><span class="meta-label">Student</span> {{ decodedResult.session.studentId }}</div>
          <div><span class="meta-label">Completed</span> {{ new Date(decodedResult.session.timestamp).toLocaleString() }}</div>
          <div><span class="meta-label">Total time</span> {{ (decodedResult.session.entries.reduce((s,e) => s + e.timeSpentMs, 0) / 1000).toFixed(1) }}s</div>
        </div>

        <table class="report-table">
          <thead>
            <tr><th>#</th><th>Answer</th><th>Expected</th><th></th><th>Pts</th><th>Time</th></tr>
          </thead>
          <tbody>
            <tr
              v-for="row in gradedRows" :key="row.problemId"
              :class="row.isCorrect === null ? 'row-pending' : row.isCorrect ? 'row-correct' : 'row-wrong'"
            >
              <td>{{ row.problemId }}</td>
              <td class="answer-cell">{{ row.studentAnswer }}</td>
              <td class="expected-cell">{{ row.expectedAnswer }}</td>
              <td class="result-cell">{{ resultSymbol(row) }}</td>
              <td class="pts-cell">{{ row.isCorrect === null ? '—' : row.pointsEarned + ' / ' + row.pointsMax }}</td>
              <td>{{ (row.timeSpentMs / 1000).toFixed(1) }}s</td>
            </tr>
          </tbody>
        </table>

        <div class="report-score">
          Auto: <strong>{{ totalAuto(gradedRows).earned }} / {{ totalAuto(gradedRows).possible }}</strong>
          <template v-if="pendingCount(gradedRows) > 0">
            · <strong>{{ pendingCount(gradedRows) }}</strong> pending human grading
          </template>
        </div>

        <!-- FFRQ responses for human grading -->
        <div v-if="getFfrqItems(decodedResult).length > 0" class="ffrq-section">
          <h3>Pending Human Grading</h3>
          <div
            v-for="item in getFfrqItems(decodedResult)"
            :key="item.problemId"
            class="ffrq-item"
          >
            <div class="ffrq-header">
              <span class="ffrq-id">Question #{{ item.problemId }}</span>
              <span class="ffrq-meta">{{ item.points }} pts · {{ (item.timeSpentMs / 1000).toFixed(1) }}s</span>
            </div>
            <div class="ffrq-text">{{ item.answer }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- PACKAGE -->
    <div v-if="activeTab === 'package'" class="tab-content">
      <p class="desc">
        Paste a teacher problem set JSON (with answers + <code>quizKey</code>).
        Produces a <strong>student-safe</strong> JSON with answers stripped and the answer key encrypted inside.
      </p>

      <textarea v-model="packageJson" class="input-area" placeholder='{ "studentId": "...", "quizKey": "...", "problems": [...] }' rows="14" spellcheck="false"></textarea>

      <div v-if="packageError" class="err">{{ packageError }}</div>

      <button class="btn-primary" :disabled="!packageJson.trim()" @click="handlePackage">Package</button>

      <div v-if="packageOutput" class="output-section">
        <p class="output-hint">Share this JSON with the student. Answers are hidden.</p>
        <textarea class="blob-out" :value="packageOutput" readonly rows="18" spellcheck="false"></textarea>
        <button class="btn-copy" @click="handleCopyPackaged">{{ packageCopied ? '✓ Copied!' : 'Copy to Clipboard' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.teacher-panel {
  max-width: 780px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

h1 { font-size: 2rem; font-weight: 600; letter-spacing: -0.02em; }

.role-badge {
  font-size: 0.7rem; font-weight: 600; text-transform: uppercase;
  letter-spacing: 0.08em; color: #d97706; background: #fffbeb;
  padding: 0.2em 0.6em; border-radius: 4px; vertical-align: middle; margin-left: 0.4em;
}

.tabs {
  display: flex; gap: 0;
  border: 1px solid var(--c-border); border-radius: 8px; overflow: hidden;
}

.tab {
  padding: 0.6rem 1.5rem; font-size: 0.9rem; font-weight: 500;
  border: none; background: var(--c-surface); color: var(--c-muted);
  cursor: pointer; transition: all 0.15s;
}
.tab:first-child { border-right: 1px solid var(--c-border); }
.tab.active { background: var(--c-accent); color: #fff; }
.tab:not(.active):hover { background: #f8faff; }

.tab-content { width: 100%; display: flex; flex-direction: column; gap: 0.75rem; }

.desc { font-size: 0.85rem; color: var(--c-muted); text-align: center; line-height: 1.4; }
.desc code { background: #f1f5f9; padding: 0.1em 0.35em; border-radius: 3px; font-size: 0.9em; }

.input-area {
  width: 100%; padding: 0.8rem;
  font-family: var(--font-mono); font-size: 0.8rem; line-height: 1.5;
  border: 1px solid var(--c-border); border-radius: 8px;
  background: var(--c-surface); color: var(--c-text);
  resize: vertical; outline: none;
}
.input-area:focus { border-color: var(--c-accent); }

.key-field {
  width: 100%; padding: 0.55rem 0.8rem;
  font-family: var(--font-mono); font-size: 0.85rem;
  border: 1px solid var(--c-border); border-radius: 8px;
  background: var(--c-surface); color: var(--c-text); outline: none;
}
.key-field:focus { border-color: var(--c-accent); }
.key-field::placeholder { color: var(--c-muted); font-family: var(--font-sans); }

.err {
  width: 100%; padding: 0.6rem 0.9rem; background: #fef2f2; color: #991b1b;
  border: 1px solid #fecaca; border-radius: 6px; font-size: 0.82rem; line-height: 1.4;
}

.btn-primary {
  padding: 0.6rem 2.2rem; font-size: 0.95rem; font-weight: 500;
  border: none; border-radius: 8px; background: var(--c-accent); color: #fff;
  cursor: pointer; align-self: center; transition: opacity 0.15s;
}
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-primary:not(:disabled):hover { opacity: 0.85; }

.output-section { width: 100%; display: flex; flex-direction: column; gap: 0.5rem; }

.output-hint {
  font-size: 0.82rem; color: var(--c-muted); text-align: center;
}

.blob-out {
  width: 100%; padding: 0.7rem;
  font-family: var(--font-mono); font-size: 0.75rem; line-height: 1.4;
  border: 1px solid var(--c-border); border-radius: 8px;
  background: var(--c-surface); color: var(--c-text);
  resize: vertical; word-break: break-all; outline: none;
}

.btn-copy {
  padding: 0.45rem 1.2rem; font-size: 0.85rem; font-weight: 500;
  border: 1.5px solid var(--c-border); border-radius: 6px;
  background: var(--c-surface); color: var(--c-text);
  cursor: pointer; align-self: center; transition: all 0.15s;
}
.btn-copy:hover { border-color: var(--c-accent); color: var(--c-accent); }

.report { width: 100%; margin-top: 0.75rem; }
.report h2 { font-size: 1.2rem; margin-bottom: 0.75rem; }

.report-meta {
  display: flex; gap: 1.5rem; flex-wrap: wrap; margin-bottom: 1rem;
  font-size: 0.82rem; color: var(--c-muted);
}
.meta-label { font-weight: 600; color: var(--c-text); margin-right: 0.3rem; }

.report-table { width: 100%; border-collapse: collapse; font-size: 0.82rem; }
.report-table th {
  text-align: left; padding: 0.4rem 0.6rem; border-bottom: 2px solid var(--c-border);
  font-weight: 600; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.04em; color: var(--c-muted);
}
.report-table td { padding: 0.45rem 0.6rem; border-bottom: 1px solid var(--c-border); }

.kind-cell { font-size: 0.7rem; font-weight: 600; letter-spacing: 0.03em; }
.answer-cell { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.expected-cell { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--c-muted); }
.result-cell { font-weight: 600; font-size: 0.85rem; }
.pts-cell { font-variant-numeric: tabular-nums; }

.row-correct { background: #f0fdf4; }
.row-wrong { background: #fef2f2; }
.row-pending { background: #fffbeb; }

.report-score { margin-top: 0.75rem; font-size: 0.9rem; text-align: right; color: var(--c-muted); }
.report-score strong { color: var(--c-text); }

/* FFRQ section */
.ffrq-section {
  margin-top: 1.5rem;
  padding-top: 1.25rem;
  border-top: 1px solid var(--c-border);
}

.ffrq-section h3 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 1rem;
  color: #d97706;
}

.ffrq-item {
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 0.75rem;
}

.ffrq-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.ffrq-id {
  font-weight: 600;
  font-size: 0.85rem;
}

.ffrq-meta {
  font-size: 0.78rem;
  color: var(--c-muted);
}

.ffrq-text {
  font-size: 0.9rem;
  line-height: 1.6;
  white-space: pre-wrap;
  color: var(--c-text);
}
</style>
