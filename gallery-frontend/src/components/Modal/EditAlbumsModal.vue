<template>
  <v-dialog
    v-if="submit !== undefined"
    v-model="modalStore.showEditAlbumsModal"
    variant="flat"
    persistent
    id="edit-album-overlay"
    max-width="400"
  >
    <v-confirm-edit
      v-model="changedAlbums"
      :disabled="false"
      @save="submit"
      @cancel="modalStore.showEditAlbumsModal = false"
    >
      <template #default="{ model: proxyModel, actions }">
        <v-card variant="elevated" retain-focus>
          <template #title>Edit Albums</template>

          <template #text>
            <v-form v-model="formIsValid" @submit.prevent="submit" validate-on="input">
              <v-select
                v-model="proxyModel.value"
                chips
                multiple
                :items="albumItems"
                item-title="displayName"
                item-value="albumId"
                label="Albums"
                variant="outlined"
                closable-chips
                return-object
              >
                <template #prepend-item v-if="albumStore.albums.size > 0">
                  <v-list-item value="" @click.stop.prevent="createNonEmptyAlbumWithLoading">
                    <template #prepend="{ isActive }">
                      <v-list-item-action>
                        <v-checkbox-btn :model-value="isActive" :disabled="loading">
                          <template #input="{ inputNode }">
                            <v-icon v-if="loading" icon="mdi-loading" class="mdi-spin" />
                            <v-icon v-else icon="mdi-plus" />
                            <RenderVNode :node="inputNode" />
                          </template>
                        </v-checkbox-btn>
                      </v-list-item-action>

                      <v-list-item-title class="wrap"> Create New Album </v-list-item-title>
                    </template>
                  </v-list-item>
                  <v-divider />
                </template>

                <template #no-data v-else>
                  <v-list-item value="" @click.stop.prevent="createNonEmptyAlbumWithLoading">
                    <template #prepend>
                      <v-list-item-action>
                        <v-btn color="transparent" icon="mdi-plus" density="comfortable" flat />
                      </v-list-item-action>
                      <v-list-item-title class="wrap"> Create New Album </v-list-item-title>
                    </template>
                  </v-list-item>
                </template>
              </v-select>
            </v-form>
          </template>

          <v-divider />

          <template #actions>
            <v-spacer />
            <component :is="actions" />
          </template>
        </v-card>
      </template>
    </v-confirm-edit>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, toRaw } from 'vue'
import { useRoute } from 'vue-router'
import { useModalStore } from '@/store/modalStore'
import { useAlbumStore } from '@/store/albumStore'
import type { AlbumInfo } from '@type/types'
import { getHashIndexDataFromRoute, getIsolationIdByRoute } from '@utils/getter'
import { editAlbums } from '@/api/editAlbums'
import { useCreateAlbumAction } from '@/script/hook/useCreateAlbumAction'
import { defineComponent } from 'vue'
import type { PropType, VNode } from 'vue'

const RenderVNode = defineComponent({
  name: 'RenderVNode',
  props: {
    node: { type: Object as PropType<VNode>, required: true }
  },
  setup: (props) => () => props.node
})
const formIsValid = ref(false)
const changedAlbums = ref<AlbumInfo[]>([])
const submit = ref<(() => Promise<void>) | undefined>()
const currentItemIndex = ref<number | undefined>(undefined)

const route = useRoute()
const modalStore = useModalStore('mainId')
const albumStore = useAlbumStore('mainId')
const { loading, createAndNavigate } = useCreateAlbumAction()

const albumItems = computed<AlbumInfo[]>(() =>
  [...albumStore.albums.values()].map((a) => structuredClone(toRaw(a)))
)

onMounted(() => {
  const initSubmit = (): (() => Promise<void>) | undefined => {
    const parsed = getHashIndexDataFromRoute(route)
    if (!parsed) {
      console.error('initSubmit: failed to parse route.')
      return
    }

    const { index, data } = parsed
    currentItemIndex.value = index
    if (data.type !== 'image' && data.type !== 'video') {
      console.error("initSubmit: 'data' is not an image or video.")
      return
    }

    const defaultAlbumIds = [...data.albums]

    const initialAlbums = defaultAlbumIds
      .map((id) => albumStore.albums.get(id))
      .filter((a): a is AlbumInfo => a !== undefined)
      .map((a) => structuredClone(toRaw(a)))
    // Temporary workaround: VConfirmEdit internally uses structuredClone,
    // which cannot clone Vue reactive proxies directly.
    // We use toRaw() + structuredClone() here to ensure plain objects.

    changedAlbums.value = initialAlbums

    const innerSubmit = async () => {
      const selectedIds = changedAlbums.value.map((a) => a.albumId)

      const addAlbumIds = selectedIds.filter((id) => !defaultAlbumIds.includes(id))
      const removeAlbumIds = defaultAlbumIds.filter((id) => !selectedIds.includes(id))

      modalStore.showEditAlbumsModal = false
      await editAlbums([index], addAlbumIds, removeAlbumIds, getIsolationIdByRoute(route))
    }

    return innerSubmit
  }

  submit.value = initSubmit()
})

const createNonEmptyAlbumWithLoading = async () => {
  if (currentItemIndex.value === undefined) return
  const isolationId = getIsolationIdByRoute(route)
  await createAndNavigate([currentItemIndex.value], isolationId, () => {
    modalStore.showEditAlbumsModal = false
  })
}
</script>

<style scoped></style>
