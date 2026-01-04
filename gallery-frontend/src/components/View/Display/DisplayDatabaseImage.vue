<template>
  <img
    :key="index"
    v-if="abstractData?.type === 'image' && imgStore.imgOriginal.get(index)"
    :src="imgStore.imgOriginal.get(index)"
    :style="{
      width: `${abstractData?.width}px`,
      height: `${abstractData?.height}px`,
      maxWidth: '100%',
      maxHeight: '100%',
      objectFit: 'scale-down',
      transform: `rotate(${-(editStore.rotationCounts.get(abstractData?.id ?? '') ?? 0) * 90}deg)`
    }"
  />
</template>

<script setup lang="ts">
import { useImgStore } from '@/store/imgStore'
import { useEditStore } from '@/store/editStore'
import { EnrichedUnifiedData, IsolationId } from '@type/types'
import { watchEffect } from 'vue'

const props = defineProps<{
  isolationId: IsolationId
  index: number
  abstractData: EnrichedUnifiedData
}>()

const imgStore = useImgStore(props.isolationId)
const editStore = useEditStore('mainId')
watchEffect(() => {
  // print editStore for debugging
  console.log('editStore.rotationCounts:', editStore.rotationCounts.get(props.abstractData.id))
})
</script>
