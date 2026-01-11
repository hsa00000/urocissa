<template>
  <v-dialog
    v-model="modelValue"
    :fullscreen="isMobile"
    :max-width="isMobile ? undefined : 600"
    :height="isMobile ? undefined : 600"
    scrollable
    transition="dialog-bottom-transition"
  >
    <v-card :height="isMobile ? '100%' : 600" class="d-flex flex-column">
      <v-toolbar density="compact" class="border-b">
        <v-container fluid class="fill-height py-0">
          <v-row no-gutters align="center" class="ga-2">
            <v-col cols="auto">
              <v-btn
                icon="mdi-arrow-left"
                variant="text"
                :disabled="!currentPath"
                @click="navigateUp"
              />
            </v-col>

            <v-col>
              <v-text-field
                v-model="currentPath"
                placeholder="Path..."
                hide-details
                density="compact"
                variant="outlined"
                single-line
                :error="!!errorMsg"
                @keyup.enter="loadItems(currentPath).catch(console.error)"
              >
                <template #append-inner>
                  <v-fade-transition>
                    <v-icon
                      v-if="currentPath"
                      icon="mdi-arrow-right"
                      class="cursor-pointer"
                      @click="loadItems(currentPath).catch(console.error)"
                    />
                  </v-fade-transition>
                </template>
              </v-text-field>

            </v-col>

            <v-col cols="auto">
              <v-btn icon="mdi-close" variant="text" @click="modelValue = false" />
            </v-col>
          </v-row>
        </v-container>
      </v-toolbar>

      <v-card-text class="pa-0 d-flex flex-column">
        <v-list v-if="loading" disabled>
          <v-skeleton-loader type="list-item@5" />
        </v-list>

        <v-empty-state
          v-else-if="items.length === 0 && roots.length === 0"
          :icon="emptyStateIcon"
          title="No folders found"
          :text="errorMsg || 'This folder has no subfolders.'"
          class="ma-auto"
        />

        <v-list v-else lines="one" density="default">
          <template v-if="isDefault && roots.length > 0">
            <v-list-subheader>Drives / Roots</v-list-subheader>
            <v-list-item
              v-for="item in roots"
              :key="item"
              :value="item"
              color="primary"
              @click="navigateDown(item)"
            >
              <template #prepend>
                <v-icon icon="mdi-harddisk" />
              </template>
              <v-list-item-title>{{ item }}</v-list-item-title>
              <template #append>
                <v-icon icon="mdi-chevron-right" size="small" />
              </template>
            </v-list-item>
            <v-divider class="my-2" />
            <v-list-subheader>Current Directory</v-list-subheader>
          </template>

          <v-list-item
            v-for="item in items"
            :key="item"
            :value="item"
            color="primary"
            @click="navigateDown(item)"
          >
            <template #prepend>
              <v-icon icon="mdi-folder" />
            </template>

            <v-list-item-title>
              {{ getFolderName(item) }}
            </v-list-item-title>

            <template #append>
              <v-icon icon="mdi-chevron-right" size="small" />
            </template>
          </v-list-item>
        </v-list>
      </v-card-text>

      <v-divider />

      <v-card-actions>
        <v-container fluid>
          <v-row align="center" dense>
            <v-col>
              <div class="text-caption text-medium-emphasis text-truncate">
                Selected:
                <span class="text-high-emphasis">{{ currentPath || 'Root' }}</span>
              </div>
            </v-col>

            <v-col cols="auto">
              <v-btn variant="tonal" @click="confirmSelection" :disabled="!currentPath">
                Select Folder
              </v-btn>
            </v-col>
          </v-row>
        </v-container>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useDisplay } from 'vuetify'
import { fetchFsCompletion } from '@/api/fs'
import axios from 'axios'

// --- Props & Emits ---
const modelValue = defineModel<boolean>({ required: true })

const props = defineProps<{
  initialPath?: string
}>()

 
const emit = defineEmits<(e: 'select', path: string) => void>()


// --- Responsiveness ---
const { mobile } = useDisplay()
const isMobile = computed(() => mobile.value)

// --- State ---
const currentPath = ref('')
const items = ref<string[]>([])
const roots = ref<string[]>([])
const isDefault = ref(false)
const loading = ref(false)
const errorMsg = ref('')

const emptyStateIcon = computed(() => {
  if (errorMsg.value) return 'mdi-folder-alert-outline'
  return 'mdi-folder-open-outline'
})

// Utilities
const getFolderName = (fullPath: string) => {
   
  if (!fullPath) return ''
  const separator = fullPath.includes('\\') ? '\\' : '/'
  if (fullPath.endsWith(separator)) return fullPath
  // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing, @typescript-eslint/strict-boolean-expressions
  return fullPath.split(separator).pop() || fullPath
}


// --- Logic ---
const loadItems = async (path: string) => {
  loading.value = true
  errorMsg.value = ''
  try {
    const res = await fetchFsCompletion(path)
    items.value = res.children
    roots.value = res.roots
    isDefault.value = res.is_default
  } catch (e: unknown) {
    console.error(e)
    items.value = []
    roots.value = []
    isDefault.value = false
    if (axios.isAxiosError(e) && e.response?.status === 404) {
      errorMsg.value = 'Directory does not exist'
    } else {
      errorMsg.value = 'Error listing directory'
    }
  } finally {
    loading.value = false
  }
}


const navigateDown = (path: string) => {
  const isWindows = path.includes('\\')
  const separator = isWindows ? '\\' : '/'

  let target = path
  if (!target.endsWith(separator)) {
    target += separator
  }

  currentPath.value = target
  loadItems(target).catch(console.error)
}

const navigateUp = () => {
  if (!currentPath.value) return

  const isWindows = currentPath.value.includes('\\')
  const separator = isWindows ? '\\' : '/'

    // clean up existing path to handle parsing
    const cleanPath = currentPath.value.endsWith(separator)
      ? currentPath.value.slice(0, -1)
      : currentPath.value

    const parts = cleanPath.split(separator)


  // Go to root logic
  if (parts.length <= 1) {
    currentPath.value = ''
  } else {
    parts.pop() // Remove last segment

    if (isWindows) {
      // e.g. "C:" needs backslash to be valid root often
      currentPath.value = parts.join('\\') + (parts.length === 1 ? '\\' : '')
    } else {
      // e.g. "" -> join -> "" implies root /
      const newPath = parts.join('/')
      currentPath.value = newPath || '/'
    }

    // Ensure trailing slash for intermediate directories to avoid "searching" mode
    const separator = isWindows ? '\\' : '/'
    if (!currentPath.value.endsWith(separator)) {
      currentPath.value += separator
    }
  }

  loadItems(currentPath.value).catch(console.error)
}


const confirmSelection = () => {
  if (currentPath.value) {
    let selected = currentPath.value
    const isWindows = selected.includes('\\')
    const separator = isWindows ? '\\' : '/'

    // Normalize root check
     
    const isRoot = (isWindows && selected.length <= 3) || (!isWindows && selected === '/')

    // Remove trailing slash if not root
     
    if (!isRoot && selected.endsWith(separator)) {
      selected = selected.slice(0, -1)
    }

    emit('select', selected)
    modelValue.value = false
  }
}

// --- Watchers ---
watch(modelValue, (isOpen) => {
  if (isOpen) {
    currentPath.value = props.initialPath ?? ''
    loadItems(currentPath.value).catch((err: unknown) => {
      console.error('Failed to load items in watcher:', err)
    })
  }
})

</script>
