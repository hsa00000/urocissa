<template>
  <v-list-item prepend-icon="mdi-rotate-left" @click="rotateImage">
    <v-list-item-title class="wrap">Rotate Left</v-list-item-title>
  </v-list-item>
</template>

<script lang="ts" setup>
import { useRoute } from 'vue-router'
import axios from 'axios'
import { getIsolationIdByRoute } from '@utils/getter'
import { getSrc } from '@utils/getter'
import { useMessageStore } from '@/store/messageStore'
import { useTokenStore } from '@/store/tokenStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const route = useRoute()
const isolationId = getIsolationIdByRoute(route)
const messageStore = useMessageStore('mainId')
const tokenStore = useTokenStore(isolationId)

const rotateImage = async () => {
  await tryWithMessageStore(isolationId, async () => {
    const hash = route.params.hash
    if (typeof hash === 'string') {
      messageStore.info('Rotating image...')

      await axios.put('/put/rotate-image', { hash })

      // Refresh hash token and fetch with bearer token to bust cache
      await tokenStore.refreshHashTokenIfExpired(hash)
      const hashToken = tokenStore.hashTokenMap.get(hash)
      if (hashToken) {
        await fetch(getSrc(hash, false, 'jpg'), {
          method: 'GET',
          cache: 'reload',
          headers: {
            Authorization: `Bearer ${hashToken}`
          }
        })
      }
      
      messageStore.success('Image rotated successfully')
    }
  })
}
</script>
