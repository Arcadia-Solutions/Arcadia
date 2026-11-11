<template>
  <div class="review" :id="`review-${review.id}`">
    <img class="avatar" :src="review.created_by.avatar ?? '/default_user_avatar.png'" :alt="review.created_by.username + '\'s avatar'" />
    <div class="body">
      <div class="header">
        <UsernameEnriched :user="review.created_by" />
        <TimeAgo :date="review.created_at" class="date" />
        <UserRatingDisplay :rating="review.rating" class="rating" />
        <i v-if="userStore.permissions.includes('delete_title_group_review')" v-tooltip.top="t('general.delete')" class="pi pi-trash" @click="emit('delete')" />
      </div>
      <BBCodeRenderer v-if="review.content" :content="review.content" class="content" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import BBCodeRenderer from '../community/BBCodeRenderer.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import TimeAgo from '../TimeAgo.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { useUserStore } from '@/stores/user'
import type { TitleGroupReviewHierarchy } from '@/services/api-schema'

defineProps<{
  review: TitleGroupReviewHierarchy
}>()

const emit = defineEmits<{
  delete: []
}>()

const { t } = useI18n()
const userStore = useUserStore()
</script>

<style scoped>
.review {
  display: flex;
  gap: 10px;
  padding: 8px 0;
}
.review:not(:last-child) {
  border-bottom: 1px solid var(--p-content-border-color);
}
.avatar {
  width: 2.5em;
  height: 2.5em;
  border-radius: 50%;
  object-fit: cover;
}
.body {
  flex: 1;
  min-width: 0;
}
.header {
  display: flex;
  align-items: center;
  gap: 8px;
}
.date {
  font-size: 0.85em;
  opacity: 0.7;
}
.rating {
  margin-left: auto;
  font-weight: bold;
  font-size: 1.1em;
}
.pi-trash {
  cursor: pointer;
}
.content {
  margin-top: 4px;
}
</style>
