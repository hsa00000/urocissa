<template>
  <BaseModal
    v-model="modelValue"
    :title="title"
    :width="mobile ? '100%' : 450"
    :fullscreen="mobile"
  >
    <ShareSettingsForm v-model="formState" />

    <template #actions>
      <v-sheet
        border
        :color="showLinkDisplay ? 'grey-darken-4' : 'transparent'"
        :style="{
          borderColor: showLinkDisplay ? 'rgba(255,255,255,0.15)' : 'transparent !important',
          transition: 'none !important'
        }"
        :class="[
          'd-flex w-100',
          mobile ? 'flex-column pa-2' : 'align-center pr-1',
          showLinkDisplay && !mobile ? 'pl-4' : '',
          !showLinkDisplay && !mobile ? 'justify-end pa-2' : ''
        ]"
        :height="mobile ? 'auto' : 54"
      >
        <div
          v-if="showLinkDisplay && !mobile"
          class="text-body-2 text-grey-lighten-1 text-truncate flex-grow-1 mr-3"
          style="user-select: all"
        >
          {{ shareLink }}
        </div>

        <v-sheet
          v-if="showLinkDisplay && mobile"
          color="grey-darken-4"
          class="pa-3 mb-2 rounded text-body-2 text-grey-lighten-1"
          style="user-select: all; word-break: break-all"
        >
          {{ shareLink }}
        </v-sheet>

        <v-btn
          color="primary"
          variant="flat"
          :width="mobile ? '100%' : 150"
          height="44"
          class="text-capitalize"
          :loading="loading"
          :disabled="!isFormValid"
          @click="handleAction"
        >
          {{ buttonLabel }}
        </v-btn>
      </v-sheet>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useDisplay } from 'vuetify'
import { useClipboard } from '@vueuse/core'
import BaseModal from '@/components/Modal/BaseModal.vue'
import ShareSettingsForm, { ShareFormData } from '@/components/Modal/ShareSettingsForm.vue'
import { useMessageStore } from '@/store/messageStore'

const props = withDefaults(
  defineProps<{
    title: string
    shareLink?: string | null
    loading?: boolean
    mode: 'create' | 'edit'
  }>(),
  {
    shareLink: null,
    loading: false
  }
)

const emit = defineEmits<{
  create: [formData: ShareFormData]
  update: [formData: ShareFormData]
  copy: [link: string]
}>()

const modelValue = defineModel<boolean>({ required: true })
const formState = defineModel<ShareFormData>('formState', { required: true })

const { mobile } = useDisplay()
const { copy, copied } = useClipboard({ legacy: true })
const messageStore = useMessageStore('mainId')

const lastSavedState = ref('')

watch(
  () => props.shareLink,
  (newLink) => {
    if (newLink) {
      lastSavedState.value = JSON.stringify(formState.value)
    }
  }
)

const showLinkDisplay = computed(() => !!props.shareLink)

const isFormValid = computed(() => {
  if (formState.value.passwordRequired && !formState.value.password) return false
  return true
})

const hasChanges = computed(() => {
  if (!props.shareLink) return true
  return JSON.stringify(formState.value) !== lastSavedState.value
})

const buttonLabel = computed(() => {
  if (props.mode === 'edit') {
    return 'Save Changes'
  }
  if (!props.shareLink) return 'Create Link'
  if (hasChanges.value) return 'Save Changes'
  return copied.value ? 'Copied!' : 'Copy'
})

const handleAction = async () => {
  if (props.mode === 'edit') {
    emit('update', { ...formState.value })
    return
  }

  if (!props.shareLink) {
    emit('create', { ...formState.value })
  } else if (hasChanges.value) {
    emit('update', { ...formState.value })
    lastSavedState.value = JSON.stringify(formState.value)
  } else {
    if (props.shareLink) {
      await copy(props.shareLink)
      messageStore.success('Link copied to clipboard')
      emit('copy', props.shareLink)
    }
  }
}

defineExpose({
  markAsSaved: () => {
    lastSavedState.value = JSON.stringify(formState.value)
  }
})
</script>
