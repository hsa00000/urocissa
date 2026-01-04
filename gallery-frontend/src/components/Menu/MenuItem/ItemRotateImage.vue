<template>
  <v-list-item prepend-icon="mdi-rotate-left" @click="rotateImage">
    <v-list-item-title class="wrap">Rotate Left</v-list-item-title>
  </v-list-item>
</template>

<script lang="ts" setup>
import { useRoute } from 'vue-router'
import axios from 'axios'
import { getIsolationIdByRoute } from '@utils/getter'
import { useMessageStore } from '@/store/messageStore'
import { useEditStore } from '@/store/editStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const route = useRoute()
const isolationId = getIsolationIdByRoute(route)
const messageStore = useMessageStore('mainId')
const editStore = useEditStore('mainId')

const rotateImage = async () => {
  const hash = route.params.hash
  if (typeof hash !== 'string') return

  if (editStore.hasRotate(hash)) return

  editStore.addRotate(hash)
  try {
    await tryWithMessageStore(isolationId, async () => {
      messageStore.info('Rotating image...')

      await axios.put('/put/rotate-image', { hash })

      editStore.incrementRotation(hash)

      messageStore.success('Image rotated successfully')
    })
  } finally {
    editStore.removeRotate(hash)
  }
}
</script>
