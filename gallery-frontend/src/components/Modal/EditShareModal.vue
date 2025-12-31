<template>
  <ShareModalBase
    v-if="modalStore.showEditShareModal"
    v-model="modalStore.showEditShareModal"
    v-model:form-state="formState"
    title="Edit Share Settings"
    mode="edit"
    :loading="loading"
    @update="saveChanges"
  />
</template>

<script setup lang="ts">
import ShareModalBase from '@/components/Modal/ShareModalBase.vue'
import { ShareFormData } from '@/components/Modal/ShareSettingsForm.vue'
import { useModalStore } from '@/store/modalStore'
import { useMessageStore } from '@/store/messageStore'
import { useAlbumStore } from '@/store/albumStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'
import { EditShareData } from '@/type/types'
import axios from 'axios'
import { ref, watchEffect } from 'vue'

const props = defineProps<{ editShareData: EditShareData }>()

const modalStore = useModalStore('mainId')
const messageStore = useMessageStore('mainId')
const albumStore = useAlbumStore('mainId')

const loading = ref(false)

const formState = ref<ShareFormData>({
  description: '',
  passwordRequired: false,
  password: '',
  expireEnabled: false,
  expDuration: null,
  showUpload: false,
  showDownload: false,
  showMetadata: false
})

watchEffect(() => {
  if (props.editShareData && props.editShareData.share) {
    const share = props.editShareData.share
    formState.value = {
      description: share.description || '',
      passwordRequired: !!share.password,
      password: share.password || '',
      expireEnabled: share.exp > 0,
      // In edit mode, default to null (unchanged)
      expDuration: null,
      showUpload: share.showUpload,
      showDownload: share.showDownload,
      showMetadata: share.showMetadata
    }
  }
})

const saveChanges = async (formData: ShareFormData) => {
  loading.value = true

  let newExp = props.editShareData.share.exp

  if (!formData.expireEnabled) {
    newExp = 0
  } else if (formData.expDuration) {
    newExp = Math.floor(Date.now() / 1000) + formData.expDuration * 60
  }

  const updatedShare = {
    url: props.editShareData.share.url,
    description: formData.description,
    password: formData.passwordRequired ? formData.password : null,
    showMetadata: formData.showMetadata,
    showDownload: formData.showDownload,
    showUpload: formData.showUpload,
    exp: newExp
  }

  try {
    // Optimistic Update
    const album = albumStore.albums.get(props.editShareData.albumId)
    if (album) {
      Object.assign(props.editShareData.share, updatedShare)
      album.shareList.set(updatedShare.url, { ...props.editShareData.share, ...updatedShare })
    }

    await tryWithMessageStore('mainId', async () => {
      await axios.put('/put/edit_share', {
        albumId: props.editShareData.albumId,
        share: updatedShare
      })
      messageStore.success('Updated share settings successfully')
      modalStore.showEditShareModal = false
    })
  } catch (e) {
    console.error('Update failed', e)
  } finally {
    loading.value = false
  }
}
</script>
