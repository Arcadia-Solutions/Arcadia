<template>
  <ContentContainer class="rating-filters-container">
    <div class="rating-filters">
      <FloatLabel>
        <InputNumber
          v-model="filters.minimum_rating"
          inputId="minimum_rating"
          :min="0"
          :max="100"
          showButtons
          :allowEmpty="false"
          :inputStyle="{ width: '9em' }"
          size="small"
        />
        <label for="minimum_rating">{{ t('title_group.minimum_rating') }}</label>
      </FloatLabel>
      <FloatLabel>
        <InputNumber
          v-model="filters.maximum_rating"
          inputId="maximum_rating"
          :min="0"
          :max="100"
          showButtons
          :allowEmpty="false"
          :inputStyle="{ width: '9em' }"
          size="small"
        />
        <label for="maximum_rating">{{ t('title_group.maximum_rating') }}</label>
      </FloatLabel>
      <FloatLabel v-if="!hideSorting">
        <Select
          v-model="filters.order_by_column"
          inputId="order_by_column"
          :options="[
            { label: t('general.created_at'), value: TitleGroupRatingSearchOrderByColumn.CreatedAt },
            { label: t('title_group.rating'), value: TitleGroupRatingSearchOrderByColumn.Rating },
          ]"
          optionLabel="label"
          optionValue="value"
          size="small"
        />
        <label for="order_by_column">{{ t('general.sort_by') }}</label>
      </FloatLabel>
      <FloatLabel v-if="!hideSorting">
        <Select
          v-model="filters.order_by_direction"
          inputId="order_by_direction"
          :options="getOrderByDirectionOptions(t)"
          optionLabel="label"
          optionValue="value"
          size="small"
        />
        <label for="order_by_direction">{{ t('general.order_by') }}</label>
      </FloatLabel>
      <FloatLabel class="content-search">
        <InputText v-model="filters.content" id="content_search" size="small" @keyup.enter="emit('search')" />
        <label for="content_search">{{ t('title_group.search_comment') }}</label>
      </FloatLabel>
      <Button :label="t('general.search')" size="small" :loading @click="emit('search')" />
    </div>
  </ContentContainer>
</template>

<script setup lang="ts">
import { Button, FloatLabel, InputNumber, InputText, Select } from 'primevue'
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
.rating-filters-container {
  margin-bottom: 10px;
}
.rating-filters {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 15px;
  padding-top: 10px;
}
.content-search {
  flex: 1;
  min-width: 15em;
  input {
    width: 100%;
  }
}
</style>
