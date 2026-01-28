<template>
  <PageTemplate preset="card" width="pane" :ready="tagStore.fetched">
    <template #content>
      <v-table hover>
        <thead>
          <tr>
            <th>tag</th>
            <th>number of items</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="tagsData in tagStore.tags" :key="tagsData.tag">
            <td class="key-cell">
              <v-btn
                @click="searchByTag(tagsData.tag, router)"
                slim
                class="text-caption"
                variant="tonal"
              >
                {{ tagsData.tag }}
              </v-btn>
            </td>
            <td>{{ tagsData.number }}</td>
          </tr>
        </tbody>
      </v-table>
    </template>
  </PageTemplate>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useTagStore } from '@/store/tagStore'
import { useInitializedStore } from '@/store/initializedStore'
import { searchByTag } from '@utils/getter'
import PageTemplate from './PageLayout/PageTemplate.vue'

const initializedStore = useInitializedStore('mainId')
const tagStore = useTagStore('mainId')
const router = useRouter()

onMounted(async () => {
  if (!tagStore.fetched) {
    await tagStore.fetchTags()
  }
  initializedStore.initialized = true
})

onBeforeUnmount(() => {
  initializedStore.initialized = false
})
</script>
