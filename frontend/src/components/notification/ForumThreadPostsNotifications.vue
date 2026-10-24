<template>
  <DataTable v-if="notifications.length > 0" :value="notifications" size="small">
    <Column :header="t('forum.thread_name')">
      <template #body="slotProps">
        <ForumThreadName
          :threadName="slotProps.data.forum_thread_name"
          :threadId="slotProps.data.forum_thread_id"
          :postId="slotProps.data.forum_post_id"
          @click="slotProps.data.is_read ? null : (notificationsStore.forum_thread_posts -= 1)"
        />
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
import ForumThreadName from '../forum/ForumThreadName.vue'
import { useNotificationsStore } from '@/stores/notifications'
import type { NotificationForumThreadPost } from '@/services/api-schema'

defineProps<{
  notifications: NotificationForumThreadPost[]
}>()

const notificationsStore = useNotificationsStore()
const { t } = useI18n()
</script>
