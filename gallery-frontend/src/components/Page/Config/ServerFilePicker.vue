<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    :fullscreen="isMobile"
    :max-width="isMobile ? undefined : 600"
    scrollable
    transition="dialog-bottom-transition"
  >
    <v-card
      class="d-flex flex-column"
      :class="isMobile ? 'h-100' : ''"
      :height="isMobile ? undefined : 600"
      :min-height="isMobile ? undefined : 600"
    >
      <!-- Toolbar -->
      <v-toolbar color="surface" density="compact" class="border-b pl-2 pr-2">
        <v-btn icon="mdi-arrow-left" @click="navigateUp" :disabled="!currentPath" variant="text" />

        <v-text-field
          v-model="currentPath"
          variant="outlined"
          density="compact"
          hide-details
          class="flex-grow-1 mx-2"
          placeholder="Path..."
          @keyup.enter="loadItems(currentPath)"
        >
          <template #append-inner>
             <v-icon
               icon="mdi-arrow-right"
               size="small"
               class="cursor-pointer"
               @click="loadItems(currentPath)"
               title="Go to path"
             />
          </template>
        </v-text-field>

        <v-btn icon="mdi-close" @click="$emit('update:modelValue', false)" variant="text" />
      </v-toolbar>

      <!-- Content (List) -->
      <v-card-text class="pa-0 flex-grow-1">
        <v-list v-if="loading" class="py-2">
          <v-skeleton-loader type="list-item@5" />
        </v-list>

        <v-list v-else lines="one" class="py-0">
          <!-- Empty State -->
          <div
            v-if="items.length === 0"
            class="d-flex flex-column align-center justify-center py-10 text-medium-emphasis"
          >
            <v-icon icon="mdi-folder-open-outline" size="48" class="mb-2" />
            <div>No folders found</div>
          </div>

          <!-- Folder List -->
          <v-list-item
            v-for="item in items"
            :key="item"
            @click="navigateDown(item)"
            ripple
            class="py-3"
          >
            <template #prepend>
              <v-icon icon="mdi-folder" color="primary" class="mr-4" />
            </template>
            <v-list-item-title class="font-weight-medium">
              {{ getFolderName(item) }}
            </v-list-item-title>
            <template #append>
              <v-icon icon="mdi-chevron-right" color="medium-emphasis" size="small" />
            </template>
          </v-list-item>
        </v-list>
      </v-card-text>

      <!-- Footer (Selection) -->
      <v-divider />
      <v-card-actions class="pa-4 bg-surface">
        <div class="d-flex flex-column w-100 ga-2">
           <div class="text-caption text-medium-emphasis mb-2 px-1" style="word-break: break-all; line-height: 1.2;">
              Currently in: <span class="text-high-emphasis font-weight-bold">{{ currentPath || 'Root' }}</span>
           </div>
           <v-btn
            block
            color="primary"
            variant="flat"
            size="large"
            @click="confirmSelection"
            :disabled="!currentPath"
          >
            Select This Folder
          </v-btn>
        </div>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useDisplay } from 'vuetify'
import { fetchFsCompletion } from '@/api/fs'

const props = defineProps<{
  modelValue: boolean
  initialPath?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', val: boolean): void
  (e: 'select', path: string): void
}>()

const { mobile } = useDisplay()
const isMobile = computed(() => mobile.value)

const currentPath = ref('')
const items = ref<string[]>([])
const loading = ref(false)

// Extract folder name from full path for display
const getFolderName = (fullPath: string) => {
  if (!fullPath) return ''
  // Handle both Windows and Unix separators
  const separator = fullPath.includes('\\') ? '\\' : '/'
  // If it ends with separator (root drives like C:\), keep it
  if (fullPath.endsWith(separator)) return fullPath

  return fullPath.split(separator).pop() || fullPath
}

const loadItems = async (path: string) => {
  loading.value = true
  try {
    const res = await fetchFsCompletion(path)
    items.value = res
  } catch (e) {
    console.error(e)
    items.value = []
  } finally {
    loading.value = false
  }
}

const navigateDown = (path: string) => {
  // Append separator to force directory listing instead of autocomplete
  const isWindows = path.includes('\\')
  const separator = isWindows ? '\\' : '/'
  
  let target = path
  if (!target.endsWith(separator)) {
    target += separator
  }

  currentPath.value = target
  loadItems(target)
}

const navigateUp = () => {
  if (!currentPath.value) return

  // Determine separator
  const isWindows = currentPath.value.includes('\\')
  const separator = isWindows ? '\\' : '/'

  // Logic to find parent
  const parts = currentPath.value.split(separator).filter(Boolean)

  // If we are at root (e.g. "C:" or "mnt"), go to empty string (Show Drives/Roots)
  if (parts.length <= 1) {
    currentPath.value = ''
  } else {
    // Remove last segment
    parts.pop()
    // Reconstruct
    if (isWindows) {
      // Windows needs special handling for drive letter
      currentPath.value = parts.join('\\') + (parts.length === 1 ? '\\' : '')
    } else {
      // Unix starts with /
      currentPath.value = '/' + parts.join('/')
    }
  }

  loadItems(currentPath.value)
}

const confirmSelection = () => {
  if (currentPath.value) {
    // Strip trailing slash for cleaner path, unless it looks like a root (e.g. C:\ or /)
    let selected = currentPath.value
    const isWindows = selected.includes('\\')
    const separator = isWindows ? '\\' : '/'
    
    // Simple heuristic for root: length <= 3 (e.g. "C:\") or just "/"
    const isRoot = (isWindows && selected.length <= 3) || (!isWindows && selected === '/')
    
    if (!isRoot && selected.endsWith(separator)) {
      selected = selected.slice(0, -1)
    }

    emit('select', selected)
    emit('update:modelValue', false)
  }
}

// Initialize
watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      // If we have an initial path, start there, otherwise start at root
      // But if initial path is invalid, backend handles it gracefully
      currentPath.value = props.initialPath || ''
      loadItems(currentPath.value)
    }
  }
)
</script>
