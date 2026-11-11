<template>
  <ContentContainer class="review-filters-container">
    <div class="review-filters">
      <label>
        {{ t('title_group.minimum_rating') }}
        <InputNumber v-model="filters.minimum_rating" :min="0" :max="100" showButtons :inputStyle="{ width: '6.5em' }" size="small" />
      </label>
      <label>
        {{ t('title_group.maximum_rating') }}
        <InputNumber v-model="filters.maximum_rating" :min="0" :max="100" showButtons :inputStyle="{ width: '6.5em' }" size="small" />
      </label>
      <label v-if="!hideSorting">
        {{ t('general.sort_by') }}
        <Select
          v-model="filters.order_by_column"
          :options="[
            { label: t('general.created_at'), value: TitleGroupReviewSearchOrderByColumn.CreatedAt },
            { label: t('title_group.rating'), value: TitleGroupReviewSearchOrderByColumn.Rating },
          ]"
          optionLabel="label"
          optionValue="value"
          size="small"
        />
      </label>
      <label v-if="!hideSorting">
        {{ t('general.order_by') }}
        <Select v-model="filters.order_by_direction" :options="getOrderByDirectionOptions(t)" optionLabel="label" optionValue="value" size="small" />
      </label>
      <InputText
        v-model="filters.content"
        :placeholder="t('title_group.search_review_text')"
        size="small"
        class="content-search"
        @keyup.enter="emit('search')"
      />
      <Button :label="t('general.search')" size="small" :loading @click="emit('search')" />
    </div>
  </ContentContainer>
</template>

<script setup lang="ts">
import { Button, InputNumber, InputText, Select } from 'primevue'
import { useI18n } from 'vue-i18n'
import ContentContainer from '../ContentContainer.vue'
import { getOrderByDirectionOptions } from '@/services/helpers'
import { OrderByDirection, TitleGroupReviewSearchOrderByColumn } from '@/services/api-schema'

export type TitleGroupReviewFilters = {
  minimum_rating: number | null
  maximum_rating: number | null
  content: string
  order_by_column: TitleGroupReviewSearchOrderByColumn
  order_by_direction: OrderByDirection
}

defineProps<{ loading: boolean; hideSorting?: boolean }>()
const filters = defineModel<TitleGroupReviewFilters>({ required: true })
const emit = defineEmits<{ search: [] }>()

const { t } = useI18n()
</script>

<style scoped>
.content-search {
  flex: 1;
  min-width: 15em;
}
.review-filters-container {
  margin-bottom: 10px;
}
.review-filters {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 15px;
  label {
    display: flex;
    align-items: center;
    gap: 5px;
  }
}
</style>
