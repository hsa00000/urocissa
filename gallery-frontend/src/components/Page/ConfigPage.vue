<template>
  <PageTemplate>
    <template #content>
      <v-container
        fluid
        class="fill-height align-start justify-center overflow-y-auto bg-surface-light"
      >
        <v-row justify="center">
          <v-col cols="12" sm="10" md="8" lg="6" xl="5">
            <v-row>
              <v-col cols="12">
                <h1 class="text-h4 font-weight-bold">Config</h1>
              </v-col>

              <v-col cols="12">
                <v-form ref="form" v-model="valid" @submit.prevent="save" :disabled="loading">
                  <v-row>
                    <ChangePassword
                      v-model:password="localSettings.password"
                      v-model:old-password="oldPassword"
                    />

                    <StorageAndSync v-model:sync-paths="localSettings.syncPaths" />

                    <AdvancedConfig
                      v-model:auth-key="localSettings.authKey"
                      v-model:discord-hook-url="localSettings.discordHookUrl"
                      v-model:read-only-mode="localSettings.readOnlyMode"
                      v-model:disable-img="localSettings.disableImg"
                    />

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
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onBeforeUnmount } from 'vue'
import { useConfigStore } from '@/store/configStore'
import { useInitializedStore } from '@/store/initializedStore'
import type { AppConfig } from '@/api/config'
import PageTemplate from './PageLayout/PageTemplate.vue'
import ChangePassword from './Config/ChangePassword.vue'
import StorageAndSync from './Config/StorageAndSync.vue'
import AdvancedConfig from './Config/AdvancedConfig.vue'
import { tryWithMessageStore } from '@/script/utils/try_catch'
import { useMessageStore } from '@/store/messageStore'

import { VForm } from 'vuetify/components'

const configStore = useConfigStore('mainId')
const initializedStore = useInitializedStore('mainId')
const messageStore = useMessageStore('mainId')

// UI State
const loading = ref(false)
const valid = ref(false)
const oldPassword = ref('')
const form = ref<VForm | null>(null)


// Local State
const localSettings = reactive<AppConfig>({
  readOnlyMode: false,
  disableImg: false,
  password: '',
  authKey: '',
  discordHookUrl: '',
  syncPaths: [],
  address: '',

  port: 0,
  limits: {}
})

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

  if (result === true) {
    initializedStore.initialized = true
  }

  loading.value = false
}

const resetToStore = () => {
  syncLocalWithStore()
  localSettings.password = ''
  oldPassword.value = ''
  form.value?.resetValidation()
}

const save = async () => {
  const { valid: isValid } = (await form.value?.validate()) ?? { valid: false }
  if (!isValid) return

  loading.value = true
  const success = await tryWithMessageStore('mainId', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const payload: any = { ...localSettings }

    // Only send password if user intends to change it
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/strict-boolean-expressions
    if (!payload.password) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      delete payload.password
    } else {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      payload.oldPassword = oldPassword.value
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    const result = await configStore.updateConfig(payload)

    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (!result) return false

    // Reset password fields on success
    oldPassword.value = ''
    localSettings.password = ''
    // Reset validation state
    form.value?.resetValidation()
    return true
  })

  if (success === true) {
    messageStore.success('Settings saved successfully.')
  }
  loading.value = false
}


onMounted(initData)

onBeforeUnmount(() => {
  initializedStore.initialized = false
})
</script>
