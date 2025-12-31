<template>
  <PageTemplate>
    <template #content>
      <v-container fluid class="fill-height align-start justify-center">
        <v-row justify="center">
          <v-col cols="12" sm="10" md="8" lg="6" xl="5">
            <v-row>
              <v-col cols="12">
                <h1 class="text-h4 font-weight-bold">Settings</h1>
              </v-col>

              <v-col cols="12">
                <v-form ref="form" v-model="valid" @submit.prevent="save" :disabled="loading">
                  <v-row>
                    <v-col cols="12">
                      <v-list-subheader class="font-weight-bold text-high-emphasis"
                        >Security</v-list-subheader
                      >
                      <v-card border flat class="rounded-lg">
                        <v-card-text>
                          <v-row dense>
                            <v-col cols="12">
                              <v-text-field
                                v-model="localSettings.password"
                                label="Application Password"
                                :type="showPassword ? 'text' : 'password'"
                                :append-inner-icon="showPassword ? 'mdi-eye' : 'mdi-eye-off'"
                                prepend-icon="mdi-lock-outline"
                                variant="outlined"
                                density="comfortable"
                                placeholder="Required for access"
                                :rules="[rules.required]"
                                persistent-placeholder
                                hide-details="auto"
                                @click:append-inner="showPassword = !showPassword"
                              ></v-text-field>
                            </v-col>
                          </v-row>
                        </v-card-text>
                      </v-card>
                    </v-col>

                    <v-col cols="12">
                      <v-list-subheader class="font-weight-bold text-high-emphasis">
                        Storage & Sync
                      </v-list-subheader>

                      <v-card border flat class="rounded-lg">
                        <v-card-item
                          density="compact"
                          prepend-icon="mdi-folder-network-outline"
                          class="border-b"
                        >
                          <v-card-title class="text-body-1 font-weight-medium">
                            Sync Paths
                          </v-card-title>

                          <template #append>
                            <v-btn
                              variant="tonal"
                              prepend-icon="mdi-plus"
                              class="text-none font-weight-medium"
                              @click="showFilePicker = true"
                            >
                              Add Path
                            </v-btn>
                          </template>
                        </v-card-item>

                        <v-list v-if="localSettings.syncPaths.length > 0" lines="one">
                          <template v-for="(path, index) in localSettings.syncPaths" :key="index">
                            <v-list-item :title="path">
                              <template #append>
                                <v-btn
                                  icon="mdi-delete-outline"
                                  variant="text"
                                  density="comfortable"
                                  @click="removePath(path)"
                                  title="Remove path"
                                ></v-btn>
                              </template>
                            </v-list-item>
                            <v-divider
                              v-if="index < localSettings.syncPaths.length - 1"
                            ></v-divider>
                          </template>
                        </v-list>

                        <v-empty-state
                          v-else
                          icon="mdi-folder-open-outline"
                          title="No sync paths"
                          text="Add a path to start syncing your files."
                        ></v-empty-state>
                      </v-card>
                    </v-col>

                    <v-col cols="12">
                      <v-list-subheader class="font-weight-bold text-high-emphasis"
                        >Advanced</v-list-subheader
                      >
                      <v-card border flat class="rounded-lg">
                        <v-card-text>
                          <v-row dense>
                            <v-col cols="12">
                              <v-text-field
                                v-model="localSettings.authKey"
                                label="JWT Authentication Key"
                                prepend-icon="mdi-key-outline"
                                placeholder="Enter JWT Key"
                                variant="outlined"
                                density="comfortable"
                                hide-details="auto"
                              ></v-text-field>
                            </v-col>

                            <v-col cols="12">
                              <v-text-field
                                v-model="localSettings.discordHookUrl"
                                label="Discord Webhook URL"
                                prepend-icon="mdi-webhook"
                                placeholder="https://discord.com/api/..."
                                variant="outlined"
                                density="comfortable"
                                hide-details="auto"
                              ></v-text-field>
                            </v-col>

                            <v-col cols="12">
                              <v-text-field
                                v-model.number="localSettings.uploadLimitMb"
                                label="Upload Limit"
                                prepend-icon="mdi-cloud-upload-outline"
                                type="number"
                                suffix="MB"
                                placeholder="0 for unlimited"
                                variant="outlined"
                                density="comfortable"
                                hide-details="auto"
                              ></v-text-field>
                            </v-col>
                          </v-row>
                        </v-card-text>

                        <v-divider></v-divider>

                        <v-list lines="two">
                          <v-list-item
                            title="Read Only Mode"
                            subtitle="Prevent modification of data"
                          >
                            <template #append>
                              <v-switch
                                v-model="localSettings.readOnlyMode"
                                color="primary"
                                hide-details
                                inset
                                density="compact"
                              ></v-switch>
                            </template>
                          </v-list-item>

                          <v-divider></v-divider>

                          <v-list-item
                            title="Disable Processing"
                            subtitle="Skip image generation and analysis"
                          >
                            <template #append>
                              <v-switch
                                v-model="localSettings.disableImg"
                                color="primary"
                                hide-details
                                inset
                                density="compact"
                              ></v-switch>
                            </template>
                          </v-list-item>
                        </v-list>
                      </v-card>
                    </v-col>

                    <v-col cols="12">
                      <v-row justify="end">
                        <v-col cols="auto">
                          <v-btn
                            variant="text"
                            class="text-none"
                            @click="resetToStore"
                            :disabled="loading"
                          >
                            Reset
                          </v-btn>
                        </v-col>
                        <v-col cols="auto">
                          <v-btn
                            color="primary"
                            variant="flat"
                            type="submit"
                            :loading="loading"
                            :disabled="!valid || loading"
                            class="text-none"
                          >
                            Save Changes
                          </v-btn>
                        </v-col>
                      </v-row>
                    </v-col>
                  </v-row>
                </v-form>
              </v-col>
            </v-row>
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
