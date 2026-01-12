<template>
  <v-col cols="12">
    <v-list-subheader class="font-weight-bold text-high-emphasis">Advanced Config</v-list-subheader>
    <v-card border flat class="rounded-lg">
      <v-list lines="two">
        <v-list-item
          title="Read Only Mode"
          subtitle="Prevent modification of data"
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
          subtitle="Skip image generation and analysis"
          @click="disableImg = !disableImg"
        >
          <template #append>
            <v-switch
              v-model="disableImg"
              color="primary"
              hide-details
              inset
              @click.stop
            ></v-switch>
          </template>
        </v-list-item>

        <v-divider></v-divider>
        <v-list-item @click="hasDiscordHook = !hasDiscordHook" :ripple="!hasDiscordHook">
          <v-text-field
            class="py-2"
            v-model="discordHookUrl"
            label="Discord Webhook URL"
            prepend-icon="mdi-webhook"
            placeholder="https://discord.com/api/..."
            variant="outlined"
            density="compact"
            hide-details
            :disabled="!(hasDiscordHook ?? false)"
            @click.stop
          ></v-text-field>
          <template #append>
            <v-switch
              v-model="hasDiscordHook"
              color="primary"
              hide-details
              inset
              density="compact"
              class="ml-4"
              @click.stop
            ></v-switch>
          </template>
        </v-list-item>

        <v-divider></v-divider>

        <v-list-item>
          <v-text-field
            class="pt-2"
            v-model="authKey"
            label="JWT Authentication Key"
            prepend-icon="mdi-key-outline"
            placeholder="Enter JWT Key"
            variant="outlined"
            density="compact"
            hide-details
            inset
          ></v-text-field>
        </v-list-item>
      </v-list>
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

watch(hasDiscordHook, (newValue) => {
  if (!(newValue ?? false)) {
    discordHookUrl.value = ''
  }
})
</script>
