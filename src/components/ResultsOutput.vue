<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { SessionState } from '@/types/quiz'
import { encodeSubmission } from '@/utils/crypto'

const props = defineProps<{
  session: SessionState
  encryptedAnswerKey: string
}>()

const emit = defineEmits<{
  restart: []
}>()

const encoded = ref('')
const encoding = ref(true)
const copied = ref(false)
const error = ref('')

onMounted(async () => {
  try {
    encoded.value = await encodeSubmission(
      props.session.studentId,
      props.session.timestamp,
      props.session.entries,
      props.encryptedAnswerKey,
    )
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Encoding failed'
  } finally {
    encoding.value = false
  }
})

const answeredCount = computed(() => props.session.entries.length)
const totalTime = computed(() =>
  props.session.entries.reduce((s, e) => s + e.timeSpentMs, 0),
)

function formatTime(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const sec = ms / 1000
  if (sec < 60) return `${sec.toFixed(1)}s`
  const min = Math.floor(sec / 60)
  const s = Math.round(sec % 60)
  return `${min}m ${s}s`
}

async function handleCopy() {
  try {
    await navigator.clipboard.writeText(encoded.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch {
    const ta = document.querySelector('.blob-output') as HTMLTextAreaElement
    if (ta) { ta.select(); document.execCommand('copy') }
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  }
}
</script>

<template>
  <div class="results">
    <h2>Quiz Complete</h2>

    <div class="stats">
      <div class="stat">
        <span class="stat-label">Questions Answered</span>
        <span class="stat-value">{{ answeredCount }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Total Time</span>
        <span class="stat-value">{{ formatTime(totalTime) }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Student</span>
        <span class="stat-value">{{ session.studentId }}</span>
      </div>
    </div>

    <div class="blob-section">
      <p class="blob-hint">
        Copy the encoded result below and send it to your instructor for grading.
        Your answers are encrypted and can only be decoded by the instructor.
      </p>

      <div v-if="encoding" class="encoding-spinner">Encoding…</div>
      <div v-else-if="error" class="encode-error">{{ error }}</div>

      <template v-else>
        <textarea class="blob-output" :value="encoded" readonly rows="8" spellcheck="false"></textarea>
        <button class="btn-copy" @click="handleCopy">
          {{ copied ? '✓ Copied!' : 'Copy to Clipboard' }}
        </button>
      </template>
    </div>

    <button class="btn-restart" @click="emit('restart')">Start New Quiz</button>
  </div>
</template>

<style scoped>
.results {
  max-width: 680px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.25rem;
}

h2 { font-size: 1.5rem; font-weight: 600; }

.stats {
  display: flex;
  gap: 1.5rem;
  flex-wrap: wrap;
  justify-content: center;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.2rem;
  padding: 0.75rem 1.25rem;
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  border-radius: 8px;
}

.stat-label {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--c-muted);
}

.stat-value {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--c-text);
}

.blob-section {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.blob-hint {
  font-size: 0.85rem;
  color: var(--c-muted);
  text-align: center;
  max-width: 480px;
  line-height: 1.5;
}

.encoding-spinner { color: var(--c-muted); font-size: 0.9rem; }
.encode-error { color: #dc2626; font-size: 0.9rem; }

.blob-output {
  width: 100%;
  padding: 0.8rem;
  font-family: var(--font-mono);
  font-size: 0.8rem;
  line-height: 1.4;
  border: 1px solid var(--c-border);
  border-radius: 8px;
  background: var(--c-surface);
  color: var(--c-muted);
  resize: vertical;
  word-break: break-all;
  outline: none;
}

.btn-copy {
  padding: 0.6rem 1.5rem;
  font-size: 0.9rem;
  font-weight: 500;
  border: 1.5px solid var(--c-border);
  border-radius: 8px;
  background: var(--c-surface);
  color: var(--c-text);
  cursor: pointer;
  transition: all 0.15s;
}
.btn-copy:hover { border-color: var(--c-accent); color: var(--c-accent); }

.btn-restart {
  padding: 0.5rem 1.5rem;
  font-size: 0.9rem;
  font-weight: 500;
  border: none;
  border-radius: 8px;
  background: var(--c-accent);
  color: #fff;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-restart:hover { opacity: 0.85; }
</style>
