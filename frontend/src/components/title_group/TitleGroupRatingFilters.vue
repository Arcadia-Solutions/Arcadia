<template>
  <ContentContainer class="rating-filters-container">
    <div class="rating-filters">
      <label>
        {{ t('title_group.minimum_rating') }}
        <InputNumber v-model="filters.minimum_rating" :min="0" :max="100" showButtons :allowEmpty="false" :inputStyle="{ width: '6.5em' }" size="small" />
      </label>
      <label>
        {{ t('title_group.maximum_rating') }}
        <InputNumber v-model="filters.maximum_rating" :min="0" :max="100" showButtons :allowEmpty="false" :inputStyle="{ width: '6.5em' }" size="small" />
      </label>
      <label v-if="!hideSorting">
        {{ t('general.sort_by') }}
        <Select
          v-model="filters.order_by_column"
          :options="[
            { label: t('general.created_at'), value: TitleGroupRatingSearchOrderByColumn.CreatedAt },
            { label: t('title_group.rating'), value: TitleGroupRatingSearchOrderByColumn.Rating },
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
      <InputText v-model="filters.content" :placeholder="t('title_group.search_comment')" size="small" class="content-search" @keyup.enter="emit('search')" />
      <Button :label="t('general.search')" size="small" :loading @click="emit('search')" />
    </div>
  </ContentContainer>
</template>

<script setup lang="ts">
import { Button, InputNumber, InputText, Select } from 'primevue'
import { useI18n } from 'vue-i18n'
import ContentContainer from '../ContentContainer.vue'
import { getOrderByDirectionOptions } from '@/services/helpers'
import { OrderByDirection, TitleGroupRatingSearchOrderByColumn } from '@/services/api-schema'

export type TitleGroupRatingFilters = {
  minimum_rating: number
  maximum_rating: number
  content: string
  order_by_column: TitleGroupRatingSearchOrderByColumn
  order_by_direction: OrderByDirection
}

defineProps<{ loading: boolean; hideSorting?: boolean }>()
const filters = defineModel<TitleGroupRatingFilters>({ required: true })
const emit = defineEmits<{ search: [] }>()

const { t } = useI18n()
</script>

<style scoped>
.content-search {
  flex: 1;
  min-width: 15em;
}
.comment-filters-container {
  margin-bottom: 10px;
}
.comment-filters {
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
