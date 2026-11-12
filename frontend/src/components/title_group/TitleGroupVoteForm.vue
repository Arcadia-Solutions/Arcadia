<template>
  <div class="rating-input">
    <Slider v-model="rating" :min="0" :max="100" class="slider" />
    <UserRatingDisplay :rating class="rating-preview" />
  </div>
  <BBCodeEditor :initialValue="ownRating?.comment ?? ''" :label="t('title_group.comment_optional')" @valueChange="comment = $event">
    <template #buttons>
      <Button :label="t(ownRating ? 'general.edit' : 'general.submit')" size="small" :loading="saving" @click="save" />
    </template>
  </BBCodeEditor>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Slider } from 'primevue'
import BBCodeEditor from '../community/BBCodeEditor.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { showToast } from '@/main'
import { createTitleGroupRating, editTitleGroupRating, type TitleGroupRating } from '@/services/api-schema'

const props = defineProps<{
  titleGroupId: number
  ownRating: TitleGroupRating | null
}>()

const emit = defineEmits<{
  saved: [TitleGroupRating]
}>()

const { t } = useI18n()

// only read once: the dialog mounts this form each time it opens
const rating = ref(props.ownRating?.rating ?? 50)
const comment = ref(props.ownRating?.comment ?? '')
const saving = ref(false)

const save = () => {
  saving.value = true
  const data = { rating: rating.value, comment: comment.value.trim() || null }
  ;(props.ownRating
    ? editTitleGroupRating({ id: props.ownRating.id, EditedTitleGroupRating: data })
    : createTitleGroupRating({ ...data, title_group_id: props.titleGroupId })
  )
    .then((saved) => {
      emit('saved', saved)
      showToast('', t('title_group.rating_saved_success'), 'success', 2000)
    })
    .finally(() => (saving.value = false))
}
</script>

<style scoped>
.rating-input {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 15px;
  margin-bottom: 15px;
}
.slider {
  width: 300px;
}
.rating-preview {
  font-size: 1.5em;
  font-weight: bold;
}
</style>
