<template>
  <v-list-item>
    <template #prepend>
      <v-avatar>
        <v-icon>mdi-camera-iris</v-icon>
      </v-avatar>
    </template>
    <v-list-item-title class="text-wrap">{{ exifMake }}</v-list-item-title>
    <v-list-item-subtitle class="text-wrap">
      <v-row>
        <v-col cols="auto">{{ exifFormatted.FNumber }}</v-col>
        <v-col cols="auto">{{ exifFormatted.ExposureTime }}</v-col>
        <v-col cols="auto">{{ exifFormatted.FocalLength }}</v-col>
        <v-col cols="auto">{{ exifFormatted.PhotographicSensitivity }}</v-col>
      </v-row>
    </v-list-item-subtitle>
  </v-list-item>
</template>

<script setup lang="ts">
import { GalleryImage, GalleryVideo } from '@type/types'
import { computed } from 'vue'

const props = defineProps<{
  database: GalleryImage | GalleryVideo
}>()

interface ExifData {
  FNumber: string
  ExposureTime: string
  FocalLength: string
  PhotographicSensitivity: string
}

const exifMake = computed(() => {
  const exifData = props.database.exif
  let makeFormatted = ''
  let modelFormatted = ''
  if (exifData.Make !== undefined) {
    makeFormatted = exifData.Make.replace(/"/g, '')
      .split(',')
      .map((part) => part.trim())
      .filter((part) => part !== '')
      .join(', ')
  }
  if (exifData.Model !== undefined) {
    modelFormatted = exifData.Model.replace(/"/g, '')
      .split(',')
      .map((part) => part.trim())
      .filter((part) => part !== '')
      .join(', ')
  }
  return makeFormatted + ' ' + modelFormatted
})

const exifFormatted = computed((): ExifData => {
  const exifData = props.database.exif
  return {
    FNumber: exifData.FNumber !== undefined ? exifData.FNumber.replace('f/', 'ƒ/') : '',
    ExposureTime:
      exifData.ExposureTime !== undefined
        ? `1/${exifData.ExposureTime.replace(' s', '').replace('1/', '')}`
        : '',
    FocalLength:
      exifData.FocalLength !== undefined ? `${exifData.FocalLength.replace(' mm', '')} mm` : '',
    PhotographicSensitivity:
      exifData.PhotographicSensitivity !== undefined
        ? `ISO ${exifData.PhotographicSensitivity}`
        : ''
  }
})
</script>
