<template>
  <DataTable v-if="notifications.length > 0" :value="notifications" size="small">
    <Column :header="t('artist.artist', 1)">
      <template #body="slotProps">
        <RouterLink :to="`/artist/${slotProps.data.artist_id}`">
          {{ slotProps.data.artist_name }}
        </RouterLink>
      </template>
    </Column>
    <Column :header="t('title_group.title')">
      <template #body="slotProps">
        <div @click="markTitleGroupAsRead(slotProps.data.title_group_id)">
          <RouterLink :to="`/title-group/${slotProps.data.title_group_id}`">
            {{ slotProps.data.title_group_name }}
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
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useNotificationsStore } from '@/stores/notifications'
import type { NotificationArtistTitleGroup } from '@/services/api-schema'

const props = defineProps<{
  notifications: NotificationArtistTitleGroup[]
}>()

const notificationsStore = useNotificationsStore()

// visiting a title group marks every notification of that title group as read on
// the backend, so the badge is decremented by all of them at once
const readTitleGroupIds = ref(new Set<number>())
function markTitleGroupAsRead(titleGroupId: number) {
  if (readTitleGroupIds.value.has(titleGroupId)) return
  readTitleGroupIds.value.add(titleGroupId)
  notificationsStore.artist_title_groups -= props.notifications.filter((n) => n.title_group_id === titleGroupId && !n.read_status).length
}
const { t } = useI18n()
</script>
