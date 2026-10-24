<template>
  <ContentContainer class="search-form">
    <FloatLabel style="margin-bottom: 10px">
      <InputText v-model="searchString" size="small" @keydown.enter="updateUrl" />
      <label>{{ t('general.search') }}</label>
    </FloatLabel>
    <Checkbox v-model="titleOnly" :binary="true" inputId="titleOnly" style="margin-right: 5px" />
    <label for="titleOnly">{{ t('wiki.title_only') }}</label>
    <div class="wrapper-center">
      <Button :label="t('general.search')" size="small" :loading="loading" @click="updateUrl" />
    </div>
  </ContentContainer>
  <PaginatedResults
    v-if="searchResults.length > 0"
    :totalItems="totalResults"
    :pageSize="pageSize"
    :initialPage="page"
    :totalPages="totalPages"
    @changePage="onPageChange"
  >
    <DataTable :value="searchResults" size="small">
      <Column field="title" :header="t('general.title')">
        <template #body="slotProps">
          <RouterLink :to="`/wiki/article/${slotProps.data.id}`">{{ slotProps.data.title }}</RouterLink>
        </template>
      </Column>
      <Column field="created_at" :header="t('general.created_at')">
        <template #body="slotProps"><TimeAgo :date="slotProps.data.created_at" /></template>
      </Column>
      <Column field="updated_at" :header="t('general.updated_at')">
        <template #body="slotProps"><TimeAgo :date="slotProps.data.updated_at" /></template>
      </Column>
    </DataTable>
  </PaginatedResults>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute } from 'vue-router'
import { Button, FloatLabel, InputText, DataTable, Column, Checkbox } from 'primevue'
import ContentContainer from '@/components/ContentContainer.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { searchWikiArticles, type WikiSearchResult } from '@/services/api-schema'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

const searchResults = ref<WikiSearchResult[]>([])
const totalResults = ref(0)
const loading = ref(false)
const searchString = ref('')
const titleOnly = ref(true)
const page = ref(1)
const pageSize = 25
const totalPages = computed(() => Math.ceil(totalResults.value / pageSize))

const onPageChange = (pagination: { page: number }) => {
  page.value = pagination.page
  updateUrl()
}

const updateUrl = () => {
  router.push({
    query: {
      search_string: searchString.value || undefined,
      title_only: titleOnly.value ? 'true' : 'false',
      page: page.value.toString(),
    },
  })
}

const fetchResults = () => {
  searchString.value = route.query.search_string?.toString() ?? ''
  titleOnly.value = route.query.title_only !== 'false'
  page.value = route.query.page ? parseInt(route.query.page.toString()) : 1

  loading.value = true
  searchWikiArticles({
    search_string: searchString.value,
    title_only: titleOnly.value,
    page: page.value,
    page_size: pageSize,
  })
    .then((response) => {
      searchResults.value = response.results
      totalResults.value = response.total_items
    })
    .finally(() => {
      loading.value = false
    })
}

onMounted(() => fetchResults())

watch(
  () => route.query,
  () => fetchResults(),
  { deep: true },
)
</script>

<style scoped>
.search-form {
  margin-bottom: 15px;
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
  align-items: center;
}
</style>
