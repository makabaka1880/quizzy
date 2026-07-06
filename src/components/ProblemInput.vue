<script setup lang="ts">
import { ref } from 'vue'
import type { StudentProblemSet } from '@/types/quiz'
import FileInput from '@/components/FileInput.vue'

const emit = defineEmits<{
  start: [problems: StudentProblemSet]
}>()

const rawInput = ref('')
const error = ref('')

function handleStart() {
  error.value = ''

  let parsed: unknown
  try {
    parsed = JSON.parse(rawInput.value)
  } catch {
    error.value = 'Invalid JSON. Please check that you pasted the problem set correctly.'
    return
  }

  if (!isStudentProblemSet(parsed)) {
    error.value =
      'Invalid format. Expected a student-safe problem set JSON with "studentId", "encryptedAnswerKey", and "problems" fields.'
    return
  }

  emit('start', parsed)
}

function isStudentProblemSet(obj: unknown): obj is StudentProblemSet {
  if (!obj || typeof obj !== 'object') return false
  const o = obj as Record<string, unknown>
  if (typeof o.studentId !== 'string') return false
  if (typeof o.encryptedAnswerKey !== 'string') return false
  if (!Array.isArray(o.problems)) return false
  for (const p of o.problems) {
    if (!p || typeof p !== 'object') return false
    const prob = p as Record<string, unknown>
    if (typeof prob.id !== 'number') return false
    if (typeof prob.points !== 'number') return false
    if (typeof prob.prompt !== 'string') return false
    if (!prob.variant || typeof prob.variant !== 'object') return false
    const v = prob.variant as Record<string, unknown>
    if (typeof v.kind !== 'string') return false
    if (!['mcq', 'mmcq', 'frq', 'ffrq'].includes(v.kind)) return false
    if ((v.kind === 'mcq' || v.kind === 'mmcq') && !Array.isArray(v.options)) return false
  }
  return true
}
</script>

<template>
  <div class="input-view">
    <h1>quizzy</h1>
    <p class="subtitle">Paste the problem set from your instructor to begin.</p>

    <textarea
      v-model="rawInput"
      class="json-input"
      placeholder='{ "studentId": "...", "encryptedAnswerKey": "...", "problems": [...] }'
      rows="18"
      spellcheck="false"
    ></textarea>

    <FileInput @loaded="rawInput = $event" />

    <div v-if="error" class="error">{{ error }}</div>

    <button class="btn-start" @click="handleStart" :disabled="!rawInput.trim()">
      Start Quiz
    </button>
  </div>
</template>

<style scoped>
.input-view {
  max-width: 640px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

h1 {
  font-size: 2rem;
  font-weight: 600;
  letter-spacing: -0.02em;
}

.subtitle {
  color: var(--c-muted);
  font-size: 0.9rem;
  text-align: center;
  line-height: 1.4;
}

.json-input {
  width: 100%;
  padding: 1rem;
  font-family: var(--font-mono);
  font-size: 0.82rem;
  line-height: 1.5;
  border: 1px solid var(--c-border);
  border-radius: 8px;
  background: var(--c-surface);
  color: var(--c-text);
  resize: vertical;
  outline: none;
  transition: border-color 0.15s;
}

.json-input:focus { border-color: var(--c-accent); }

.error {
  width: 100%;
  padding: 0.75rem 1rem;
  background: #fef2f2;
  color: #991b1b;
  border: 1px solid #fecaca;
  border-radius: 6px;
  font-size: 0.85rem;
  line-height: 1.4;
}

.btn-start {
  padding: 0.7rem 2.5rem;
  font-size: 1rem;
  font-weight: 500;
  border: none;
  border-radius: 8px;
  background: var(--c-accent);
  color: #fff;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-start:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-start:not(:disabled):hover { opacity: 0.85; }
</style>
