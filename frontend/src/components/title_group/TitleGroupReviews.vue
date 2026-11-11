<template>
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
  <ContentContainer v-if="userStore.permissions.includes('rate_title_groups')" :containerTitle="t('title_group.your_rating')" class="own-review">
    <div class="rating-input">
      <Slider v-model="rating" :min="0" :max="100" class="slider" />
      <UserRatingDisplay :rating />
    </div>
    <BBCodeEditor :initialValue="ownReview?.review ?? ''" :label="t('title_group.review_optional')" @valueChange="review = $event">
      <template #buttons>
        <Button :label="t(ownReview ? 'general.edit' : 'general.submit')" size="small" :loading="saving" @click="save" />
      </template>
    </BBCodeEditor>
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
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Dialog, Slider } from 'primevue'
import ContentContainer from '../ContentContainer.vue'
import BBCodeEditor from '../community/BBCodeEditor.vue'
import BBCodeRenderer from '../community/BBCodeRenderer.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import TimeAgo from '../TimeAgo.vue'
import DeleteDialog from '../DeleteDialog.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { useUserStore } from '@/stores/user'
import { showToast } from '@/main'
import { createTitleGroupReview, deleteTitleGroupReview, editTitleGroupReview, type TitleGroupReviewHierarchy } from '@/services/api-schema'

const props = defineProps<{
  titleGroupId: number
  reviews: TitleGroupReviewHierarchy[]
}>()

const emit = defineEmits<{
  saved: [TitleGroupReviewHierarchy]
  deleted: [number]
}>()

const { t } = useI18n()
const userStore = useUserStore()

const ownReview = computed(() => props.reviews.find((r) => r.user_id === userStore.id))
// only read once: the parent re-creates this component when it shows another title group
const rating = ref(ownReview.value?.rating ?? 50)
const review = ref(ownReview.value?.review ?? '')
const saving = ref(false)
const reviewToDelete = ref<number | null>(null)

const save = () => {
  saving.value = true
  const data = { rating: rating.value, review: review.value.trim() || null }
  ;(ownReview.value
    ? editTitleGroupReview({ id: ownReview.value.id, EditedTitleGroupReview: data })
    : createTitleGroupReview({ ...data, title_group_id: props.titleGroupId })
  )
    .then((saved) => {
      const { id, username, class_name, banned, avatar, warned, custom_title } = userStore
      emit('saved', { ...saved, user: { id, username, class_name, banned, avatar, warned, custom_title } })
      showToast('', t('title_group.review_saved_success'), 'success', 2000)
    })
    .finally(() => (saving.value = false))
}

const reviewDeleted = () => {
  if (reviewToDelete.value !== null) emit('deleted', reviewToDelete.value)
  reviewToDelete.value = null
}
</script>

<style scoped>
.review,
.own-review {
  margin-top: 7px;
}
.date {
  margin-left: 7px;
}
.pi-trash {
  margin-left: 7px;
  cursor: pointer;
}
.rating-input {
  display: flex;
  align-items: center;
  gap: 15px;
  margin-bottom: 15px;
}
.slider {
  width: 300px;
}
</style>
