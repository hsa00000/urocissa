<template>
  <PageTemplate>
    <template #content>
      <v-container fluid class="h-100 overflow-y-auto">
        <v-row justify="center">
          <v-col cols="12" md="8" lg="6" xl="5">
            <v-defaults-provider
              :defaults="{
                VTextField: {
                  variant: 'outlined',
                  color: 'primary',
                  hideDetails: 'auto',
                  density: 'comfortable'
                },
                VCombobox: {
                  variant: 'outlined',
                  color: 'primary',
                  hideDetails: 'auto',
                  density: 'comfortable'
                },
                VSwitch: {
                  color: 'primary',
                  density: 'compact',
                  hideDetails: true,
                  inset: true
                }
              }"
            >
              <v-card elevation="2" class="rounded-lg" :loading="loading">
                <v-overlay
                  :model-value="loading"
                  contained
                  class="align-center justify-center"
                  persistent
                  scrim="white"
                >
                  <v-progress-circular indeterminate color="primary" size="64" />
                </v-overlay>

                <v-card-item class="pb-4">
                  <template #prepend>
                    <v-avatar color="primary" variant="tonal" rounded="lg">
                      <v-icon icon="mdi-tune" />
                    </v-avatar>
                  </template>
                  <v-card-title>Configuration</v-card-title>
                  <v-card-subtitle>Manage storage access and security</v-card-subtitle>
                </v-card-item>

                <v-divider />

                <v-form ref="form" v-model="valid" @submit.prevent="save" :disabled="loading">
                  <v-card-text class="d-flex flex-column gap-y-4 py-6">
                    <v-text-field
                      v-model="localSettings.password"
                      label="Application Password"
                      :type="showPassword ? 'text' : 'password'"
                      :append-inner-icon="showPassword ? 'mdi-eye' : 'mdi-eye-off'"
                      @click:append-inner="showPassword = !showPassword"
                      prepend-inner-icon="mdi-lock-outline"
                      :rules="[rules.required]"
                      hint="Required for accessing the interface"
                      persistent-hint
                    />

                    <v-combobox
                      v-model="localSettings.syncPaths"
                      label="Synchronization Paths"
                      placeholder="e.g. /mnt/data/images"
                      prepend-inner-icon="mdi-folder-network-outline"
                      chips
                      closable-chips
                      multiple
                      :delimiters="[',']"
                      hint="Directories to scan for content"
                      persistent-hint
                      class="mt-4"
                      autocomplete="off"
                    >
                      <template #selection="{ item, index }">
                        <v-chip
                          v-if="index < 2"
                          size="small"
                          variant="flat"
                          color="surface-variant"
                          closable
                          @click:close="removePath(item.value)"
                        >
                          {{ item.title }}
                        </v-chip>
                        <span
                          v-if="index === 2"
                          class="text-caption text-medium-emphasis align-self-center ml-2"
                        >
                          (+{{ localSettings.syncPaths.length - 2 }})
                        </span>
                      </template>
                    </v-combobox>
                  </v-card-text>

                  <v-divider />

                  <v-card-text class="py-6">
                    <div
                      class="text-subtitle-2 font-weight-bold mb-4 d-flex align-center text-medium-emphasis"
                    >
                      <v-icon icon="mdi-cog-outline" size="small" class="me-2" />
                      Advanced Settings
                    </div>

                    <v-row dense>
                      <v-col cols="12">
                        <v-list density="compact" bg-color="transparent" class="py-0">
                          <v-list-item class="px-0">
                            <template #prepend>
                              <v-icon icon="mdi-pencil-off" color="medium-emphasis" class="me-2" />
                            </template>
                            <v-list-item-title>Read Only Mode</v-list-item-title>
                            <template #append>
                              <v-switch v-model="localSettings.readOnlyMode" />
                            </template>
                          </v-list-item>

                          <v-list-item class="px-0">
                            <template #prepend>
                              <v-icon
                                icon="mdi-image-off-outline"
                                color="medium-emphasis"
                                class="me-2"
                              />
                            </template>
                            <v-list-item-title>Disable Image Processing</v-list-item-title>
                            <template #append>
                              <v-switch v-model="localSettings.disableImg" color="warning" />
                            </template>
                          </v-list-item>
                        </v-list>
                      </v-col>

                      <v-col cols="12" class="mt-2">
                        <v-text-field
                          v-model="localSettings.authKey"
                          label="JWT Authentication Key"
                          prepend-inner-icon="mdi-shield-key-outline"
                          class="mb-3"
                        />
                      </v-col>

                      <v-col cols="12" md="8">
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
                    </v-row>
                  </v-card-text>

                  <v-divider />

                  <v-card-actions class="pa-4">
                    <v-btn
                      variant="text"
                      color="medium-emphasis"
                      @click="resetToStore"
                      :disabled="loading"
                    >
                      Reset Defaults
                    </v-btn>
                    <v-spacer />
                    <v-btn
                      color="primary"
                      variant="flat"
                      type="submit"
                      :loading="loading"
                      :disabled="!valid || loading"
                      min-width="120"
                      size="large"
                    >
                      Save Configuration
                    </v-btn>
                  </v-card-actions>
                </v-form>
              </v-card>
            </v-defaults-provider>
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

// UI State
const loading = ref(false)
const valid = ref(false)
const showPassword = ref(false)
const form = ref<any>(null)

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

<style scoped>
/* Vuetify helper classes should handle most things, but this ensures slightly tighter spacing in list items */
.gap-y-4 {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
