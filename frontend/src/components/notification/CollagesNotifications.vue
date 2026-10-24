<template>
  <DataTable v-if="notifications.length > 0" :value="notifications" size="small">
    <Column :header="t('collage.collage')">
      <template #body="slotProps">
        <div @click="markCollageAsRead(slotProps.data.collage_id)">
          <RouterLink :to="`/collage/${slotProps.data.collage_id}`">
            {{ slotProps.data.collage_name }}
          </RouterLink>
        </div>
      </template>
    </Column>
    <Column :header="t('title_group.title')">
      <template #body="slotProps">
        <RouterLink :to="`/title-group/${slotProps.data.title_group_id}`">
          {{ slotProps.data.title_group_name }}
        </RouterLink>
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
import type { NotificationCollage } from '@/services/api-schema'
const props = defineProps<{
  notifications: NotificationCollage[]
}>()
const notificationsStore = useNotificationsStore()
// visiting a collage marks every notification of that collage as read on
// the backend, so the badge is decremented by all of them at once
const readCollageIds = ref(new Set<number>())
function markCollageAsRead(collageId: number) {
  if (readCollageIds.value.has(collageId)) return
  readCollageIds.value.add(collageId)
  notificationsStore.collages -= props.notifications.filter((n) => n.collage_id === collageId && !n.read_status).length
}
const { t } = useI18n()
</script>
