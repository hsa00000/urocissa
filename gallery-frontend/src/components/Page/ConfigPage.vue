<template>
  <PageTemplate>
    <template #content>
      <v-container fluid class="h-100 overflow-y-auto pa-3 pa-md-4">
        <v-row justify="center">
          <v-col cols="12" md="10" lg="8" xl="6">
            <v-card :loading="loading" class="rounded-lg" elevation="2">
              <!-- Loading Overlay -->
              <v-overlay
                :model-value="loading"
                contained
                class="align-center justify-center"
                persistent
                scrim="white"
              >
                <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
              </v-overlay>

              <v-form ref="form" v-model="valid" @submit.prevent="save" :disabled="loading">
                
                <div class="pa-4 d-flex flex-column">
                  
                  <!-- General Section -->
                  <div class="mb-4">
                    <div class="text-subtitle-1 font-weight-bold mb-2">General</div>

                    <v-list lines="two" bg-color="transparent" class="pa-0" density="compact">
                      <v-list-item class="px-0">
                        <v-list-item-title class="font-weight-medium">Read Only Mode</v-list-item-title>
                        <v-list-item-subtitle>Prevent any modifications to the database</v-list-item-subtitle>
                        <template v-slot:append>
                          <v-switch
                            v-model="localSettings.readOnlyMode"
                            color="primary"
                            hide-details
                            inset
                            density="compact"
                          ></v-switch>
                        </template>
                      </v-list-item>

                      <v-divider class="my-2"></v-divider>

                      <v-list-item class="px-0">
                        <v-list-item-title class="font-weight-medium">Disable Image Processing</v-list-item-title>
                        <v-list-item-subtitle>Stop generating thumbnails and metadata</v-list-item-subtitle>
                        <template v-slot:append>
                          <v-switch
                            v-model="localSettings.disableImg"
                            color="warning"
                            hide-details
                            inset
                            density="compact"
                          ></v-switch>
                        </template>
                      </v-list-item>
                    </v-list>

                    <v-text-field
                      v-model="localSettings.discordHookUrl"
                      label="Discord Webhook URL"
                      prepend-inner-icon="mdi-discord"
                      variant="outlined"
                      density="compact"
                      placeholder="https://discord.com/api/webhooks/..."
                      hide-details="auto"
                      bg-color="surface"
                      class="mt-2"
                    ></v-text-field>
                  </div>

                  <v-divider class="my-4"></v-divider>

                  <!-- Security Section -->
                  <div class="mb-4">
                    <div class="text-subtitle-1 font-weight-bold mb-2">Security</div>
                    
                    <v-alert
                      type="info"
                      variant="tonal"
                      class="mb-4"
                      density="compact"
                      border="start"
                      closable
                    >
                      Sensitive information is only visible to authenticated administrators.
                    </v-alert>

                    <v-row dense>
                      <v-col cols="12" md="6">
                        <v-text-field
                          v-model="localSettings.password"
                          label="Application Password"
                          :type="showPassword ? 'text' : 'password'"
                          :append-inner-icon="showPassword ? 'mdi-eye' : 'mdi-eye-off'"
                          @click:append-inner="showPassword = !showPassword"
                          prepend-inner-icon="mdi-key"
                          variant="outlined"
                          density="compact"
                          :rules="[rules.required]"
                          bg-color="surface"
                        ></v-text-field>
                      </v-col>
                      <v-col cols="12" md="6">
                        <v-text-field
                          v-model="localSettings.authKey"
                          label="JWT Authentication Key"
                          prepend-inner-icon="mdi-shield-key"
                          variant="outlined"
                          density="compact"
                          hint="Leave empty to generate automatically"
                          persistent-hint
                          bg-color="surface"
                        ></v-text-field>
                      </v-col>
                    </v-row>
                  </div>

                  <v-divider class="my-4"></v-divider>

                  <!-- Storage Section -->
                  <div class="mb-2">
                    <div class="text-subtitle-1 font-weight-bold mb-2">Storage</div>

                    <v-alert
                      type="warning"
                      variant="tonal"
                      icon="mdi-folder-sync"
                      class="mb-4"
                      density="compact"
                      border="start"
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
                      density="compact"
                      prepend-inner-icon="mdi-folder-multiple"
                      :delimiters="[',']"
                      bg-color="surface"
                      class="mb-4"
                    >
                      <template v-slot:selection="{ item, index }">
                        <v-chip
                          v-if="index < 5"
                          closable
                          size="x-small"
                          color="primary"
                          variant="flat"
                          @click:close="removePath(item.value)"
                        >
                          {{ item.title }}
                        </v-chip>
                        <span
                          v-if="index === 5"
                          class="text-grey text-caption align-self-center ml-2"
                        >
                          (+{{ localSettings.syncPaths.length - 5 }} others)
                        </span>
                      </template>
                      <template v-slot:no-data>
                        <v-list-item density="compact">
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
                      density="compact"
                      type="number"
                      hint="Maximum file size for uploads in megabytes"
                      persistent-hint
                      bg-color="surface"
                      style="max-width: 300px;"
                    ></v-text-field>
                  </div>

                </div>

                <v-divider></v-divider>

                <v-card-actions class="pa-3 bg-surface-light">
                  <v-btn variant="text" @click="resetToStore" :disabled="loading" color="medium-emphasis" size="small">
                    Reset
                  </v-btn>
                  <v-spacer></v-spacer>
                  <v-btn
                    color="primary"
                    variant="flat"
                    type="submit"
                    :loading="loading"
                    :disabled="!valid || loading"
                    min-width="100"
                  >
                    Save Changes
                  </v-btn>
                </v-card-actions>
              </v-form>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </template>
  </PageTemplate>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onBeforeUnmount } from 'vue'
import { useConfigStore } from '@/store/configStore'
import { useInitializedStore } from '@/store/initializedStore'
import type { AppConfig } from '@/api/config'
import PageTemplate from './PageLayout/PageTemplate.vue'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const configStore = useConfigStore('mainId')
const initializedStore = useInitializedStore('mainId')
const loading = ref(false)
const valid = ref(false)
const showPassword = ref(false)

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
    Object.assign(localSettings, JSON.parse(JSON.stringify(configStore.config)))
  }
}

const initData = async () => {
  loading.value = true
  const result = await tryWithMessageStore('mainId', async () => {
    await configStore.fetchConfig()
    syncLocalWithStore()
    return true
  })

  // If initialization failed (undefined result), we can assume it might be auth related
  // or a serious error where we shouldn't enable the UI
  if (result) {
    initializedStore.initialized = true
  }

  loading.value = false
}

const resetToStore = () => {
  syncLocalWithStore()
}

onMounted(async () => {
  await initData()
})

onBeforeUnmount(() => {
  initializedStore.initialized = false
})

const removePath = (path: string) => {
  const index = localSettings.syncPaths.indexOf(path)
  if (index >= 0) localSettings.syncPaths.splice(index, 1)
}

const save = async () => {
  loading.value = true
  await tryWithMessageStore('mainId', async () => {
    await configStore.updateConfig(localSettings)
  })
  loading.value = false
}
</script>
