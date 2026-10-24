<template>
  <DataTable :value="searchResults" size="small" tableStyle="table-layout: fixed">
    <Column :header="t('title_group.title')" style="width: 25%">
      <template #body="slotProps">
        <RouterLink :to="`/title-group/${slotProps.data.title_group_id}`">
          {{ slotProps.data.title_group_name }}
        </RouterLink>
      </template>
    </Column>
    <Column :header="t('general.comment')">
      <template #body="slotProps">
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div class="left" style="overflow: hidden">
            <span style="text-overflow: ellipsis; white-space: nowrap; overflow: hidden; display: block">
              {{ slotProps.data.content }}
            </span>
          </div>
          <div class="right" style="width: 10em; text-align: right">
            {{ t('general.by') }}
            <RouterLink :to="`/user/${slotProps.data.created_by.id}`">
              {{ slotProps.data.created_by.username }}
            </RouterLink>
          </div>
        </div>
      </template>
    </Column>
    <Column :header="t('general.time')" style="width: 7em">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.created_at" />
      </template>
    </Column>
  </DataTable>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Column, DataTable } from 'primevue'
import { useI18n } from 'vue-i18n'
import type { TitleGroupCommentSearchResult } from '@/services/api-schema'

const { t } = useI18n()

defineProps<{
  searchResults: TitleGroupCommentSearchResult[]
}>()
</script>

<style scoped></style>
