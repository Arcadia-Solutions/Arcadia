<template>
  <DataTable v-if="visibleNotifications.length > 0" :value="visibleNotifications" data-key="torrent_id" size="small">
    <Column :header="t('title_group.title')">
      <template #body="slotProps">
        <div>
          <RouterLink :to="`/title-group/${slotProps.data.title_group_id}`">
            {{ slotProps.data.title_group_name }}
          </RouterLink>
          <RouterLink :to="`/title-group/${slotProps.data.title_group_id}?torrentId=${slotProps.data.torrent_id}`">
            <i class="pi pi-arrow-right" style="color: white; font-size: 0.7em; margin-left: 5px" />
          </RouterLink>
        </div>
      </template>
    </Column>
    <Column :header="t('reseed_request.requested_by')">
      <template #body="slotProps">
        <span v-for="(user, index) in slotProps.data.requested_by" :key="user.id">
          <UsernameEnriched :user="user" /><span v-if="index < slotProps.data.requested_by.length - 1">, </span>
        </span>
      </template>
    </Column>
    <Column :header="t('notification.notified_at')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.created_at" />
      </template>
    </Column>
    <Column header-style="width: 3rem">
      <template #body="slotProps">
        <i v-tooltip.top="t('notification.mark_as_read')" class="pi pi-eye mark-as-read" @click="markAsRead(slotProps.data.torrent_id)" />
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
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useNotificationsStore } from '@/stores/notifications'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { markReseedRequestNotificationsAsRead, type NotificationReseedRequest } from '@/services/api-schema'

const props = defineProps<{
  notifications: NotificationReseedRequest[]
}>()

// marking a reseed request as read only hides it for this recipient, the row is kept in the
// database and removed by the backend once the torrent is seeded again
const { t } = useI18n()
const notificationsStore = useNotificationsStore()

const dismissed = ref<Set<number>>(new Set())

const visibleNotifications = computed(() => props.notifications.filter((n) => !dismissed.value.has(n.torrent_id)))

const markAsRead = (torrentId: number) => {
  markReseedRequestNotificationsAsRead(torrentId).then(() => {
    dismissed.value.add(torrentId)
    notificationsStore.reseed_requests = Math.max(0, notificationsStore.reseed_requests - 1)
  })
}
</script>

<style scoped>
.mark-as-read {
  cursor: pointer;
}
</style>
