<template>
  <DataTable :value="searchResults" size="small" tableStyle="table-layout: fixed">
    <Column :header="t('title_group.title')" style="width: 25%">
      <template #body="slotProps">
        <RouterLink :to="`/title-group/${slotProps.data.title_group.id}`">
          {{ slotProps.data.title_group.name }}
        </RouterLink>
      </template>
    </Column>
    <Column :header="t('title_group.rating')" style="width: 7em">
      <template #body="slotProps">
        <UserRatingDisplay :rating="slotProps.data.rating" />
      </template>
    </Column>
    <Column :header="t('general.comment')">
      <template #body="slotProps">
        <div style="display: flex; justify-content: space-between; align-items: center">
          <div class="left" style="overflow: hidden">
            <span style="text-overflow: ellipsis; white-space: nowrap; overflow: hidden; display: block">
              {{ slotProps.data.comment }}
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
import UserRatingDisplay from './UserRatingDisplay.vue'
import { Column, DataTable } from 'primevue'
import { useI18n } from 'vue-i18n'
import type { TitleGroupRatingWithTitleGroup } from '@/services/api-schema'

const { t } = useI18n()

defineProps<{
  searchResults: TitleGroupRatingWithTitleGroup[]
}>()
</script>
