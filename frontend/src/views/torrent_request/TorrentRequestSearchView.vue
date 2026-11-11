<template>
  <div v-if="initialForm">
    <div class="actions">
      <RouterLink v-if="userStore.permissions.includes('create_torrent_request')" to="/new-torrent-request">
        <i class="pi pi-plus" v-tooltip.top="t('torrent_request.new_request')" />
      </RouterLink>
      <i class="pi pi-user" v-tooltip.top="t('torrent_request.my_requests')" />
      <i class="pi pi-heart" v-tooltip.top="t('torrent_request.voted_requests')" />
      <i class="pi pi-bookmark" v-tooltip.top="t('torrent_request.bookmarked_requests')" />
    </div>
    <TorrentRequestSearchInputs ref="searchInputsRef" class="torrent-request-search-inputs" :loading :initialForm />
    <PaginatedResults
      v-if="initialForm"
      :totalPages
      :initialPage="initialForm.page ?? 1"
      :totalItems="totalResults"
      :pageSize
      @changePage="searchInputsRef.changePage($event.page)"
    >
      <TorrentRequestsTable
        :torrentRequests="searchResults"
        displayTitleGroup
        :displayFilled="initialForm.include_filled"
        sortable
        :sortField="initialForm.order_by"
        :sortOrder
        @sort="onSort"
      />
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import TorrentRequestsTable from '@/components/torrent_request/TorrentRequestsTable.vue'
import TorrentRequestSearchInputs from '@/components/torrent_request/TorrentRequestSearchInputs.vue'
import { useRoute } from 'vue-router'
import type { VNodeRef } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  searchTorrentRequests,
  TorrentRequestSearchOrderBy,
  OrderByDirection,
  type SearchTorrentRequestsRequest,
  type TorrentRequestWithTitleGroupLite,
} from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import type { DataTableSortEvent } from 'primevue/datatable'

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()

const searchInputsRef = ref<VNodeRef | null>(null)

const searchResults = ref<TorrentRequestWithTitleGroupLite[]>([])
const loading = ref(false)
const initialForm = ref<SearchTorrentRequestsRequest | null>(null)
const totalResults = ref(0)
const pageSize = ref(0)
const totalPages = computed(() => Math.ceil(totalResults.value / pageSize.value))

const sortOrder = computed(() => (initialForm.value?.order_by_direction === OrderByDirection.Asc ? 1 : -1))

const onSort = (event: DataTableSortEvent) => {
  const direction = event.sortOrder === 1 ? OrderByDirection.Asc : OrderByDirection.Desc
  searchInputsRef.value?.setSort(event.sortField as TorrentRequestSearchOrderBy, direction)
}

const search = (form: SearchTorrentRequestsRequest) => {
  loading.value = true
  searchTorrentRequests(form)
    .then((results) => {
      pageSize.value = form.page_size ?? 25
      totalResults.value = results.total_items
      searchResults.value = results.results
    })
    .finally(() => {
      loading.value = false
    })
}

const loadSearchForm = async () => {
  loading.value = true
  initialForm.value = null
  await nextTick()

  const form: SearchTorrentRequestsRequest = {
    title_group_name: route.query.title_group_name?.toString() ?? null,
    tags: route.query.tags ? (Array.isArray(route.query.tags) ? (route.query.tags as string[]) : [route.query.tags.toString()]) : null,
    page: route.query.page ? parseInt(route.query.page as string) : null,
    page_size: route.query.page_size ? parseInt(route.query.page_size as string) : 25,
    order_by: (route.query.order_by as TorrentRequestSearchOrderBy) ?? TorrentRequestSearchOrderBy.CreatedAt,
    order_by_direction: (route.query.order_by_direction as OrderByDirection) ?? OrderByDirection.Desc,
    include_filled: route.query.include_filled === 'true',
  }
  initialForm.value = form
  pageSize.value = initialForm.value.page_size ?? 25
  search(initialForm.value)
}

onMounted(async () => {
  loadSearchForm()
})

watch(
  () => route.query,
  () => {
    loadSearchForm()
  },
  { deep: true },
)
</script>

<style scoped>
.torrent-request-search-inputs {
  margin-bottom: 25px;
}
.actions {
  text-align: center;
  i {
    margin: 15px 10px;
    color: white;
  }
}
</style>
