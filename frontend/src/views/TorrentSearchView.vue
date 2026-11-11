<template>
  <div v-if="search_results">
    <TorrentSearchInputs v-if="initialForm" ref="searchInputsRef" class="torrent-search-inputs" :loading :initialForm="initialForm" :totalResults />
    <PaginatedResults
      v-if="initialForm"
      :totalPages
      :initialPage="initialForm.page"
      :totalItems="totalResults"
      :pageSize
      @changePage="searchInputsRef.changePage($event.page)"
    >
      <TitleGroupList :titleGroups="search_results" :titleGroupPreview />
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import TorrentSearchInputs from '@/components/torrent/TorrentSearchInputs.vue'
import TitleGroupList from '@/components/title_group/TitleGroupList.vue'
import type { titleGroupPreviewMode } from '@/components/title_group/TitleGroupList.vue'
import { useRoute } from 'vue-router'
import type { VNodeRef } from 'vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { searchTorrents, type TitleGroupHierarchyLite, type TorrentSearch } from '@/services/api-schema'

const route = useRoute()

const searchInputsRef = ref<VNodeRef | null>(null)

const search_results = ref<TitleGroupHierarchyLite[]>([])
const titleGroupPreview = ref<titleGroupPreviewMode>('table') // TODO: make a select button to switch from cover-only to table
const loading = ref(false)
const initialForm = ref<TorrentSearch | null>(null)
const totalResults = ref(0)
const pageSize = ref(0)
const totalPages = computed(() => Math.ceil(totalResults.value / pageSize.value))

const search = async (torrentSearch: TorrentSearch) => {
  const results = await searchTorrents(torrentSearch).finally(() => {
    loading.value = false
  })
  // page.value = torrentSearch.page
  pageSize.value = torrentSearch.page_size
  totalResults.value = results.total_items
  search_results.value = results.results
}

const parseArrayParam = (param: string | string[] | undefined): string[] => {
  if (!param) return []
  if (Array.isArray(param)) return param
  return param.split(',')
}

const loadFormFromUrl = () => {
  loading.value = true
  const form: TorrentSearch = {
    title_group_name: route.query.title_group_name?.toString() ?? '',
    title_group_tags: route.query.title_group_tags?.toString() || null,
    page: route.query.page ? parseInt(route.query.page as string) : 1,
    page_size: route.query.page_size ? parseInt(route.query.page_size as string) : 25,
    torrent_created_by_id: route.query.torrent_created_by_id ? parseInt(route.query.torrent_created_by_id as string) : null,
    torrent_snatched_by_id: route.query.torrent_snatched_by_id ? parseInt(route.query.torrent_snatched_by_id as string) : null,
    torrent_staff_checked: route.query.torrent_staff_checked === 'true' ? true : route.query.torrent_staff_checked === 'false' ? false : null,
    torrent_reported: route.query.torrent_reported === 'true' ? true : route.query.torrent_reported === 'false' ? false : null,
    torrent_trumpable: route.query.torrent_trumpable === 'true' ? true : route.query.torrent_trumpable === 'false' ? false : null,
    user_id_bookmarks: route.query.user_id_bookmarks ? parseInt(route.query.user_id_bookmarks as string) : null,
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    order_by_column: route.query.order_by_column ? (route.query.order_by_column as string) : 'torrent_created_at',
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    order_by_direction: route.query.order_by_direction ? (route.query.order_by_direction as string) : 'desc',
    title_group_include_empty_groups: route.query.title_group_include_empty_groups === 'true' ? true : false,
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    title_group_content_type: parseArrayParam(route.query.title_group_content_type as string | string[]),
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    title_group_category: parseArrayParam(route.query.title_group_category as string | string[]),
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    edition_group_source: parseArrayParam(route.query.edition_group_source as string | string[]),
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    torrent_language: parseArrayParam(route.query.torrent_language as string | string[]),
    // @ts-expect-error what is placed in this query always comes from the form, so there shouldn't be a wrong value
    torrent_video_resolution: parseArrayParam(route.query.torrent_video_resolution as string | string[]),
  }
  initialForm.value = form
  pageSize.value = initialForm.value.page_size
  search(initialForm.value)
}

onMounted(() => {
  loadFormFromUrl()
})

watch(
  () => route.query,
  () => {
    loadFormFromUrl()
  },
  { deep: true },
)
</script>

<style scoped>
.torrent-search-inputs {
  margin-bottom: 25px;
}
</style>
