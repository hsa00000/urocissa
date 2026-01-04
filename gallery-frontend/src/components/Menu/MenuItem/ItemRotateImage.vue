<template>
  <v-list-item prepend-icon="mdi-rotate-left" @click="rotateImage">
    <v-list-item-title class="wrap">Rotate Left</v-list-item-title>
    <template #append>
      <span class="text-caption text-medium-emphasis ms-4">Shift + R</span>
    </template>
  </v-list-item>
</template>

<script lang="ts" setup>
import { useRoute } from 'vue-router'
import { getIsolationIdByRoute } from '@utils/getter'
import { handleRotateImage } from '@/script/utils/rotate'

const route = useRoute()
const isolationId = getIsolationIdByRoute(route)

const rotateImage = async () => {
  const hash = route.params.hash
  if (typeof hash !== 'string') return

  await handleRotateImage(hash, isolationId)
}
</script>
