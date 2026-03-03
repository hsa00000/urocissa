<template>
  <v-list-item prepend-icon="mdi-image-album" @click="modalStore.showBatchEditAlbumsModal = true">
    <v-list-item-title class="wrap">Batch Edit Albums</v-list-item-title>
  </v-list-item>
</template>

<script lang="ts" setup>
import { useModalStore } from '@/store/modalStore'
const modalStore = useModalStore('mainId')
</script>
