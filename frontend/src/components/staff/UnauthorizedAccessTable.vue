<template>
  <div class="unauthorized-access-table">
    <UnauthorizedAccessSearchForm v-if="initialForm" ref="searchFormRef" :loading :initialForm />
    <PaginatedResults
      v-if="initialForm && records"
      :totalPages
      :initialPage="initialForm.page"
      :totalItems="totalItems"
      :pageSize
      @changePage="searchFormRef.changePage($event.page)"
    >
      <DataTable :value="records" scrollable scrollHeight="70vh" size="small">
        <Column field="user.username" :header="t('user.user')">
          <template #body="slotProps">
            <UsernameEnriched :user="slotProps.data.user" />
          </template>
        </Column>
        <Column field="missing_permission" :header="t('unauthorized_access.missing_permission')">
          <template #body="slotProps">
            {{ slotProps.data.missing_permission }}
          </template>
        </Column>
        <Column field="path" header="path">
          <template #body="slotProps">
            {{ slotProps.data.path }}
          </template>
        </Column>
        <Column field="created_at" :header="t('general.created_at')">
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.created_at" />
          </template>
        </Column>
      </DataTable>
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { Column, DataTable } from 'primevue'
import {
  searchUnauthorizedAccessLogs,
  type PaginatedResultsUnauthorizedAccessResultsInner,
  type SearchUnauthorizedAccessLogsRequest,
  UnauthorizedAccessSortByColumn,
  type UserPermission,
} from '@/services/api-schema'
import PaginatedResults from '@/components/PaginatedResults.vue'
import type { VNodeRef } from 'vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import UnauthorizedAccessSearchForm from './UnauthorizedAccessSearchForm.vue'

const { t } = useI18n()
const route = useRoute()

const searchFormRef = ref<VNodeRef | null>(null)
const records = ref<PaginatedResultsUnauthorizedAccessResultsInner[]>()
const loading = ref(false)
const initialForm = ref<SearchUnauthorizedAccessLogsRequest | null>(null)
const totalItems = ref(0)
const pageSize = ref(20)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize.value))

const search = async (form: SearchUnauthorizedAccessLogsRequest) => {
  loading.value = true
  const response = await searchUnauthorizedAccessLogs(form).finally(() => {
    loading.value = false
  })
  pageSize.value = form.page_size
  totalItems.value = response.total_items
  records.value = response.results
}

const loadFormFromUrl = async () => {
  loading.value = true
  initialForm.value = null
  await nextTick()

  const now = new Date()
  now.setHours(23, 59, 59, 999)
  const monthAgo = new Date(now.getTime() - 28 * 24 * 60 * 60 * 1000)
  monthAgo.setHours(0, 0, 0, 0)

  const form: SearchUnauthorizedAccessLogsRequest = {
    page: route.query.page ? parseInt(route.query.page as string) : 1,
    page_size: route.query.page_size ? parseInt(route.query.page_size as string) : 20,
    user_id: route.query.user_id ? parseInt(route.query.user_id as string) : undefined,
    permission: route.query.permission ? (route.query.permission as UserPermission) : undefined,
    from_date: route.query.from_date?.toString() ?? monthAgo.toISOString(),
    to_date: route.query.to_date?.toString() ?? now.toISOString(),
    sort_by_column:
      (route.query.sort_by_column as (typeof UnauthorizedAccessSortByColumn)[keyof typeof UnauthorizedAccessSortByColumn]) ??
      UnauthorizedAccessSortByColumn.CreatedAt,
    sort_by_direction: (route.query.sort_by_direction as 'asc' | 'desc') ?? 'desc',
  }

  initialForm.value = form
  pageSize.value = form.page_size
  search(form)
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
.unauthorized-access-table {
  margin-top: 20px;
}
</style>
