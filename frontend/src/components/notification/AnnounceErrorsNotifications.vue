<template>
  <Message severity="info" icon="pi pi-info-circle" :closable="false">
    {{ t('notification.announce_errors_info') }}
  </Message>
  <DataTable v-if="notifications.length > 0" :value="notifications" size="small">
    <Column :header="t('title_group.title')">
      <template #body="slotProps">
        <RouterLink v-if="slotProps.data.title_group_id" :to="`/title-group/${slotProps.data.title_group_id}`">
          {{ slotProps.data.title_group_name }}
        </RouterLink>
        <template v-else-if="slotProps.data.title_group_name">{{ slotProps.data.title_group_name }}</template>
        <template v-else>{{ t('notification.unknown_torrent', [slotProps.data.info_hash]) }}</template>
      </template>
    </Column>
    <Column :header="t('notification.announce_error')">
      <template #body="slotProps">
        {{ t(`notification.announce_error_${slotProps.data.error_code}`, [publicArcadiaSettings.bonus_points_alias]) }}
      </template>
    </Column>
    <Column :header="t('notification.occurrences')" field="occurrences" />
    <Column :header="t('notification.first_seen_at')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.first_seen_at" />
      </template>
    </Column>
    <Column :header="t('notification.last_seen_at')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.last_seen_at" />
      </template>
    </Column>
  </DataTable>
  <div v-else class="wrapper-center">
    {{ t('notification.no_notification') }}
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Column, DataTable, Message } from 'primevue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import type { NotificationAnnounceError } from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

defineProps<{
  notifications: NotificationAnnounceError[]
}>()

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
</script>
