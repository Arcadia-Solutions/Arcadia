<template>
  <DataTable v-if="notifications.length > 0" :value="notifications" size="small">
    <Column :header="t('torrent_request.request')">
      <template #body="slotProps">
        <div @click="slotProps.data.read_status ? null : (notificationsStore.torrent_request_comments -= 1)">
          <RouterLink :to="`/torrent-request/${slotProps.data.torrent_request_id}`">
            {{ slotProps.data.title_group_name }}
          </RouterLink>
          <RouterLink
            :to="`/torrent-request/${slotProps.data.torrent_request_id}?post_id=${slotProps.data.torrent_request_comment_id}#post-${slotProps.data.torrent_request_comment_id}`"
          >
            <i class="pi pi-arrow-right" style="color: white; font-size: 0.7em; margin-left: 5px" />
          </RouterLink>
        </div>
      </template>
    </Column>
    <Column :header="t('notification.notified_at')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.created_at" />
      </template>
    </Column>
  </DataTable>
  <div v-else class="wrapper-center">
    {{ t('notification.no_notification') }}
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Column, DataTable } from 'primevue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useNotificationsStore } from '@/stores/notifications'
import type { NotificationTorrentRequestComment } from '@/services/api-schema'

defineProps<{
  notifications: NotificationTorrentRequestComment[]
}>()

const notificationsStore = useNotificationsStore()
const { t } = useI18n()
</script>
