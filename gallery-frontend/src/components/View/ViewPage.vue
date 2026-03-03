<template>
  <v-overlay
    v-model="overlayVisible"
    height="100%"
    width="100%"
    class="d-flex"
    id="view-page"
    :transition="false"
    :close-on-back="false"
  >
    <div v-if="index !== undefined" class="pa-0 h-100 w-100 d-flex position-relative bg-background">
      <ViewPageDisplay
        :abstract-data="abstractData"
        :index="index"
        :hash="hash"
        :isolation-id="isolationId"
      />
      <ViewPageMetadata
        v-if="abstractData && constStore.showInfo"
        :abstract-data="abstractData"
        :index="index"
        :hash="hash"
        :isolation-id="isolationId"
      />
    </div>
    <div
      v-else
      class="pa-0 h-100 overflow-hidden position-relative"
      style="background-color: black"
    >
      <div class="d-flex align-center justify-center w-100 h-100">
        <v-progress-circular indeterminate color="primary" size="64" />
      </div>
    </div>
  </v-overlay>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useDataStore } from '@/store/dataStore'
import ViewPageDisplay from '@/components/View/Display/Display.vue'
import ViewPageMetadata from '@/components/View/Metadata/ViewPageMetadata.vue'
import { IsolationId } from '@type/types'
import { useConstStore } from '@/store/constStore'
const props = defineProps<{
  isolationId: IsolationId
}>()

const dataStore = useDataStore(props.isolationId)
const route = useRoute()
const router = useRouter()
const constStore = useConstStore('mainId')

const overlayVisible = computed<boolean>({
  get() {
    return true
  },
  set(val: boolean) {
    if (!val) {
      router.back()
    }
  }
})

const hash = computed(() => {
  return props.isolationId === 'mainId'
    ? (route.params.hash as string)
    : (route.params.subhash as string)
})

const index = computed(() => dataStore.hashMapData.get(hash.value))

const abstractData = computed(() => {
  if (index.value !== undefined) {
    return dataStore.data.get(index.value)
  }
  return undefined
})
</script>
<style scoped>
.v-container::-webkit-scrollbar {
  display: none;
}
</style>
