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
          :rules="[(v) => !hasDiscordHook || !!v || 'Discord Webhook URL is required']"
          label="Discord Webhook URL"
          prepend-icon="mdi-webhook"
          placeholder="https://discord.com/api/..."
          variant="outlined"
          density="compact"
          hide-details
          :disabled="!hasDiscordHook"
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
          :rules="[(v) => !hasAuthKey || !!v || 'JWT Authentication Key is required']"
          label="JWT Authentication Key"
          prepend-icon="mdi-key-outline"
          placeholder="Enter JWT Key"
          variant="outlined"
          density="compact"
          hide-details
          :disabled="!hasAuthKey"
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

const authKey = defineModel<string | null>('authKey', { required: true })
const discordHookUrl = defineModel<string | null>('discordHookUrl', { required: true })
const readOnlyMode = defineModel<boolean>('readOnlyMode', { required: true })
const disableImg = defineModel<boolean>('disableImg', { required: true })
const hasDiscordHook = defineModel<boolean>('hasDiscordHook', { required: true })
const hasAuthKey = defineModel<boolean>('hasAuthKey', { required: true })

const configStore = useConfigStore('mainId')
const messageStore = useMessageStore('mainId')
const loading = ref(false)

watch(hasDiscordHook, (newValue) => {
  if (!newValue) {
    discordHookUrl.value = ''
  }
})

watch(hasAuthKey, (newValue) => {
  if (!newValue) {
    authKey.value = ''
  }
})

const save = async () => {
  loading.value = true

  if (hasAuthKey.value && (authKey.value == null || authKey.value.trim() === '')) {
    messageStore.error('JWT Authentication Key is required when enabled')
    loading.value = false
    return
  }

  if (
    hasDiscordHook.value &&
    (discordHookUrl.value == null || discordHookUrl.value.trim() === '')
  ) {
    messageStore.error('Discord Webhook URL is required when enabled')
    loading.value = false
    return
  }

  const payload: Partial<AppConfig> = {
    readOnlyMode: readOnlyMode.value,
    disableImg: disableImg.value
  }

  if (!hasAuthKey.value) {
    payload.authKey = null
  } else if (authKey.value != null && authKey.value !== '') {
    payload.authKey = authKey.value
  }

  // Handle Discord Hook logic
  if (!hasDiscordHook.value) {
    payload.discordHookUrl = null
  } else if (discordHookUrl.value != null && discordHookUrl.value !== '') {
    payload.discordHookUrl = discordHookUrl.value
  }

  const success = await configStore.updateConfig(payload)

  if (success === true) {
    messageStore.success('Advanced settings saved successfully')
  }
  loading.value = false
}
</script>
