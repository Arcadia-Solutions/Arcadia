<template>
  <LazyAutoComplete
    v-model="name"
    :suggestions="foundSeries"
    @complete="search"
    size="small"
    :placeholder
    optionLabel="name"
    @option-select="seriesSelected"
    @input="onInput"
  >
    <template #option="slotProps">
      <RouterLink v-if="clickableSeriesLink" :to="`/series/${slotProps.option.id}`" style="width: 100%">
        {{ slotProps.option.name }}
      </RouterLink>
      <div v-else>{{ slotProps.option.name }}</div>
    </template>
  </LazyAutoComplete>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue'
import type { AutoCompleteOptionSelectEvent } from 'primevue'
import LazyAutoComplete from '@/components/LazyAutoComplete.vue'
import { searchSeriesLite, type SeriesLite } from '@/services/api-schema'
import type { RouterLink } from 'vue-router'

const props = defineProps<{
  placeholder: string
  clearInputOnSelect: boolean
  modelValue: string
  clickableSeriesLink: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [string]
  seriesSelected: [SeriesLite]
}>()

const name = ref('')

watch(
  () => props.modelValue,
  (newValue) => {
    name.value = newValue
  },
  { immediate: true },
)

const foundSeries = ref<SeriesLite[]>()

const seriesSelected = (event: AutoCompleteOptionSelectEvent) => {
  if (props.clearInputOnSelect) {
    name.value = ''
  }
  const selectedSeriesName = (event.value as SeriesLite).name
  emit('seriesSelected', event.value)
  emit('update:modelValue', selectedSeriesName)
}

const onInput = () => {
  emit('update:modelValue', name.value)
}

const search = () => {
  if (name.value !== '') {
    searchSeriesLite(name.value).then((series) => {
      foundSeries.value = series
    })
  } else {
    foundSeries.value = []
  }
}
</script>
