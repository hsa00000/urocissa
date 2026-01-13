<template>
  <v-col cols="12">
    <v-card border flat class="rounded-lg">
      <v-card-title class="font-weight-bold">Paths</v-card-title>
      <v-divider thickness="4" variant="double"></v-divider>

      <v-list-item
        title="Monitored Paths"
        subtitle="Automatically detect changed files in folders and index them"
        prepend-icon="mdi-folder-network-outline"
        lines="two"
      >
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
      </v-list-item>

      <v-divider></v-divider>
      <v-list v-if="syncPaths.length > 0" lines="one">
        <template v-for="(path, index) in syncPaths" :key="index">
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
          <v-divider v-if="index !== syncPaths.length - 1"></v-divider>
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
  <ServerFilePicker v-model="showFilePicker" @select="onFilePickerSelect" />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ServerFilePicker from './ServerFilePicker.vue'

const syncPaths = defineModel<string[]>('syncPaths', { required: true })

const showFilePicker = ref(false)

const removePath = (path: string) => {
  syncPaths.value = syncPaths.value.filter((p) => p !== path)
}

const onFilePickerSelect = (path: string) => {
  if (path && !syncPaths.value.includes(path)) {
    syncPaths.value = [...syncPaths.value, path]
  }
}
</script>
