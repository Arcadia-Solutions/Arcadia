<template>
  <DataTable :value="searchResults" size="small" tableStyle="table-layout: fixed">
    <Column :header="`${t('forum.category')} > ${t('forum.subcategory')}`" style="width: 17%">
      <template #body="slotProps">
        <RouterLink to="/forum">
          {{ slotProps.data.category_name }}
        </RouterLink>
        &gt;
        <RouterLink :to="`/forum/sub-category/${slotProps.data.sub_category_id}`">
          {{ slotProps.data.sub_category_name }}
        </RouterLink>
      </template>
    </Column>
    <Column :header="t('forum.thread')">
      <template #body="slotProps">
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div class="left" style="overflow: hidden; min-width: 0">
            <div class="top">
              <ForumThreadName :threadId="slotProps.data.thread_id" :threadName="slotProps.data.thread_name" :postId="slotProps.data.post_id" />
            </div>
            <div class="bottom" style="font-size: 0.8em; text-overflow: ellipsis; white-space: nowrap; overflow: hidden">
              {{ slotProps.data.post }}
            </div>
          </div>
          <div class="right" style="max-width: 12em; flex-shrink: 0; text-align: right; white-space: nowrap; overflow: hidden; text-overflow: ellipsis">
            {{ t('general.by') }}
            <RouterLink :to="`/user/${slotProps.data.post_created_by_id}`">
              {{ slotProps.data.post_created_by_username }}
            </RouterLink>
          </div>
        </div>
      </template>
    </Column>
    <Column :header="t('general.time')" style="width: 7em">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.post_created_at" />
      </template>
    </Column>
  </DataTable>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Column, DataTable } from 'primevue'
import { useI18n } from 'vue-i18n'
import ForumThreadName from './ForumThreadName.vue'
import type { ForumSearchResult } from '@/services/api-schema'

const { t } = useI18n()

defineProps<{
  searchResults: ForumSearchResult[]
}>()
</script>

<style scoped></style>
