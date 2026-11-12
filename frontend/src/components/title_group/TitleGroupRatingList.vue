<template>
  <div v-if="ratings.length === 0" class="no-rating">{{ t('title_group.no_ratings') }}</div>
  <div class="ratings">
    <TitleGroupRating v-for="rating in ratings" :key="rating.id" :rating @delete="ratingToDelete = rating" />
  </div>
  <Dialog closeOnEscape modal :header="t('title_group.delete_rating')" :visible="ratingToDelete !== null" @update:visible="ratingToDelete = null">
    <DeleteDialog
      v-if="ratingToDelete !== null"
      :message="t('title_group.confirm_delete_rating')"
      :action="() => deleteTitleGroupRating(ratingToDelete!.id)"
      :successMessage="t('title_group.rating_deleted_success')"
      @deleted="ratingDeleted"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Dialog } from 'primevue'
import DeleteDialog from '../DeleteDialog.vue'
import TitleGroupRating from './TitleGroupRating.vue'
import { deleteTitleGroupRating, type TitleGroupRatingHierarchy } from '@/services/api-schema'

defineProps<{
  ratings: TitleGroupRatingHierarchy[]
}>()

const emit = defineEmits<{
  deleted: [TitleGroupRatingHierarchy]
}>()

const { t } = useI18n()

const ratingToDelete = ref<TitleGroupRatingHierarchy | null>(null)

const ratingDeleted = () => {
  if (ratingToDelete.value !== null) emit('deleted', ratingToDelete.value)
  ratingToDelete.value = null
}
</script>

<style scoped>
.no-rating {
  opacity: 0.7;
}
.ratings {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  align-items: start;
  gap: 10px;
}
@media (max-width: 767px) {
  .ratings {
    grid-template-columns: 1fr;
  }
}
</style>
