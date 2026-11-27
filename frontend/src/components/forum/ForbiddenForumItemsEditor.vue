<template>
  <div class="forbidden-forum-items-editor">
    <FloatLabel>
      <MultiSelect v-model="categoryIds" :options="categoryOptions" optionLabel="name" optionValue="id" display="chip" filter size="small" class="select" />
      <label>{{ t('user_class.hidden_forum_categories') }}</label>
    </FloatLabel>
    <FloatLabel>
      <MultiSelect
        v-model="subCategoryIds"
        :options="subCategoryOptions"
        optionLabel="label"
        optionValue="id"
        display="chip"
        filter
        size="small"
        class="select"
      />
      <label>{{ t('user_class.hidden_forum_sub_categories') }}</label>
    </FloatLabel>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { FloatLabel, MultiSelect } from 'primevue'
import { getForum, type ForumCategoryHierarchy } from '@/services/api-schema'

const { t } = useI18n()

const categoryIds = defineModel<number[]>('categoryIds', { required: true })
const subCategoryIds = defineModel<number[]>('subCategoryIds', { required: true })

const forumCategories = ref<ForumCategoryHierarchy[]>([])

const categoryOptions = computed(() => forumCategories.value.map((category) => ({ id: category.id, name: category.name })))
const subCategoryOptions = computed(() =>
  forumCategories.value.flatMap((category) =>
    category.sub_categories.map((subCategory) => ({ id: subCategory.id, label: `${category.name} > ${subCategory.name}` })),
  ),
)

onMounted(() => {
  getForum().then((forumOverview) => (forumCategories.value = forumOverview.forum_categories))
})
</script>

<style scoped>
.forbidden-forum-items-editor {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.select {
  width: 100%;
}
</style>
