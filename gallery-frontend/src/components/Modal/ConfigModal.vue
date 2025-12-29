<template>
  <v-dialog v-model="modalStore.showConfigModal" max-width="600" persistent scrollable>
    <v-card>
      <v-toolbar color="primary" density="compact">
        <v-toolbar-title>System Configuration</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon="mdi-close" variant="text" @click="close"></v-btn>
      </v-toolbar>

      <v-card-text class="pa-0 position-relative">
        <v-overlay :model-value="loading" contained class="align-center justify-center" persistent>
          <v-progress-circular indeterminate color="primary"></v-progress-circular>
        </v-overlay>

        <v-form ref="form" v-model="valid" @submit.prevent="save" :disabled="loading">
          <v-tabs v-model="tab" color="primary" align-tabs="start">
            <v-tab value="general">General</v-tab>
            <v-tab value="security">Security</v-tab>
            <v-tab value="paths">Storage</v-tab>
          </v-tabs>

          <v-window v-model="tab" class="pa-4">
            <v-window-item value="general">
              <v-list subheader>
                <v-list-subheader>Behavior</v-list-subheader>
                <v-list-item>
                  <template v-slot:prepend>
                    <v-icon :icon="localSettings.readOnlyMode ? 'mdi-lock' : 'mdi-pencil'"></v-icon>
                  </template>
                  <v-list-item-title>Read Only Mode</v-list-item-title>
                  <v-list-item-subtitle
                    >Prevent any modifications to the database</v-list-item-subtitle
                  >
                  <template v-slot:append>
                    <v-switch
                      v-model="localSettings.readOnlyMode"
                      color="primary"
                      hide-details
                      inset
                    ></v-switch>
                  </template>
                </v-list-item>

                <v-divider class="my-2"></v-divider>

                <v-list-item>
                  <template v-slot:prepend>
                    <v-icon
                      :icon="localSettings.disableImg ? 'mdi-image-off' : 'mdi-image'"
                    ></v-icon>
                  </template>
                  <v-list-item-title>Disable Image Processing</v-list-item-title>
                  <v-list-item-subtitle
                    >Stop generating thumbnails and metadata</v-list-item-subtitle
                  >
                  <template v-slot:append>
                    <v-switch
                      v-model="localSettings.disableImg"
                      color="warning"
                      hide-details
                      inset
                    ></v-switch>
                  </template>
                </v-list-item>

                <v-divider class="my-4"></v-divider>

                <v-list-subheader>Integrations</v-list-subheader>
                <v-text-field
                  v-model="localSettings.discordHookUrl"
                  label="Discord Webhook URL"
                  prepend-inner-icon="mdi-discord"
                  variant="outlined"
                  density="comfortable"
                  placeholder="https://discord.com/api/webhooks/..."
                  persistent-placeholder
                  hide-details="auto"
                  class="mt-2"
                ></v-text-field>
              </v-list>
            </v-window-item>

            <v-window-item value="security">
              <v-alert
                type="info"
                variant="tonal"
                class="mb-4"
                text="Sensitive information is only visible to authenticated administrators."
              ></v-alert>

              <v-text-field
                v-model="localSettings.password"
                label="Application Password"
                :type="showPassword ? 'text' : 'password'"
                :append-inner-icon="showPassword ? 'mdi-eye' : 'mdi-eye-off'"
                @click:append-inner="showPassword = !showPassword"
                prepend-inner-icon="mdi-key"
                variant="outlined"
                :rules="[rules.required]"
                class="mb-4"
              ></v-text-field>

              <v-text-field
                v-model="localSettings.authKey"
                label="JWT Authentication Key"
                prepend-inner-icon="mdi-shield-key"
                variant="outlined"
                hint="Leave empty to generate automatically on restart"
                persistent-hint
              ></v-text-field>
            </v-window-item>

            <v-window-item value="paths">
              <v-alert
                type="warning"
                variant="tonal"
                icon="mdi-folder-sync"
                class="mb-4"
                density="compact"
              >
                Changing sync paths will trigger a file system rescan.
              </v-alert>

              <v-combobox
                v-model="localSettings.syncPaths"
                label="Synchronization Paths"
                placeholder="Type a path and press Enter"
                chips
                closable-chips
                multiple
                hide-selected
                variant="outlined"
                prepend-inner-icon="mdi-folder-multiple"
                :delimiters="[',']"
              >
                <template v-slot:selection="{ item, index }">
                  <v-chip
                    v-if="index < 5"
                    closable
                    size="small"
                    @click:close="removePath(item.value)"
                  >
                    {{ item.title }}
                  </v-chip>
                  <span v-if="index === 5" class="text-grey text-caption align-self-center ml-2">
                    (+{{ localSettings.syncPaths.length - 5 }} others)
                  </span>
                </template>
                <template v-slot:no-data>
                  <v-list-item>
                    <v-list-item-title>
                      Press <kbd>Enter</kbd> to add a directory path
                    </v-list-item-title>
                  </v-list-item>
                </template>
              </v-combobox>

              <v-text-field
                v-model.number="localSettings.uploadLimitMb"
                label="Upload Size Limit (MB)"
                prepend-inner-icon="mdi-upload"
                variant="outlined"
                type="number"
                hint="Maximum file size for uploads in megabytes"
                persistent-hint
                class="mt-4"
              ></v-text-field>
            </v-window-item>
          </v-window>
        </v-form>
      </v-card-text>

      <v-divider></v-divider>

      <v-card-actions>
        <v-btn variant="text" @click="resetToStore" :disabled="loading">Reset</v-btn>
        <v-spacer></v-spacer>
        <v-btn variant="text" @click="close">Cancel</v-btn>
        <v-btn
          color="primary"
          variant="elevated"
          @click="save"
          :loading="loading"
          :disabled="!valid || loading"
        >
          Save Changes
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useModalStore } from '@/store/modalStore'
import { useConfigStore } from '@/store/configStore'
import type { AppConfig } from '@/api/config'

