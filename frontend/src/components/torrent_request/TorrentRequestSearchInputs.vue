<template>
  <ContentContainer>
    <div id="torrent-request-search-inputs">
      <div class="line">
        <FloatLabel>
          <InputText class="title-group-name" size="small" v-model="searchForm.title_group_name" name="title_group_name" />
          <label for="title_group_name">{{ t('general.search_terms') }}</label>
        </FloatLabel>
      </div>
      <!-- <div class="line">
        <FloatLabel>
          <InputText class="tags" size="small" v-model="tagsInput" name="tags" />
          <label for="tags">{{ t('general.tags_comma_separated') }}</label>
        </FloatLabel>
      </div> -->
      <div class="include-filled-checkbox" style="margin-top: 15px">
        <Checkbox v-model="searchForm.include_filled" inputId="include_filled" :binary="true" />
        <label for="include_filled" style="margin-left: 5px">{{ t('torrent_request.include_filled') }}</label>
      </div>
      <div class="flex justify-content-center" style="margin-top: 15px">
        <Button :loading :label="t('general.search')" @click="search" size="small" />
      </div>
    </div>
  </ContentContainer>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import ContentContainer from '../ContentContainer.vue'
import InputText from 'primevue/inputtext'
import FloatLabel from 'primevue/floatlabel'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import { TorrentRequestSearchOrderBy, OrderByDirection, type SearchTorrentRequestsRequest } from '@/services/api-schema'

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  loading: boolean
  initialForm: SearchTorrentRequestsRequest
}>()

const searchForm = ref<SearchTorrentRequestsRequest>({ ...props.initialForm })

const changePage = (page: number) => {
  searchForm.value.page = page
  updateUrl()
}

const setSort = (orderBy: TorrentRequestSearchOrderBy, orderByDirection: OrderByDirection) => {
  searchForm.value.order_by = orderBy
  searchForm.value.order_by_direction = orderByDirection
  updateUrl()
}

const search = () => {
  searchForm.value.page = 1
  updateUrl()
}

const updateUrl = () => {
  router.push({
    query: Object.fromEntries(
      Object.entries({
        title_group_name: searchForm.value.title_group_name,
        tags: searchForm.value.tags,
        page: searchForm.value.page,
        page_size: searchForm.value.page_size,
        order_by: searchForm.value.order_by,
        order_by_direction: searchForm.value.order_by_direction,
        include_filled: searchForm.value.include_filled ? 'true' : undefined,
      }).filter(([, v]) => v !== undefined && v !== null),
    ),
  })
}

defineExpose({
  searchForm,
  changePage,
  setSort,
})

watch(
  () => [searchForm.value.title_group_name, searchForm.value.tags, searchForm.value.include_filled],
  () => {
    searchForm.value.page = 1
  },
  { deep: true },
)
</script>

<style scoped>
.title-group-name {
  width: 40em;
}
.tags {
  width: 30em;
}
</style>
