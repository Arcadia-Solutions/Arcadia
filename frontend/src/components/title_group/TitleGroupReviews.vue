<template>
  <div v-if="reviews.length === 0" class="no-rating">{{ t('title_group.no_ratings') }}</div>
  <div class="reviews">
    <TitleGroupReview v-for="review in reviews" :key="review.id" :review @delete="reviewToDelete = review" />
  </div>
  <Dialog closeOnEscape modal :header="t('title_group.delete_review')" :visible="reviewToDelete !== null" @update:visible="reviewToDelete = null">
    <DeleteDialog
      v-if="reviewToDelete !== null"
      :message="t('title_group.confirm_delete_review')"
      :action="() => deleteTitleGroupReview(reviewToDelete!.id)"
      :successMessage="t('title_group.review_deleted_success')"
      @deleted="reviewDeleted"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog } from 'primevue'
import DeleteDialog from '../DeleteDialog.vue'
import TitleGroupReview from './TitleGroupReview.vue'
import { deleteTitleGroupReview, type TitleGroupReviewHierarchy } from '@/services/api-schema'

defineProps<{
  reviews: TitleGroupReviewHierarchy[]
}>()

const emit = defineEmits<{
  deleted: [TitleGroupReviewHierarchy]
}>()

const { t } = useI18n()

const reviewToDelete = ref<TitleGroupReviewHierarchy | null>(null)

const reviewDeleted = () => {
  if (reviewToDelete.value !== null) emit('deleted', reviewToDelete.value)
  reviewToDelete.value = null
}
</script>

<style scoped>
.no-rating {
  opacity: 0.7;
}
.reviews {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  align-items: start;
  gap: 10px;
}
@media (max-width: 767px) {
  .reviews {
    grid-template-columns: 1fr;
  }
}
</style>
