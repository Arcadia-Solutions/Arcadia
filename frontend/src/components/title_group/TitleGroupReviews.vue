<template>
  <div v-if="reviews.length === 0" class="no-rating">{{ t('title_group.no_ratings') }}</div>
  <div v-for="review in reviews" :key="review.id" :id="`review-${review.id}`">
    <GeneralComment
      :comment="review"
      :hasEditPermission="false"
      :hasDeletePermission="userStore.permissions.includes('delete_title_group_review')"
      :deleteCommentMethod="deleteTitleGroupReview"
      @commentDeleted="emit('deleted', $event)"
    >
      <template #top-right>
        <UserRatingDisplay :rating="review.rating" />
      </template>
    </GeneralComment>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GeneralComment from '../community/GeneralComment.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { useUserStore } from '@/stores/user'
import { deleteTitleGroupReview, type TitleGroupReviewHierarchy } from '@/services/api-schema'

defineProps<{
  reviews: TitleGroupReviewHierarchy[]
}>()

const emit = defineEmits<{
  deleted: [number]
}>()

const { t } = useI18n()
const userStore = useUserStore()
</script>

<style scoped>
.no-rating {
  opacity: 0.7;
}
</style>
