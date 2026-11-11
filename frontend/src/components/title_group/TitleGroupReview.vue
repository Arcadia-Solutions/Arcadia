<template>
  <div class="review" :id="`review-${review.id}`">
    <div class="header">
      <img class="avatar" :src="review.created_by.avatar ?? '/default_user_avatar.png'" :alt="review.created_by.username + '\'s avatar'" />
      <div class="author">
        <UsernameEnriched :user="review.created_by" />
        <TimeAgo :date="review.created_at" class="date" />
        <UserRatingDisplay :rating="review.rating" class="rating" />
      </div>
      <i v-if="userStore.permissions.includes('delete_title_group_review')" v-tooltip.top="t('general.delete')" class="pi pi-trash" @click="emit('delete')" />
    </div>
    <BBCodeRenderer v-if="review.content" :content="review.content" class="content" />
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
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--p-content-border-color);
  border-radius: 7px;
  min-width: 0;
}
.header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.avatar {
  width: 5em;
  height: 5em;
  border-radius: 7px;
  object-fit: cover;
}
.author {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
  min-width: 0;
}
.date {
  font-size: 0.85em;
  opacity: 0.7;
}
.rating {
  font-weight: bold;
  font-size: 1.2em;
}
.pi-trash {
  align-self: flex-start;
  cursor: pointer;
}
.content {
  overflow-wrap: anywhere;
}
</style>
