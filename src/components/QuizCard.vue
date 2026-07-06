<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { StudentProblem, Answer, GradingEntry } from '@/types/quiz'
import { renderMarkdown } from '@/utils/markdown'

const props = defineProps<{
  problem: StudentProblem
  previousEntry: GradingEntry | null
}>()

const emit = defineEmits<{
  submit: [entry: GradingEntry]
}>()

const startTime = performance.now()

// ---- MCQ state ----
const selectedMcq = ref<number | null>(null)

// ---- MMCQ state ----
const selectedMmcq = ref<Set<number>>(new Set())
function toggleMmcq(idx: number) {
  const next = new Set(selectedMmcq.value)
  if (next.has(idx)) { next.delete(idx) } else { next.add(idx) }
  selectedMmcq.value = next
}

// ---- FRQ / FFRQ state ----
const frqText = ref('')
const ffrqText = ref('')

// ---- Submission ----
const submitted = ref(false)
const elapsedSec = ref(0)

watch(() => props.problem.id, () => {
  selectedMcq.value = null
  selectedMmcq.value = new Set()
  frqText.value = ''
  ffrqText.value = ''
  submitted.value = false
})

const canSubmit = computed(() => {
  if (submitted.value) return false
  const v = props.problem.variant
  switch (v.kind) {
    case 'mcq': return selectedMcq.value !== null
    case 'mmcq': return selectedMmcq.value.size > 0
    case 'frq': return frqText.value.trim().length > 0
    case 'ffrq': return ffrqText.value.trim().length > 0
  }
})

function buildAnswer(): Answer | null {
  const v = props.problem.variant
  switch (v.kind) {
    case 'mcq':
      return selectedMcq.value !== null ? { kind: 'mcq', value: selectedMcq.value } : null
    case 'mmcq':
      return selectedMmcq.value.size > 0
        ? { kind: 'mmcq', value: [...selectedMmcq.value].sort((a, b) => a - b) }
        : null
    case 'frq':
      return frqText.value.trim() ? { kind: 'frq', value: frqText.value.trim() } : null
    case 'ffrq':
      return ffrqText.value.trim() ? { kind: 'ffrq', value: ffrqText.value.trim() } : null
  }
}

function handleSubmit() {
  if (submitted.value) return
  const answer = buildAnswer()
  if (!answer) return

  const elapsed = performance.now() - startTime
  const isFfrq = props.problem.variant.kind === 'ffrq'
  const entry: GradingEntry = {
    problemId: props.problem.id,
    isCorrect: null,
    gradingStatus: isFfrq ? 'pending-human' : 'auto-graded',
    answer,
    timeSpentMs: Math.round(elapsed),
    pointsAwarded: 0,
  }
  elapsedSec.value = Math.round(elapsed) / 1000
  submitted.value = true
  emit('submit', entry)
}

const promptHtml = computed(() => renderMarkdown(props.problem.prompt))
const variants = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J']
</script>

<template>
  <div class="card">
    <div class="card-header">
      <span class="badge">{{ problem.points }} pts</span>
      <span class="variant-tag">{{ problem.variant.kind.toUpperCase() }}</span>
    </div>

    <div class="prompt" v-html="promptHtml"></div>

    <!-- MCQ -->
    <div v-if="problem.variant.kind === 'mcq'" class="options">
      <label
        v-for="(opt, idx) in problem.variant.options"
        :key="idx"
        class="option"
        :class="{ selected: selectedMcq === idx, disabled: submitted }"
      >
        <input type="radio" :value="idx" v-model="selectedMcq" :disabled="submitted" />
        <span class="opt-letter">{{ variants[idx] }}</span>
        <span class="opt-text">{{ opt }}</span>
      </label>
    </div>

    <!-- MMCQ -->
    <div v-if="problem.variant.kind === 'mmcq'" class="options">
      <label
        v-for="(opt, idx) in problem.variant.options"
        :key="idx"
        class="option"
        :class="{ selected: selectedMmcq.has(idx), disabled: submitted }"
      >
        <input type="checkbox" :checked="selectedMmcq.has(idx)" @change="toggleMmcq(idx)" :disabled="submitted" />
        <span class="opt-letter">{{ variants[idx] }}</span>
        <span class="opt-text">{{ opt }}</span>
      </label>
    </div>

    <!-- FRQ -->
    <div v-if="problem.variant.kind === 'frq'" class="text-area-wrap">
      <input
        type="text" v-model="frqText" class="text-input"
        placeholder="Type your answer..." :disabled="submitted"
        @keydown.enter="handleSubmit"
      />
    </div>

    <!-- FFRQ -->
    <div v-if="problem.variant.kind === 'ffrq'" class="text-area-wrap">
      <textarea
        v-model="ffrqText" class="text-area"
        placeholder="Write your response..." rows="6" :disabled="submitted"
      ></textarea>
    </div>

    <button v-if="!submitted" class="btn-submit" :disabled="!canSubmit" @click="handleSubmit">
      Save Answer
    </button>

    <div v-if="submitted" class="saved-notice">
      <span class="fb-time">{{ elapsedSec.toFixed(1) }}s</span>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  border-radius: 12px;
  padding: 1.5rem 2rem;
  max-width: 680px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.badge {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--c-accent);
  background: #eef2ff;
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
}

