<template>
  <v-dialog
    v-if="submit !== undefined"
    v-model="modalStore.showEditTagsModal"
    variant="flat"
    persistent
    id="edit-tag-overlay"
    max-width="400"
  >
    <v-confirm-edit
      v-model="changedTagsArray"
      :disabled="false"
      @save="submit"
      @cancel="modalStore.showEditTagsModal = false"
    >
      <template #default="{ model: proxyModel, actions }">
        <v-card variant="elevated" retain-focus>
          <template #title> Edit Tags </template>
          <template #text>
            <v-form v-model="formIsValid" @submit.prevent="submit" validate-on="input">
              <v-combobox
                v-model="proxyModel.value"
                chips
                multiple
                return-object
                item-title="title"
                item-value="value"
                :items="allItems"
                label="Tags"
                closable-chips
                variant="outlined"
                autocomplete="off"
              >
                <template #chip="{ props: chipProps, item }">
                  <v-chip v-bind="chipProps">
                    <template v-if="isFlagItem(item.raw)" #prepend>
                      <v-icon size="small" class="me-1">{{ item.raw.icon }}</v-icon>
                    </template>
                    {{ item.title }}
                  </v-chip>
                </template>
                <template #item="{ item, props: itemProps }">
                  <v-list-item v-bind="itemProps">
                    <template #prepend="{ isActive }">
                      <v-list-item-action>
                        <v-checkbox-btn :model-value="isActive" />
                      </v-list-item-action>
                    </template>
                    <template #append>
                      <v-icon v-if="item.raw.isFlag">{{ item.raw.icon }}</v-icon>
                    </template>
                  </v-list-item>
                </template>
              </v-combobox>
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
/**
 * This modal is used for editing the tags of a single photo on the single photo view page.
 */
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useModalStore } from '@/store/modalStore'
import { useTagStore } from '@/store/tagStore'
import { getHashIndexDataFromRoute, getIsolationIdByRoute } from '@utils/getter'
import { editTags } from '@/api/editTags'
import { editFlags } from '@/api/editFlags'

interface ComboboxItem {
  title: string
  value: string
  isFlag: boolean
  icon?: string
}

type ModelValue = string | ComboboxItem

const FAVORITE_ITEM: ComboboxItem = {
  title: 'Favorite',
  value: 'isFavorite',
  isFlag: true,
  icon: 'mdi-star'
}
const ARCHIVED_ITEM: ComboboxItem = {
  title: 'Archived',
  value: 'isArchived',
  isFlag: true,
  icon: 'mdi-archive-arrow-down'
}

function isFlagItem(v: ModelValue): v is ComboboxItem {
  return typeof v === 'object' && v.isFlag
}

function getTagString(v: ModelValue): string {
  return typeof v === 'string' ? v : v.value
}

const formIsValid = ref(false)
const changedTagsArray = ref<ModelValue[]>([])
const submit = ref<(() => Promise<void>) | undefined>(undefined)

const route = useRoute()
const modalStore = useModalStore('mainId')
const tagStore = useTagStore('mainId')

const allItems = computed<ComboboxItem[]>(() => {
  const tagItems = tagStore.tags.map((t) => ({ title: t.tag, value: t.tag, isFlag: false }))
  return [FAVORITE_ITEM, ARCHIVED_ITEM, ...tagItems]
})

onMounted(() => {
  const useSubmit = (): undefined | (() => Promise<void>) => {
    const initializeResult = getHashIndexDataFromRoute(route)
    if (initializeResult === undefined) {
      console.error(
        "useSubmit Error: Failed to initialize result. 'getHashIndexDataFromRoute(route)' returned undefined."
      )
      return undefined
    }
    const { index, data } = initializeResult
    let defaultTags: string[]

    if (data.type === 'image' || data.type === 'video') {
      defaultTags = data.tags
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    } else if (data.type === 'album') {
      defaultTags = data.tags
    } else {
      console.error("useSubmit Error: 'data' type is not recognized.")
      return undefined
    }

    const defaultIsFavorite = data.isFavorite
    const defaultIsArchived = data.isArchived

    changedTagsArray.value = [...defaultTags]
    if (data.isFavorite) changedTagsArray.value.push(FAVORITE_ITEM)
    if (data.isArchived) changedTagsArray.value.push(ARCHIVED_ITEM)

    const innerSubmit = async () => {
      const currentValues = changedTagsArray.value
      const currentTags = currentValues.filter((v) => !isFlagItem(v)).map(getTagString)

      const hashArray: number[] = [index]
      const addTagsArray = currentTags.filter((tag) => !defaultTags.includes(tag))
      const removeTagsArray = defaultTags.filter((tag) => !currentTags.includes(tag))

      const isolationId = getIsolationIdByRoute(route)

      modalStore.showEditTagsModal = false

      if (addTagsArray.length > 0 || removeTagsArray.length > 0) {
        await editTags(hashArray, addTagsArray, removeTagsArray, isolationId)
      }

      const isFavoriteNow = currentValues.some(
        (v) => isFlagItem(v) && v.value === 'isFavorite'
      )
      const isArchivedNow = currentValues.some(
        (v) => isFlagItem(v) && v.value === 'isArchived'
      )
      const flagChanges: { isFavorite?: boolean; isArchived?: boolean } = {}
      if (defaultIsFavorite !== isFavoriteNow) flagChanges.isFavorite = isFavoriteNow
      if (defaultIsArchived !== isArchivedNow) flagChanges.isArchived = isArchivedNow
      if (Object.keys(flagChanges).length > 0) {
        await editFlags(hashArray, flagChanges, isolationId)
      }
    }
    return innerSubmit
  }
  submit.value = useSubmit()
})
</script>
