<script setup lang="ts">
import type { GradingEntry } from '@/types/quiz'

const props = defineProps<{
  entries: (GradingEntry | null)[]
  currentIndex: number
}>()

const emit = defineEmits<{
  jump: [index: number]
}>()

function dotClass(index: number): string {
  if (index === props.currentIndex) return 'dot current'
  return props.entries[index] ? 'dot answered' : 'dot unanswered'
}

function dotLabel(entry: GradingEntry | null, index: number): string {
  if (!entry) return `Question ${index + 1} — not yet answered`
  return `Question ${index + 1} — answered`
}
</script>

<template>
  <div class="progress">
    <button
      v-for="(entry, idx) in entries"
      :key="idx"
      :class="dotClass(idx)"
      :title="dotLabel(entry, idx)"
      :aria-label="dotLabel(entry, idx)"
      @click="emit('jump', idx)"
    >
      {{ idx + 1 }}
    </button>
  </div>
</template>

<style scoped>
.progress {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  flex-wrap: wrap;
}

.dot {
  width: 2rem;
  height: 2rem;
  border-radius: 50%;
  border: 1.5px solid var(--c-border);
  background: var(--c-surface);
  color: var(--c-muted);
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  padding: 0;
}

.dot:hover { border-color: var(--c-accent); }

.dot.current {
  border-color: var(--c-accent);
  background: #eef2ff;
  color: var(--c-accent);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.15);
}

.dot.answered {
  border-color: var(--c-accent);
  background: #eef2ff;
  color: var(--c-accent);
}

.dot.unanswered {
  border-color: var(--c-border);
  background: var(--c-surface);
  color: var(--c-muted);
}
</style>
