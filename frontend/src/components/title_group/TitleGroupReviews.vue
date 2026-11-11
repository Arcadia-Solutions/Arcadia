<template>
  <div v-if="reviews.length === 0" class="no-rating">{{ t('title_group.no_ratings') }}</div>
  <ContentContainer v-for="review in reviews" :key="review.id" :id="`review-${review.id}`" class="review">
    <template #top-left>
      <UsernameEnriched :user="review.user" displayAllInfo />
      <TimeAgo :date="review.created_at" class="date" />
    </template>
    <template #top-right>
      <UserRatingDisplay :rating="review.rating" />
      <i
        v-if="userStore.permissions.includes('delete_title_group_review')"
        v-tooltip.top="t('general.delete')"
        class="pi pi-trash"
        @click="reviewToDelete = review.id"
      />
    </template>
    <BBCodeRenderer v-if="review.review" :content="review.review" />
  </ContentContainer>
  <Dialog closeOnEscape modal :header="t('title_group.delete_review')" :visible="reviewToDelete !== null" @update:visible="reviewToDelete = null">
    <DeleteDialog
      v-if="reviewToDelete !== null"
      :message="t('title_group.confirm_delete_review')"
      :action="() => deleteTitleGroupReview(reviewToDelete!)"
      :successMessage="t('title_group.review_deleted_success')"
      @deleted="reviewDeleted"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog } from 'primevue'
import ContentContainer from '../ContentContainer.vue'
import BBCodeRenderer from '../community/BBCodeRenderer.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import TimeAgo from '../TimeAgo.vue'
import DeleteDialog from '../DeleteDialog.vue'
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

const reviewToDelete = ref<number | null>(null)

const reviewDeleted = () => {
  if (reviewToDelete.value !== null) emit('deleted', reviewToDelete.value)
  reviewToDelete.value = null
}
</script>

<style scoped>
.review:not(:first-child) {
  margin-top: 7px;
}
.date {
  margin-left: 7px;
}
.pi-trash {
  margin-left: 7px;
  cursor: pointer;
}
.no-rating {
  opacity: 0.7;
}
</style>
