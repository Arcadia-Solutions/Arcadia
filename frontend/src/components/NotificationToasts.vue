<template>
  <Toast position="bottom-right" group="bottom-right">
    <template #message="slotProps">
      <div class="p-toast-detail notification">
        {{ slotProps.message.detail }}
        <br />
        <RouterLink :to="viewRoutes[slotProps.message.summary!]">{{ t('general.view') }}</RouterLink>
        <span v-if="slotProps.message.summary === 'announce_error'" class="dismiss-link" @click="$toast.remove(slotProps.message)" aria-label="Close">
          {{ t('general.dismiss') }}
        </span>
      </div>
    </template>
  </Toast>
</template>

<script setup lang="ts">
import { removeToastGroup, showToast } from '@/main'
import { useNotificationsStore } from '@/stores/notifications'
import { useUserStore } from '@/stores/user'
import { Toast } from 'primevue'
import { computed, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'

const notificationsStore = useNotificationsStore()
const userStore = useUserStore()
const { t } = useI18n()

const viewRoutes = computed<Record<string, string>>(() => ({
  announcement: '/forum/sub-category/1',
  conversation: '/conversations',
  forum_sub_category_thread: '/notifications?tab=forum_sub_category_threads',
  forum_thread_post: '/notifications?tab=forum_thread_posts',
  title_group_comment: '/notifications?tab=title_group_comments',
  title_group_torrent: '/notifications?tab=title_group_torrents',
  reseed_request: '/notifications?tab=reseed_requests',
  artist_title_group: '/notifications?tab=artist_title_groups',
  collage: '/notifications?tab=collages',
  torrent_request_comment: '/notifications?tab=torrent_request_comments',
  staff_pm: userStore.permissions.includes('read_staff_pm') ? '/staff-dashboard?tab=staffPms' : '/staff-pms',
  torrent_deletion: '/notifications?tab=torrent_deletions',
  announce_error: '/notifications?tab=announce_errors',
}))

watch(
  [
    () => notificationsStore.announcements,
    () => notificationsStore.conversations,
    () => notificationsStore.forum_sub_category_threads,
    () => notificationsStore.forum_thread_posts,
    () => notificationsStore.title_group_comments,
    () => notificationsStore.title_group_torrents,
    () => notificationsStore.reseed_requests,
    () => notificationsStore.artist_title_groups,
    () => notificationsStore.collages,
    () => notificationsStore.torrent_request_comments,
    () => notificationsStore.staff_pm_messages,
    () => notificationsStore.torrent_deletions,
    () => notificationsStore.announce_errors,
  ],
  async ([
    newAnnouncements,
    newConversations,
    newForumSubCategoryThreads,
    newForumThreadPosts,
    newTitleGroupComments,
    newTitleGroupTorrents,
    newReseedRequests,
    newArtistTitleGroups,
    newCollages,
    newTorrentRequestComments,
    newStaffPms,
    newTorrentDeletions,
    newAnnounceErrors,
  ]) => {
    removeToastGroup('bottom-right')
    await nextTick()

    if (newAnnouncements > 0) {
      showToast('announcement', t('user.announcements', [newAnnouncements]), 'info', undefined, false, 'bottom-right')
    }

    if (newConversations > 0) {
      showToast('conversation', t('user.conversations', [newConversations]), 'info', undefined, false, 'bottom-right')
    }

    if (newForumSubCategoryThreads > 0) {
      showToast('forum_sub_category_thread', t('user.forum_sub_category_threads', [newForumSubCategoryThreads]), 'info', undefined, false, 'bottom-right')
    }

    if (newForumThreadPosts > 0) {
      showToast('forum_thread_post', t('user.forum_thread_posts', [newForumThreadPosts]), 'info', undefined, false, 'bottom-right')
    }

    if (newTitleGroupComments > 0) {
      showToast('title_group_comment', t('user.title_group_comments', [newTitleGroupComments]), 'info', undefined, false, 'bottom-right')
    }

    if (newTitleGroupTorrents > 0) {
      showToast('title_group_torrent', t('user.title_group_torrents', [newTitleGroupTorrents]), 'info', undefined, false, 'bottom-right')
    }

    if (newReseedRequests > 0) {
      showToast('reseed_request', t('user.reseed_requests', [newReseedRequests]), 'info', undefined, false, 'bottom-right')
    }

    if (newArtistTitleGroups > 0) {
      showToast('artist_title_group', t('user.artist_title_groups', [newArtistTitleGroups]), 'info', undefined, false, 'bottom-right')
    }

    if (newCollages > 0) {
      showToast('collage', t('user.collages', [newCollages]), 'info', undefined, false, 'bottom-right')
    }

    if (newTorrentRequestComments > 0) {
      showToast('torrent_request_comment', t('user.torrent_request_comments', [newTorrentRequestComments]), 'info', undefined, false, 'bottom-right')
    }

    if (newStaffPms > 0) {
      showToast('staff_pm', t('user.staff_pm_messages', [newStaffPms]), 'info', undefined, false, 'bottom-right')
    }

    if (newTorrentDeletions > 0) {
      showToast('torrent_deletion', t('user.torrent_deletions', [newTorrentDeletions]), 'info', undefined, false, 'bottom-right')
    }

    if (newAnnounceErrors > 0) {
      showToast('announce_error', t('user.announce_errors', [newAnnounceErrors], newAnnounceErrors), 'warn', undefined, false, 'bottom-right')
    }
  },
  { immediate: true },
)
</script>
<style scoped>
.notification {
  margin-bottom: -3px;
}
.dismiss-link {
  margin-left: 8px;
  color: var(--p-primary-color);
  cursor: pointer;
}
</style>
