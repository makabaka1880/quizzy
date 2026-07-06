<script setup lang="ts">
import { ref } from 'vue'
import type { StudentProblemSet, SessionState } from '@/types/quiz'
import ProblemInput from '@/components/ProblemInput.vue'
import QuizRunner from '@/components/QuizRunner.vue'
import ResultsOutput from '@/components/ResultsOutput.vue'
import TeacherPanel from '@/components/TeacherPanel.vue'

type Phase = 'input' | 'quiz' | 'results'
type Role = 'student' | 'teacher'

const role = ref<Role>('student')

// ---- Student state ----
const phase = ref<Phase>('input')
const problemSet = ref<StudentProblemSet | null>(null)
const session = ref<SessionState | null>(null)
const encryptedAnswerKey = ref('')

function handleStart(ps: StudentProblemSet) {
  problemSet.value = ps
  encryptedAnswerKey.value = ps.encryptedAnswerKey
  phase.value = 'quiz'
}

function handleFinish(state: SessionState, answerKey: string) {
  session.value = state
  encryptedAnswerKey.value = answerKey
  phase.value = 'results'
}

function handleRestart() {
  phase.value = 'input'
  problemSet.value = null
  session.value = null
  encryptedAnswerKey.value = ''
}
</script>

<template>
  <div class="app-shell">
    <!-- Role toggle -->
    <div class="role-toggle">
      <button :class="{ active: role === 'student' }" @click="role = 'student'">Student</button>
      <button :class="{ active: role === 'teacher' }" @click="role = 'teacher'">Teacher</button>
    </div>

    <!-- Student flow -->
    <template v-if="role === 'student'">
      <ProblemInput v-if="phase === 'input'" @start="handleStart" />

      <QuizRunner
        v-else-if="phase === 'quiz' && problemSet"
        :problemSet="problemSet"
        @finish="handleFinish"
        @back="handleRestart"
      />

      <ResultsOutput
        v-else-if="phase === 'results' && session"
        :session="session"
        :encryptedAnswerKey="encryptedAnswerKey"
        @restart="handleRestart"
      />
    </template>

    <!-- Teacher panel -->
    <TeacherPanel v-else />
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  padding: 2rem 1.5rem 3rem;
}

.role-toggle {
  display: flex;
  justify-content: center;
  gap: 0;
  margin-bottom: 2rem;
}

.role-toggle button {
  padding: 0.4rem 1.2rem;
  font-size: 0.82rem;
  font-weight: 500;
  border: 1px solid var(--c-border);
  background: var(--c-surface);
  color: var(--c-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.role-toggle button:first-child { border-radius: 6px 0 0 6px; }
.role-toggle button:last-child { border-radius: 0 6px 6px 0; border-left: none; }

.role-toggle button.active {
  background: var(--c-accent);
  color: #fff;
  border-color: var(--c-accent);
}

.role-toggle button:not(.active):hover { background: #f8faff; }
</style>
