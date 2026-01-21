<template>
  <PageTemplate>
    <template #content>
      <v-container fluid class="fill-height overflow-y-auto bg-surface-light">
        <v-row justify="center" no-gutters>
          <v-col cols="12" md="8" lg="6">
            <v-row>
              <ChangePassword v-model:has-password="localSettings.hasPassword" />
              <StorageAndSync v-model:sync-paths="localSettings.syncPaths" />
              <AdvancedConfig
                v-model:auth-key="localSettings.authKey"
                v-model:has-auth-key="localSettings.hasAuthKey"
                v-model:discord-hook-url="localSettings.discordHookUrl"
                v-model:read-only-mode="localSettings.readOnlyMode"
                v-model:disable-img="localSettings.disableImg"
                v-model:has-discord-hook="localSettings.hasDiscordHook"
              />
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

const configStore = useConfigStore('mainId')
const initializedStore = useInitializedStore('mainId')

// UI State
const loading = ref(false)

// Local State
const localSettings = reactive<AppConfig>({
  readOnlyMode: false,
  disableImg: false,
  localMode: false,
  hasPassword: false,
  hasAuthKey: false,
  hasDiscordHook: false,
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

onMounted(initData)

onBeforeUnmount(() => {
  initializedStore.initialized = false
})
</script>
