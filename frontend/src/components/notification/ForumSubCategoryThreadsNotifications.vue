<template>
  <DataTable v-if="notifications.length > 0" :value="notifications" size="small">
    <Column :header="t('forum.thread_name')">
      <template #body="slotProps">
        <RouterLink
          :to="`/forum/thread/${slotProps.data.forum_thread_id}`"
          @click="slotProps.data.read_status ? null : (notificationsStore.forum_sub_category_threads -= 1)"
        >
          {{ slotProps.data.forum_thread_name }}
        </RouterLink>
        <span class="sub-category-label">
          {{ t('general.in') }}
          <RouterLink :to="`/forum/sub-category/${slotProps.data.forum_sub_category_id}`">
            {{ slotProps.data.forum_sub_category_name }}
          </RouterLink>
        </span>
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
import { useNotificationsStore } from '@/stores/notifications'
import type { NotificationForumSubCategoryThread } from '@/services/api-schema'

defineProps<{
  notifications: NotificationForumSubCategoryThread[]
}>()

const notificationsStore = useNotificationsStore()
const { t } = useI18n()
</script>

<style scoped>
.sub-category-label {
  margin-left: 5px;
  opacity: 0.7;
}
</style>
