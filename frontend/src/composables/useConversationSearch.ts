import { ref, computed, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  searchConversations,
  ConversationSearchOrderByColumn,
  OrderByDirection,
  type PaginatedResultsConversationSearchResultResultsInner,
  type UserLite,
} from '@/services/api-schema'
import type { DataTableSortEvent } from 'primevue/datatable'

const orderByColumns: ConversationSearchOrderByColumn[] = Object.values(ConversationSearchOrderByColumn)
const orderByDirections: OrderByDirection[] = Object.values(OrderByDirection)

const isOrderByColumn = (value: unknown): value is ConversationSearchOrderByColumn => typeof value === 'string' && (orderByColumns as string[]).includes(value)

const isOrderByDirection = (value: unknown): value is OrderByDirection => typeof value === 'string' && (orderByDirections as string[]).includes(value)

export function useConversationSearch(options: { tab?: string; allConversations?: boolean } = {}) {
  const allConversations = options.allConversations === true
  const router = useRouter()
  const route = useRoute()

  const searchForm = ref<{
    search_term: string
    search_titles_only: boolean
    user_id: number | undefined
    order_by_column: ConversationSearchOrderByColumn
    order_by_direction: OrderByDirection
    page: number
    page_size: number
  }>({
    search_term: '',
    search_titles_only: true,
    user_id: undefined,
    order_by_column: ConversationSearchOrderByColumn.LastMessage,
    order_by_direction: OrderByDirection.Desc,
    page: 1,
    page_size: 50,
  })
  const filterUsername = ref('')
  const searchResults = ref<PaginatedResultsConversationSearchResultResultsInner[]>([])
  const totalResults = ref(0)
  const totalPages = computed(() => Math.ceil(totalResults.value / searchForm.value.page_size))

  const updateUrl = () => {
    if (filterUsername.value === '') {
      searchForm.value.user_id = undefined
    }
    router.push({
      query: {
        ...(searchForm.value.search_term ? { search_term: searchForm.value.search_term } : {}),
        ...(!searchForm.value.search_titles_only ? { search_titles_only: 'false' } : {}),
        ...(searchForm.value.user_id ? { user_id: searchForm.value.user_id.toString() } : {}),
        ...(filterUsername.value ? { user_filter_username: filterUsername.value } : {}),
        order_by_column: searchForm.value.order_by_column,
        order_by_direction: searchForm.value.order_by_direction,
        page: searchForm.value.page.toString(),
        page_size: searchForm.value.page_size.toString(),
        ...(options.tab ? { tab: options.tab } : {}),
      },
    })
  }

  const onChangePage = (pagination: { page: number; pageSize: number }) => {
    searchForm.value.page = pagination.page
    updateUrl()
  }

  const onSort = (event: DataTableSortEvent) => {
    searchForm.value.order_by_column = isOrderByColumn(event.sortField) ? event.sortField : ConversationSearchOrderByColumn.LastMessage
    searchForm.value.order_by_direction = event.sortOrder === 1 ? OrderByDirection.Asc : OrderByDirection.Desc
    searchForm.value.page = 1
    updateUrl()
  }

  const onUserSelected = (user: UserLite) => {
    searchForm.value.user_id = user.id
    filterUsername.value = user.username
  }

  const fetchConversations = () => {
    searchForm.value.page = route.query.page ? parseInt(route.query.page as string) : 1
    searchForm.value.page_size = route.query.page_size ? parseInt(route.query.page_size as string) : 50
    searchForm.value.search_term = (route.query.search_term as string) || ''
    searchForm.value.search_titles_only = route.query.search_titles_only !== 'false'
    searchForm.value.user_id = route.query.user_id ? parseInt(route.query.user_id as string) : undefined
    filterUsername.value = (route.query.user_filter_username as string) || ''
    searchForm.value.order_by_column = isOrderByColumn(route.query.order_by_column) ? route.query.order_by_column : ConversationSearchOrderByColumn.LastMessage
    searchForm.value.order_by_direction = isOrderByDirection(route.query.order_by_direction) ? route.query.order_by_direction : OrderByDirection.Desc

    searchConversations({
      search_term: searchForm.value.search_term || undefined,
      search_titles_only: searchForm.value.search_titles_only,
      user_id: searchForm.value.user_id,
      order_by_column: searchForm.value.order_by_column,
      order_by_direction: searchForm.value.order_by_direction,
      page: searchForm.value.page,
      page_size: searchForm.value.page_size,
      all_conversations: allConversations,
    }).then((response) => {
      searchResults.value = response.results
      totalResults.value = response.total_items
    })
  }

  onMounted(fetchConversations)

  watch(
    () => route.query,
    () => {
      if (!options.tab || route.query.tab === options.tab) {
        fetchConversations()
      }
    },
    { deep: true },
  )

  return {
    searchForm,
    filterUsername,
    searchResults,
    totalResults,
    totalPages,
    updateUrl,
    onChangePage,
    onSort,
    onUserSelected,
    fetchConversations,
  }
}
