<template>
  <PaginatedResults :totalItems :pageSize :initialPage="page" :totalPages @changePage="onPageChange">
    <DataTable :value="activities" :loading size="small" scrollable lazy :sortField :sortOrder @sort="onSort" class="torrent-activity-table">
      <template #empty>{{ t('shop.no_activities') }}</template>
      <Column field="username" :header="t('user.user')" sortable>
        <template #body="slotProps">
          <UsernameEnriched :user="slotProps.data.user" />
        </template>
      </Column>
      <Column field="seeder" :header="t('torrent.status')" sortable>
        <template #body="slotProps">
          {{ slotProps.data.seeder ? t('torrent.seeding') : '-' }}
        </template>
      </Column>
      <Column field="uploaded" :header="t('general.uploaded')" sortable>
        <template #body="slotProps">
          {{ bytesToReadable(slotProps.data.uploaded) }}
        </template>
      </Column>
      <Column field="real_uploaded" :header="t('user.real_uploaded')" sortable>
        <template #body="slotProps">
          {{ bytesToReadable(slotProps.data.real_uploaded) }}
        </template>
      </Column>
      <Column field="downloaded" :header="t('general.downloaded')" sortable>
        <template #body="slotProps">
          {{ bytesToReadable(slotProps.data.downloaded) }}
        </template>
      </Column>
      <Column field="real_downloaded" :header="t('user.real_downloaded')" sortable>
        <template #body="slotProps">
          {{ bytesToReadable(slotProps.data.real_downloaded) }}
        </template>
      </Column>
      <Column field="total_seed_time" :header="t('shop.seed_time')" sortable>
        <template #body="slotProps">
          {{ secondsToReadable(slotProps.data.total_seed_time) }}
        </template>
      </Column>
      <Column field="bonus_points" :header="publicArcadiaSettings.bonus_points_alias" sortable>
        <template #body="slotProps">
          {{ formatBp(slotProps.data.bonus_points, publicArcadiaSettings.bonus_points_decimal_places, true) }}
        </template>
      </Column>
      <Column v-for="dateField in dateFields" :key="dateField.field" :field="dateField.field" :header="dateField.header" sortable>
        <template #body="slotProps">
          {{ slotProps.data[dateField.field] ? timeAgo(slotProps.data[dateField.field]) : '-' }}
        </template>
      </Column>
    </DataTable>
  </PaginatedResults>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import type { DataTableSortEvent } from 'primevue/datatable'
import { useI18n } from 'vue-i18n'
import PaginatedResults from '@/components/PaginatedResults.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import { getTorrentActivities, OrderByDirection, TorrentActivityOfTorrentOrderByColumn, type TorrentActivityAndUser } from '@/services/api-schema'
import { bytesToReadable, formatBp, secondsToReadable, timeAgo } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const props = defineProps<{
  torrentId: number
}>()

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const dateFields: { field: TorrentActivityOfTorrentOrderByColumn & keyof TorrentActivityAndUser; header: string }[] = [
  { field: 'grabbed_at', header: t('shop.grabbed_at') },
  { field: 'completed_at', header: t('torrent.snatched_at') },
  { field: 'first_seen_seeding_at', header: t('torrent.first_seen_seeding') },
  { field: 'last_seen_seeding_at', header: t('torrent.last_seen_seeding') },
]

const activities = ref<TorrentActivityAndUser[]>([])
const loading = ref(true)
const sortField = ref<TorrentActivityOfTorrentOrderByColumn>(TorrentActivityOfTorrentOrderByColumn.GrabbedAt)
const sortOrder = ref<1 | -1>(-1)
const page = ref(1)
const pageSize = 100
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))

const fetchActivities = () => {
  loading.value = true
  getTorrentActivities({
    torrent_id: props.torrentId,
    page: page.value,
    page_size: pageSize,
    order_by_column: sortField.value,
    order_by_direction: sortOrder.value === 1 ? OrderByDirection.Asc : OrderByDirection.Desc,
  })
    .then((data) => {
      activities.value = data.results
      totalItems.value = data.total_items
    })
    .finally(() => {
      loading.value = false
    })
}

const onSort = (event: DataTableSortEvent) => {
  sortField.value = event.sortField as TorrentActivityOfTorrentOrderByColumn
  sortOrder.value = (event.sortOrder as 1 | -1) ?? -1
  page.value = 1
  fetchActivities()
}

const onPageChange = (pagination: { page: number }) => {
  page.value = pagination.page
  fetchActivities()
}

onMounted(fetchActivities)
</script>

<style scoped>
.torrent-activity-table :deep(.p-datatable-column-header-content) {
  margin-right: 0.8rem;
}
</style>
