<template>
  <v-col cols="12">
    <v-list-subheader class="font-weight-bold text-high-emphasis">Advanced Config</v-list-subheader>
    <v-card border flat class="rounded-lg">
      <v-list lines="two">
        <v-list-item title="Read Only Mode" subtitle="Prevent modification of data">
          <template #append>
            <v-switch
              v-model="readOnlyMode"
              color="primary"
              hide-details
              inset
              density="compact"
            ></v-switch>
          </template>
        </v-list-item>

        <v-divider></v-divider>

        <v-list-item title="Disable Processing" subtitle="Skip image generation and analysis">
          <template #append>
            <v-switch
              v-model="disableImg"
              color="primary"
              hide-details
              inset
              density="compact"
            ></v-switch>
          </template>
        </v-list-item>
      </v-list>
      <v-divider></v-divider>
      <v-card-text>
        <v-row>
          <v-col cols="12">
            <v-text-field
              v-model="authKey"
              label="JWT Authentication Key"
              prepend-icon="mdi-key-outline"
              placeholder="Enter JWT Key"
              variant="outlined"
              density="comfortable"
              hide-details="auto"
            ></v-text-field>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
const authKey = defineModel<string | null>('authKey')
const readOnlyMode = defineModel<boolean>('readOnlyMode')
const disableImg = defineModel<boolean>('disableImg')
</script>
