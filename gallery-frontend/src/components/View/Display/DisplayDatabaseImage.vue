<template>
  <img
    :key="index"
    v-if="abstractData?.type === 'image' && imgStore.imgOriginal.get(index)"
    :src="imgStore.imgOriginal.get(index)"
    :style="{
      width: `${abstractData?.width}px`,
      height: `${abstractData?.height}px`,
      maxWidth: isVertical ? '100cqh' : '100%',
      maxHeight: isVertical ? '100cqw' : '100%',
      objectFit: 'scale-down',
      transform: `rotate(${-(editStore.rotationCounts.get(abstractData?.id ?? '') ?? 0) * 90}deg)`
    }"
  />
</template>

<script setup lang="ts">
import { useImgStore } from '@/store/imgStore'
import { useEditStore } from '@/store/editStore'
import { EnrichedUnifiedData, IsolationId } from '@type/types'
import { computed } from 'vue'

const props = defineProps<{
  isolationId: IsolationId
  index: number
  abstractData: EnrichedUnifiedData
}>()

const imgStore = useImgStore(props.isolationId)
const editStore = useEditStore('mainId')

const isVertical = computed(() => {
  const rotationCount = editStore.rotationCounts.get(props.abstractData?.id ?? '') ?? 0
  // If count is odd (1, 3, 5...), it is 90 or 270 degrees
  return Math.abs(rotationCount % 2) === 1
})
</script>
