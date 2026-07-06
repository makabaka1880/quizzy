<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  loaded: [contents: string]
}>()

const label = ref('Upload file')

function handleFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  label.value = file.name
  const reader = new FileReader()
  reader.onload = () => {
    emit('loaded', reader.result as string)
  }
  reader.onerror = () => {
    label.value = 'Error — retry'
  }
  reader.readAsText(file)

  // Allow re-selecting the same file
  input.value = ''
}
</script>

<template>
  <label class="file-upload">
    <input type="file" class="file-input" @change="handleFile" />
    <span class="file-label">{{ label }}</span>
  </label>
</template>

<style scoped>
.file-upload {
  display: inline-flex;
  cursor: pointer;
  font-size: 0.78rem;
  color: var(--c-muted);
  transition: color 0.15s;
  align-self: flex-start;
}

.file-upload:hover {
  color: var(--c-accent);
}

.file-input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  pointer-events: none;
}

.file-label {
  border-bottom: 1px dashed var(--c-border);
  transition: border-color 0.15s;
}

.file-upload:hover .file-label {
  border-color: var(--c-accent);
}
</style>
