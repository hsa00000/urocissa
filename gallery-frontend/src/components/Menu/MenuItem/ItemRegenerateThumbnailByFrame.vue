<template>
  <v-list-item prepend-icon="mdi-image-refresh-outline" @click="regenerateThumbnailByFrame">
    <v-list-item-title class="wrap">Capture Frame as Thumb</v-list-item-title>
  </v-list-item>
</template>

<script lang="ts" setup>
import { useRoute } from 'vue-router'
import axios from 'axios'
import { getIsolationIdByRoute } from '@utils/getter'
import { useCurrentFrameStore } from '@/store/currentFrameStore'
import { useMessageStore } from '@/store/messageStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const route = useRoute()
const isolationId = getIsolationIdByRoute(route)
const currentFrameStore = useCurrentFrameStore(isolationId)
const messageStore = useMessageStore('mainId')

const regenerateThumbnailByFrame = async () => {
  await tryWithMessageStore(isolationId, async () => {
    const hash = route.params.hash
    const currentFrameBlob = await currentFrameStore.getCapture()
    if (typeof hash === 'string' && currentFrameBlob) {
      const formData = new FormData()

      // Append the hash first
      formData.append('hash', hash)

      // Append the frame file
      formData.append('frame', currentFrameBlob)
      messageStore.info('Regenerating thumbnail...')

      const response = await axios.put('/put/regenerate-thumbnail-with-frame', formData, {
        headers: {
          'Content-Type': 'multipart/form-data'
        }
      })

      // Refresh hash token and fetch with bearer token to bust cache
      /*       await tokenStore.refreshHashTokenIfExpired(hash)
      const hashToken = tokenStore.hashTokenMap.get(hash)
      if (hashToken) {
        await fetch(getSrc(hash, false, 'jpg', Date.now()), {
          method: 'GET',
          cache: 'reload',
          headers: {
            Authorization: `Bearer ${hashToken}`
          }
        })
      } */

      messageStore.success('Regenerating thumbnail successfully')
      console.log('Response:', response.data)
    }
  })
}
</script>
