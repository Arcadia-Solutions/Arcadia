<template>
  <div class="rating-input">
    <Slider v-model="rating" :min="0" :max="100" class="slider" />
    <UserRatingDisplay :rating />
  </div>
  <BBCodeEditor :initialValue="ownReview?.review ?? ''" :label="t('title_group.review_optional')" @valueChange="review = $event">
    <template #buttons>
      <Button :label="t(ownReview ? 'general.edit' : 'general.submit')" size="small" :loading="saving" @click="save" />
    </template>
  </BBCodeEditor>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Slider } from 'primevue'
import BBCodeEditor from '../community/BBCodeEditor.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { useUserStore } from '@/stores/user'
import { showToast } from '@/main'
import { createTitleGroupReview, editTitleGroupReview, type TitleGroupReviewHierarchy } from '@/services/api-schema'

const props = defineProps<{
  titleGroupId: number
  ownReview?: TitleGroupReviewHierarchy
}>()

const emit = defineEmits<{
  saved: [TitleGroupReviewHierarchy]
}>()

const { t } = useI18n()
const userStore = useUserStore()

// only read once: the parent mounts this form after the user's own review is fetched
const rating = ref(props.ownReview?.rating ?? 50)
const review = ref(props.ownReview?.review ?? '')
const saving = ref(false)

const save = () => {
  saving.value = true
  const data = { rating: rating.value, review: review.value.trim() || null }
  ;(props.ownReview
    ? editTitleGroupReview({ id: props.ownReview.id, EditedTitleGroupReview: data })
    : createTitleGroupReview({ ...data, title_group_id: props.titleGroupId })
  )
    .then((saved) => {
      const { id, username, class_name, banned, avatar, warned, custom_title } = userStore
      emit('saved', { ...saved, user: { id, username, class_name, banned, avatar, warned, custom_title } })
      showToast('', t('title_group.review_saved_success'), 'success', 2000)
    })
    .finally(() => (saving.value = false))
}
</script>

<style scoped>
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