.variant-tag {
  font-size: 0.7rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--c-muted);
}

.prompt {
  margin-bottom: 1.25rem;
  line-height: 1.6;
}

.prompt :deep(h1), .prompt :deep(h2), .prompt :deep(h3), .prompt :deep(h4) { margin: 0.5rem 0 0.75rem; font-weight: 600; }
.prompt :deep(p) { margin: 0 0 0.5rem; }
.prompt :deep(code) { background: #f1f5f9; padding: 0.15em 0.4em; border-radius: 3px; font-size: 0.9em; font-family: var(--font-mono); }
.prompt :deep(pre) { background: #1e293b; color: #e2e8f0; padding: 1rem; border-radius: 8px; overflow-x: auto; font-size: 0.85rem; line-height: 1.5; }
.prompt :deep(pre code) { background: none; padding: 0; color: inherit; }
.prompt :deep(ul), .prompt :deep(ol) { padding-left: 1.5rem; margin: 0.5rem 0; }
.prompt :deep(a) { color: var(--c-accent); }
.prompt :deep(hr) { border: none; border-top: 1px solid var(--c-border); margin: 1rem 0; }

.options { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }

.option {
  display: flex; align-items: center; gap: 0.6rem;
  padding: 0.6rem 0.8rem;
  border: 1.5px solid var(--c-border);
  border-radius: 8px; cursor: pointer;
  transition: all 0.15s;
}
.option:hover:not(.disabled) { border-color: var(--c-accent); background: #f8faff; }
.option.selected:not(.disabled) { border-color: var(--c-accent); background: #eef2ff; }
.option.disabled { cursor: default; opacity: 0.7; }
.option input { display: none; }

.opt-letter {
  display: inline-flex; align-items: center; justify-content: center;
  width: 1.5rem; height: 1.5rem;
  border-radius: 4px; background: #f1f5f9;
  font-size: 0.8rem; font-weight: 600; color: var(--c-muted);
  flex-shrink: 0;
}
.selected .opt-letter { background: var(--c-accent); color: #fff; }
.opt-text { font-size: 0.95rem; }

.text-area-wrap { margin-bottom: 1.25rem; }

.text-input {
  width: 100%; padding: 0.7rem 0.9rem; font-size: 0.95rem; font-family: inherit;
  border: 1.5px solid var(--c-border); border-radius: 8px; outline: none;
  background: var(--c-surface); color: var(--c-text);
}
.text-input:focus { border-color: var(--c-accent); }

.text-area {
  width: 100%; padding: 0.8rem; font-size: 0.95rem; font-family: inherit;
  line-height: 1.6; border: 1.5px solid var(--c-border); border-radius: 8px;
  outline: none; resize: vertical;
  background: var(--c-surface); color: var(--c-text);
}
.text-area:focus { border-color: var(--c-accent); }

.btn-submit {
  padding: 0.6rem 1.8rem; font-size: 0.95rem; font-weight: 500;
  border: none; border-radius: 8px;
  background: var(--c-accent); color: #fff; cursor: pointer;
}
.btn-submit:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-submit:not(:disabled):hover { opacity: 0.85; }

.saved-notice {
  display: flex; align-items: center;
  margin-top: 1.25rem;
  font-size: 0.82rem; color: var(--c-muted);
}
.saved-notice .fb-time { margin-left: auto; }
</style>
