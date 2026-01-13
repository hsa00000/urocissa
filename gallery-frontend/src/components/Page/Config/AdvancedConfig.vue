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
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
import { watch } from 'vue'

const authKey = defineModel<string | null>('authKey')
const discordHookUrl = defineModel<string | null>('discordHookUrl')
const readOnlyMode = defineModel<boolean>('readOnlyMode')
const disableImg = defineModel<boolean>('disableImg')
const hasDiscordHook = defineModel<boolean>('hasDiscordHook')
const hasAuthKey = defineModel<boolean>('hasAuthKey')

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
</script>
