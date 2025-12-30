<template>
  <PageTemplate>
    <template #content>
      <v-container fluid class="fill-height pa-0">
        <v-row justify="center" class="ma-0 h-100 overflow-y-auto">
          <v-col cols="12" sm="11" md="9" lg="7" xl="5" class="py-6 py-md-10">
            <v-defaults-provider
              :defaults="{
                VTextField: {
                  variant: 'outlined',
                  hideDetails: 'auto',
                  density: 'compact',
                  class: 'mb-1'
                },
                VSwitch: {
                  color: 'primary',
                  density: 'comfortable',
                  hideDetails: true,
                  inset: true
                },
                VBtn: {
                  height: 44
                },
                VCard: {
                  elevation: 0,
                  border: true
                }
              }"
            >
              <v-card class="rounded-xl overflow-hidden" :loading="loading">
                <v-overlay
                  :model-value="loading"
                  contained
                  class="align-center justify-center"
                  persistent
                  scrim="surface"
                >
                  <v-progress-circular indeterminate size="64" />
                </v-overlay>

                <v-card-item class="px-6 pt-6 pb-2">
                  <template #prepend>
                    <v-avatar variant="tonal" rounded="lg" size="52">
                      <v-icon icon="mdi-tune" size="28" />
                    </v-avatar>
                  </template>
                  <v-card-title class="text-h5 font-weight-bold">Configuration</v-card-title>
                  <v-card-subtitle class="text-body-2">Manage storage, security, and global settings</v-card-subtitle>
                </v-card-item>

                <v-divider class="my-2" />

                <v-form ref="form" v-model="valid" @submit.prevent="save" :disabled="loading">
                  <v-card-text class="px-6 py-4 d-flex flex-column ga-6">
                    
                    <!-- Section: Security -->
                    <div>
                      <div class="text-subtitle-2 font-weight-bold text-medium-emphasis text-uppercase mb-3">SECURITY</div>
                      <v-row dense>
                        <v-col cols="12">
                          <v-text-field
                            v-model="localSettings.password"
                            label="Application Password"
                            :type="showPassword ? 'text' : 'password'"
                            :append-inner-icon="showPassword ? 'mdi-eye' : 'mdi-eye-off'"
                            @click:append-inner="showPassword = !showPassword"
                            prepend-inner-icon="mdi-lock-outline"
                            :rules="[rules.required]"
                            hint="Required for accessing the web interface"
                            persistent-hint
                            persistent-placeholder
                          />
                        </v-col>
                      </v-row>
                    </div>

                    <!-- Section: Storage -->
                    <div>
                      <div class="d-flex justify-space-between align-center mb-3">
                        <div class="text-subtitle-2 font-weight-bold text-medium-emphasis text-uppercase">STORAGE & SYNC</div>
                        <v-btn
                          size="small"
                          variant="text"
                          prepend-icon="mdi-plus"
                          @click="showFilePicker = true"
                          class="px-2 font-weight-medium"
                        >
                          Add Path
                        </v-btn>
                      </div>
                      
                      <v-sheet border rounded="lg" class="pa-4">
                        <div v-if="localSettings.syncPaths.length > 0" class="d-flex flex-wrap ga-2">
                          <v-chip
                            v-for="(path, index) in localSettings.syncPaths"
                            :key="index"
                            closable
                            label
                            variant="outlined"
                            class="border-opacity-100"
                            @click:close="removePath(path)"
                          >
                            <v-icon start icon="mdi-folder-network-outline" size="small" />
                            {{ path }}
                          </v-chip>
                        </div>
                        <div v-else class="text-center py-6 text-medium-emphasis text-body-2">
                          <v-icon icon="mdi-folder-open-outline" size="large" class="mb-2 opacity-50" />
                          <div>No sync paths configured</div>
                          <div class="text-caption">Click "Add Path" to start monitoring folders</div>
                        </div>
                      </v-sheet>
                    </div>

                    <!-- Section: Advanced -->
                    <div>
                      <div class="text-subtitle-2 font-weight-bold text-medium-emphasis text-uppercase mb-3">ADVANCED SETTINGS</div>
                      
                      <v-row dense>
                        <v-col cols="12" md="4">
                          <v-text-field
                            v-model="localSettings.authKey"
                            label="JWT Authentication Key"
                            prepend-inner-icon="mdi-shield-key-outline"
                          />
                        </v-col>

                        <v-col cols="12" md="4">
                          <v-text-field
                            v-model="localSettings.discordHookUrl"
                            label="Discord Webhook URL"
                            prepend-inner-icon="mdi-webhook"
                            placeholder="https://discord.com/api/..."
                          />
                        </v-col>

                        <v-col cols="12" md="4">
                          <v-text-field
                            v-model.number="localSettings.uploadLimitMb"
                            label="Upload Limit"
                            type="number"
                            suffix="MB"
                            prepend-inner-icon="mdi-upload-network-outline"
                          />
                        </v-col>
                        
                        <v-col cols="12" sm="6">
                          <v-card variant="outlined" class="pa-3">
                            <div class="d-flex align-center justify-space-between">
                              <div class="d-flex align-center">
                                <v-icon icon="mdi-pencil-off" class="me-3 text-medium-emphasis" />
                                <div>
                                  <div class="text-body-2 font-weight-medium">Read Only Mode</div>
                                  <div class="text-caption text-medium-emphasis">Prevent modification</div>
                                </div>
                              </div>
                              <v-switch v-model="localSettings.readOnlyMode" />
                            </div>
                          </v-card>
                        </v-col>

                        <v-col cols="12" sm="6">
                          <v-card variant="outlined" class="pa-3">
                            <div class="d-flex align-center justify-space-between">
                              <div class="d-flex align-center">
                                <v-icon icon="mdi-image-off-outline" class="me-3 text-medium-emphasis" />
                                <div>
                                  <div class="text-body-2 font-weight-medium text-high-emphasis">Disable Processing</div>
                                  <div class="text-caption text-medium-emphasis">Skip generation</div>
                                </div>
                              </div>
                              <v-switch v-model="localSettings.disableImg" />
                            </div>
                          </v-card>
                        </v-col>
                      </v-row>
                    </div>
                  </v-card-text>

                  <v-divider />

                  <v-card-actions class="pa-6">
                    <v-btn
                      variant="text"
                      @click="resetToStore"
                      :disabled="loading"
                    >
                      Reset Defaults
                    </v-btn>
                    <v-spacer />
                    <v-btn
                      variant="flat"
                      type="submit"
                      :loading="loading"
                      :disabled="!valid || loading"
                      min-width="140"
                      class="font-weight-bold text-none bg-surface-variant"
                    >
                      Save Changes
                    </v-btn>
                  </v-card-actions>
                </v-form>
              </v-card>

              <div class="text-center mt-6 text-caption text-medium-emphasis">
                Urocissa Configuration Manager
              </div>
            </v-defaults-provider>
          </v-col>
        </v-row>
      </v-container>
    </template>
  </PageTemplate>

  <ServerFilePicker v-model="showFilePicker" @select="onFilePickerSelect" />
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onBeforeUnmount } from 'vue'
import { useConfigStore } from '@/store/configStore'
import { useInitializedStore } from '@/store/initializedStore'
import type { AppConfig } from '@/api/config'
import PageTemplate from './PageLayout/PageTemplate.vue'
import ServerFilePicker from './Config/ServerFilePicker.vue'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const configStore = useConfigStore('mainId')
const initializedStore = useInitializedStore('mainId')

// UI State
const loading = ref(false)
const valid = ref(false)
const showPassword = ref(false)
const form = ref<any>(null)
const showFilePicker = ref(false)

// Local State
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
  required: (v: string) => !!v || 'Required'
}

const onFilePickerSelect = (path: string) => {
  if (path && !localSettings.syncPaths.includes(path)) {
    localSettings.syncPaths.push(path)
  }
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

  if (result) {
    initializedStore.initialized = true
  }
  loading.value = false
}

const resetToStore = () => {
  syncLocalWithStore()
  form.value?.resetValidation()
}

const removePath = (path: string) => {
  const index = localSettings.syncPaths.indexOf(path)
  if (index >= 0) localSettings.syncPaths.splice(index, 1)
}

const save = async () => {
  const { valid } = (await form.value?.validate()) || { valid: false }
  if (!valid) return

  loading.value = true
  await tryWithMessageStore('mainId', async () => {
    await configStore.updateConfig(localSettings)
  })
  loading.value = false
}

onMounted(initData)

onBeforeUnmount(() => {
  initializedStore.initialized = false
})
</script>
