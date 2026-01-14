<template>
  <v-col cols="12">
    <v-card border flat class="rounded-lg">
      <v-card-title class="font-weight-bold"> Advanced Settings </v-card-title>
      <v-divider thickness="4" variant="double"></v-divider>

      <v-list-item
        title="Read Only Mode"
        subtitle="Prevent modification of data and settings"
        @click="readOnlyMode = !readOnlyMode"
      >
        <template #append>
          <v-switch
            v-model="readOnlyMode"
            color="primary"
            hide-details
            inset
            @click.stop
          ></v-switch>
        </template>
      </v-list-item>

      <v-divider></v-divider>

      <v-list-item
        title="Disable Processing"
        subtitle="Skip frontend image rendering for debugging"
        @click="disableImg = !disableImg"
      >
        <template #append>
          <v-switch v-model="disableImg" color="primary" hide-details inset @click.stop></v-switch>
        </template>
      </v-list-item>

      <v-divider></v-divider>

      <v-list-item
        title="Discord Notifications"
        subtitle="Send backend error messages to Discord Webhook"
        @click="hasDiscordHook = !hasDiscordHook"
      >
        <template #append>
          <v-switch
            v-model="hasDiscordHook"
            color="primary"
            hide-details
            inset
            @click.stop
          ></v-switch>
        </template>
      </v-list-item>

      <v-list-item>
        <v-text-field
          v-model="discordHookUrl"
          label="Discord Webhook URL"
          prepend-icon="mdi-webhook"
          placeholder="https://discord.com/api/..."
          variant="outlined"
          density="compact"
          hide-details
          :disabled="!(hasDiscordHook ?? false)"
          @click.stop
          class="py-2"
        ></v-text-field>
      </v-list-item>

      <v-divider></v-divider>
      <v-list-item
        title="JWT Authentication Key"
        subtitle="Provide a key for JWT authentication"
        @click="hasAuthKey = !hasAuthKey"
      >
        <template #append>
          <v-switch v-model="hasAuthKey" color="primary" hide-details inset @click.stop></v-switch>
        </template>
      </v-list-item>
      <v-list-item>
        <v-text-field
          v-model="authKey"
          label="JWT Authentication Key"
          prepend-icon="mdi-key-outline"
          placeholder="Enter JWT Key"
          variant="outlined"
          density="compact"
          hide-details
          :disabled="!(hasAuthKey ?? false)"
          @click.stop
          class="py-2"
        >
        </v-text-field>
      </v-list-item>
      <v-card-actions class="justify-end px-4 pb-4">
        <v-btn color="primary" variant="flat" :loading="loading" @click="save" class="text-none">
          Save Advanced Settings
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useConfigStore } from '@/store/configStore'
import { useMessageStore } from '@/store/messageStore'
import type { AppConfig } from '@/api/config'

const authKey = defineModel<string | null>('authKey')
const discordHookUrl = defineModel<string | null>('discordHookUrl')
const readOnlyMode = defineModel<boolean>('readOnlyMode')
const disableImg = defineModel<boolean>('disableImg')
const hasDiscordHook = defineModel<boolean>('hasDiscordHook')
const hasAuthKey = defineModel<boolean>('hasAuthKey')

const configStore = useConfigStore('mainId')
const messageStore = useMessageStore('mainId')
const loading = ref(false)

watch(hasDiscordHook, (newValue) => {
  if (!(newValue ?? false)) {
    discordHookUrl.value = ''
  }
})

watch(hasAuthKey, (newValue) => {
  if (!(newValue ?? false)) {
    authKey.value = ''
  }
})

const save = async () => {
  loading.value = true

  const payload: Partial<AppConfig> = {
    readOnlyMode: readOnlyMode.value,
    disableImg: disableImg.value
  }

  // Handle Auth Key logic
  // If disabled, send empty string to clear.
  // If enabled and user typed something, send it.
  // If enabled and empty (unchanged), don't send it (preserve existing).
  if (!hasAuthKey.value) {
    payload.authKey = ''
  } else if (authKey.value) {
    payload.authKey = authKey.value
  }

  // Handle Discord Hook logic
  if (!hasDiscordHook.value) {
    payload.discordHookUrl = ''
  } else if (discordHookUrl.value) {
    payload.discordHookUrl = discordHookUrl.value
  }

  const success = await configStore.updateConfig(payload)

  if (success) {
    messageStore.success('Advanced settings saved successfully')
    // Clear sensitive inputs after save if we want, or keep them.
    // Usually good to clear authKey if it's write-only, but keeping it allows correction.
    // Given the API doesn't return it, maybe clear it?
    // Let's leave it as is for now.
  }
  loading.value = false
}
</script>
