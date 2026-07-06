<script setup lang="ts">
import { ref, computed } from 'vue'
import type { StudentProblemSet, StudentProblem, GradingEntry, SessionState } from '@/types/quiz'
import QuizProgress from './QuizProgress.vue'
import QuizCard from './QuizCard.vue'

const props = defineProps<{
  problemSet: StudentProblemSet
}>()

const emit = defineEmits<{
  finish: [state: SessionState, encryptedAnswerKey: string]
  back: []
}>()

const currentIndex = ref(0)
const entries = ref<(GradingEntry | null)[]>(
  props.problemSet.problems.map(() => null),
)

const problems = computed(() => props.problemSet.problems)
const currentProblem = computed(() => problems.value[currentIndex.value])
const allAnswered = computed(() => entries.value.every((e) => e !== null))
const answeredCount = computed(() => entries.value.filter((e) => e !== null).length)

function handleSubmit(entry: GradingEntry) {
  entries.value[currentIndex.value] = entry
  setTimeout(() => {
    const nextUnanswered = entries.value.findIndex((e) => e === null)
    if (nextUnanswered !== -1) {
      currentIndex.value = nextUnanswered
    }
  }, 400)
}

function handleJump(index: number) {
  currentIndex.value = index
}

function handleFinish() {
  const session: SessionState = {
    studentId: props.problemSet.studentId,
    timestamp: Date.now(),
    entries: entries.value as GradingEntry[],
  }
  emit('finish', session, props.problemSet.encryptedAnswerKey)
}
</script>

<template>
  <div class="runner">
    <QuizProgress
      :entries="entries"
      :currentIndex="currentIndex"
      @jump="handleJump"
    />

    <div class="counter">
      Question {{ currentIndex + 1 }} of {{ problems.length }}
      <span class="counter-answered">({{ answeredCount }} answered)</span>
    </div>

    <QuizCard
      v-if="currentProblem"
      :key="currentProblem.id"
      :problem="currentProblem"
      :previousEntry="entries[currentIndex] ?? null"
      @submit="handleSubmit"
    />

    <div class="nav">
      <button class="btn-nav" :disabled="currentIndex === 0" @click="currentIndex--">← Prev</button>
      <button class="btn-nav" :disabled="currentIndex === problems.length - 1" @click="currentIndex++">Next →</button>
    </div>

    <div v-if="allAnswered" class="finish-area">
      <p class="finish-msg">All {{ problems.length }} questions answered. Submit your responses.</p>
      <button class="btn-finish" @click="handleFinish">
        Finish &amp; Submit
      </button>
    </div>

    <button class="btn-back" @click="emit('back')">← Start over</button>
  </div>
</template>

<style scoped>
.runner {
  max-width: 740px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.counter {
  text-align: center;
  font-size: 0.85rem;
  color: var(--c-muted);
}

.counter-answered {
  color: var(--c-accent);
  font-weight: 500;
}

.nav {
  display: flex;
  justify-content: center;
  gap: 0.75rem;
}

.btn-nav {
  padding: 0.5rem 1.2rem;
  font-size: 0.9rem;
  font-weight: 500;
  border: 1px solid var(--c-border);
  border-radius: 8px;
  background: var(--c-surface);
  color: var(--c-text);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-nav:hover:not(:disabled) { border-color: var(--c-accent); color: var(--c-accent); }
.btn-nav:disabled { opacity: 0.35; cursor: not-allowed; }

.finish-area {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--c-border);
}

.finish-msg {
  font-size: 0.9rem;
  color: var(--c-muted);
}

.btn-finish {
  padding: 0.75rem 2.5rem;
  font-size: 1.05rem;
  font-weight: 600;
  border: none;
  border-radius: 8px;
  background: #16a34a;
  color: #fff;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-finish:hover { opacity: 0.88; }

.btn-back {
  align-self: center;
  padding: 0.4rem 1rem;
  font-size: 0.85rem;
  border: none;
  background: none;
  color: var(--c-muted);
  cursor: pointer;
  transition: color 0.15s;
}

.btn-back:hover { color: var(--c-text); }
</style>
