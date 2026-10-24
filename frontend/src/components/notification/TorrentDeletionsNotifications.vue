<template>
  <div v-if="visibleNotifications.length > 0">
    <Button
      :label="t('notification.mark_selected_as_read')"
      icon="pi pi-check"
      size="small"
      :disabled="selected.length === 0"
      style="margin-bottom: 8px"
      @click="clearSelected"
    />
    <DataTable v-model:selection="selected" :value="visibleNotifications" data-key="torrent_id" size="small">
      <Column selection-mode="multiple" header-style="width: 3rem" />
      <Column :header="t('title_group.title')">
        <template #body="slotProps">
          {{ slotProps.data.title_group_name }}
        </template>
      </Column>
      <Column :header="t('notification.deletion_reason')">
        <template #body="slotProps">
          {{ t(`notification.deletion_reason_${slotProps.data.deletion_reason}`) }}
        </template>
      </Column>
      <Column :header="t('notification.extra_information')">
        <template #body="slotProps">
          {{ slotProps.data.extra_information ?? '' }}
        </template>
      </Column>
      <Column :header="t('notification.replacement')">
        <template #body="slotProps">
          <RouterLink v-if="slotProps.data.replacement_torrent_id" :to="`/torrent/${slotProps.data.replacement_torrent_id}`">
            #{{ slotProps.data.replacement_torrent_id }}
          </RouterLink>
        </template>
      </Column>
      <Column :header="t('notification.deleted_at')">
        <template #body="slotProps">
          <TimeAgo :date="slotProps.data.deleted_at" />
        </template>
      </Column>
    </DataTable>
  </div>
  <div v-else class="wrapper-center">
    {{ t('notification.no_notification') }}
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Button, Column, DataTable } from 'primevue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useNotificationsStore } from '@/stores/notifications'
import { markTorrentDeletionNotificationsAsRead, type NotificationTorrentDeletion } from '@/services/api-schema'

const props = defineProps<{
  notifications: NotificationTorrentDeletion[]
}>()

const { t } = useI18n()
const notificationsStore = useNotificationsStore()

const dismissed = ref<Set<number>>(new Set())
const selected = ref<NotificationTorrentDeletion[]>([])

const visibleNotifications = computed(() => props.notifications.filter((n) => !dismissed.value.has(n.torrent_id)))

const clearSelected = () => {
  const selectedItems = selected.value
  if (selectedItems.length === 0) return

  const torrentIds = selectedItems.map((n) => n.torrent_id)
  const unreadCount = selectedItems.filter((n) => !n.read_status).length

  markTorrentDeletionNotificationsAsRead({ torrent_ids: torrentIds }).then(() => {
    selectedItems.forEach((item) => {
      item.read_status = true
      dismissed.value.add(item.torrent_id)
    })
    notificationsStore.torrent_deletions = Math.max(0, notificationsStore.torrent_deletions - unreadCount)
    selected.value = []
  })
}
</script>
