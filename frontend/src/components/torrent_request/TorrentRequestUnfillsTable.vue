<template>
  <ContentContainer :containerTitle="t('torrent_request.unfill', 2)">
    <DataTable :value="unfills" size="small">
      <Column :header="t('torrent_request.unfilled_by')">
        <template #body="slotProps">
          <UsernameEnriched :user="slotProps.data.unfilled_by" />
        </template>
      </Column>
      <Column :header="t('torrent_request.unfilled_at')">
        <template #body="slotProps">
          <TimeAgo :date="slotProps.data.unfilled_at" />
        </template>
      </Column>
      <Column :header="t('general.reason')" field="reason" />
      <Column :header="t('torrent.torrent')">
        <template #body="slotProps">
          <RouterLink :to="`/torrent/${slotProps.data.filled_by_torrent_id}`">#{{ slotProps.data.filled_by_torrent_id }}</RouterLink>
        </template>
      </Column>
      <Column :header="t('torrent_request.filled_by')">
        <template #body="slotProps">
          <UsernameEnriched :user="slotProps.data.filled_by_user" />
        </template>
      </Column>
      <Column :header="t('torrent_request.filled_at')">
        <template #body="slotProps">
          <TimeAgo :date="slotProps.data.filled_at" />
        </template>
      </Column>
    </DataTable>
  </ContentContainer>
</template>

<script setup lang="ts">
import ContentContainer from '@/components/ContentContainer.vue'
import TimeAgo from '@/components/TimeAgo.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import type { TorrentRequestUnfillHierarchy } from '@/services/api-schema'
import { Column, DataTable } from 'primevue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  unfills: TorrentRequestUnfillHierarchy[]
}>()
</script>
