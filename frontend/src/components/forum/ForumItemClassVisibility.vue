<template>
  <div v-if="classVisibility.hidden_from_class_names.length === 0">{{ t('forum.visible_to_all_user_classes') }}</div>
  <div v-else class="class-visibility">
    <div>{{ t('forum.visible_to_user_classes', { classes: classVisibility.visible_to_class_names.join(', ') }) }}</div>
    <div>{{ t('forum.hidden_from_user_classes', { classes: classVisibility.hidden_from_class_names.join(', ') }) }}</div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ForumItemClassVisibility } from '@/services/api-schema'

const { t } = useI18n()

defineProps<{
  classVisibility: ForumItemClassVisibility
}>()
</script>

<style scoped>
.class-visibility {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
</style>