const modalStore = useModalStore('mainId')
const configStore = useConfigStore('mainId')
const tab = ref('general')
const loading = ref(false)
const valid = ref(false)
const showPassword = ref(false)

// 用於表單操作的本地副本
// 雖然給了初始值，但在 loading 結束前因為有 v-overlay 擋住，使用者看不到錯誤的預設值
const localSettings = reactive<AppConfig>({
  readOnlyMode: false,
  disableImg: false,
  password: '',
  authKey: '',
  discordHookUrl: '',
  syncPaths: [],
  uploadLimitMb: 0,
  address: '',
  port: 0,
  limits: {}
})

const rules = {
  required: (v: string) => !!v || 'This field is required'
}

const syncLocalWithStore = () => {
  if (configStore.config) {
    // Deep copy 以切斷與 Store 的 reference 連結
    Object.assign(localSettings, JSON.parse(JSON.stringify(configStore.config)))
  }
}

// 初始化資料
const initData = async () => {
  loading.value = true
  // 確保 store 有資料
  await configStore.fetchConfig()
  // 同步到本地變數
  syncLocalWithStore()
  loading.value = false
}

const resetToStore = () => {
  syncLocalWithStore()
}

// Watch modal open to fetch/sync data
watch(
  () => modalStore.showConfigModal,
  (val) => {
    if (val) initData()
  },
  { immediate: true }
)

const removePath = (path: string) => {
  const index = localSettings.syncPaths.indexOf(path)
  if (index >= 0) localSettings.syncPaths.splice(index, 1)
}

const save = async () => {
  loading.value = true
  try {
    await configStore.updateConfig(localSettings)
    modalStore.showConfigModal = false
  } finally {
    loading.value = false
  }
}

const close = () => {
  modalStore.showConfigModal = false
}
</script>
