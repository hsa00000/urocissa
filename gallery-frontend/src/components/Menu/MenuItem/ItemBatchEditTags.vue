<template>
  <v-list-item prepend-icon="mdi-tag" @click="modalStore.showBatchEditTagsModal = true">
    <v-list-item-title class="wrap">Batch Edit Tags</v-list-item-title>
  </v-list-item>
</template>

<script lang="ts" setup>
import { useModalStore } from '@/store/modalStore'

const modalStore = useModalStore('mainId')
</script>
