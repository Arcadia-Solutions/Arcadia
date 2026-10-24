<template>
  <div class="torrent-activities">
    <div class="calculator">
      <div class="calculator-form">
        <FloatLabel>
          <InputNumber v-model="hoursSeedingPerDay" inputId="hours-seeding" size="small" :min="1" :max="24" showButtons />
          <label for="hours-seeding">{{ t('shop.hours_seeding_per_day') }}</label>
        </FloatLabel>
        <FloatLabel style="width: 10em">
          <Select
            v-model="seedersPerTorrent"
            style="width: 10em"
            inputId="seeders-per-torrent"
            :options="seedersOptions"
            optionLabel="label"
            optionValue="value"
            size="small"
          />
          <label for="seeders-per-torrent">{{ t('shop.seeders_per_torrent') }}</label>
        </FloatLabel>
        <Button :label="t('shop.calculate')" size="small" @click="calculate" :loading="overviewLoading" />
      </div>
    </div>

    <div v-if="overview" class="overview">
      <div class="overview-grid">
        <ContentContainer class="overview-item">
          <div class="label">{{ bpAlias }}/{{ t('shop.hour') }}</div>
          <div class="value">{{ formatBpOverview(overview.bonus_points_per_day / 24) }}</div>
        </ContentContainer>
        <ContentContainer class="overview-item">
          <div class="label">{{ bpAlias }}/{{ t('shop.day') }}</div>
          <div class="value">{{ formatBpOverview(overview.bonus_points_per_day) }}</div>
        </ContentContainer>
        <ContentContainer class="overview-item">
          <div class="label">{{ bpAlias }}/{{ t('shop.week') }}</div>
          <div class="value">{{ formatBpOverview(overview.bonus_points_per_day * 7) }}</div>
        </ContentContainer>
        <ContentContainer class="overview-item">
          <div class="label">{{ bpAlias }}/{{ t('shop.month') }}</div>
          <div class="value">{{ formatBpOverview(overview.bonus_points_per_day * 28) }}</div>
        </ContentContainer>
        <ContentContainer class="overview-item">
          <div class="label">{{ bpAlias }}/{{ t('shop.year') }}</div>
          <div class="value">{{ formatBpOverview(overview.bonus_points_per_day * 365) }}</div>
        </ContentContainer>
      </div>
    </div>

    <PaginatedResults :totalItems :pageSize :initialPage="page" :totalPages @changePage="onPageChange">
      <DataTable :value="activities" :loading="loading" size="small" lazy :sortField :sortOrder @sort="onSort">
        <template #empty>
          <div class="empty-message">{{ t('shop.no_activities') }}</div>
        </template>
        <Column :header="t('shop.torrent')">
          <template #body="slotProps">
            <TitleGroupSlimHeader
              :titleGroup="slotProps.data.title_group"
              :series="slotProps.data.title_group.series"
              :affiliatedArtists="slotProps.data.title_group.affiliated_artists"
              nameLink
            />
            <span class="light-slug">
              <TorrentSlug
                :torrent="slotProps.data.title_group.edition_groups[0].torrents[0]"
                :editionGroup="slotProps.data.title_group.edition_groups[0]"
                :contentType="slotProps.data.title_group.content_type"
                sortedBy=""
                hidePeerStatus
              />
            </span>
          </template>
        </Column>
        <Column field="total_seed_time" :header="t('shop.seed_time')" sortable>
          <template #body="slotProps">
            {{ formatSeedTime(slotProps.data.torrent_activity.total_seed_time) }}
          </template>
        </Column>
        <Column field="uploaded" :header="t('general.uploaded')" sortable>
          <template #body="slotProps">
            {{ bytesToReadable(slotProps.data.torrent_activity.uploaded) }}
          </template>
        </Column>
        <Column field="downloaded" :header="t('general.downloaded')" sortable>
          <template #body="slotProps">
            {{ bytesToReadable(slotProps.data.torrent_activity.downloaded) }}
          </template>
        </Column>
        <Column field="torrent_size" :header="t('torrent.size')" sortable>
          <template #body="slotProps">
            {{ bytesToReadable(slotProps.data.title_group.edition_groups[0].torrents[0].size) }}
          </template>
        </Column>
        <Column field="torrent_seeders" :header="t('torrent.seeders')" sortable>
          <template #body="slotProps">
            {{ formatNumber(slotProps.data.title_group.edition_groups[0].torrents[0].seeders) }}
          </template>
        </Column>
        <Column field="bonus_points" :header="bpAlias" sortable>
          <template #body="slotProps">
            {{ formatBp(slotProps.data.torrent_activity.bonus_points) }}
          </template>
        </Column>
        <Column :header="`${bpAlias}/h`">
          <template #body="slotProps">
            {{ formatBp(slotProps.data.torrent_activity.bonus_points_per_day / 24) }}
          </template>
        </Column>
        <Column field="bonus_points_per_day" :header="`${bpAlias}/d`" sortable>
          <template #body="slotProps">
            {{ formatBp(slotProps.data.torrent_activity.bonus_points_per_day) }}
          </template>
        </Column>
        <Column :header="`${bpAlias}/w`">
          <template #body="slotProps">
            {{ formatBp(slotProps.data.torrent_activity.bonus_points_per_day * 7) }}
          </template>
        </Column>
        <Column :header="`${bpAlias}/m`">
          <template #body="slotProps">
            {{ formatBp(slotProps.data.torrent_activity.bonus_points_per_day * 28) }}
          </template>
        </Column>
        <Column :header="`${bpAlias}/y`">
          <template #body="slotProps">
            {{ formatBp(slotProps.data.torrent_activity.bonus_points_per_day * 365) }}
          </template>
        </Column>
        <Column field="grabbed_at" :header="t('shop.grabbed_at')" sortable>
          <template #body="slotProps">
            <TimeAgo v-if="slotProps.data.torrent_activity.grabbed_at" :date="slotProps.data.torrent_activity.grabbed_at" /><template v-else>-</template>
          </template>
        </Column>
      </DataTable>
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import InputNumber from 'primevue/inputnumber'
import Select from 'primevue/select'
import FloatLabel from 'primevue/floatlabel'
import { Button } from 'primevue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import TitleGroupSlimHeader from '@/components/title_group/TitleGroupSlimHeader.vue'
import TorrentSlug from '@/components/torrent/TorrentSlug.vue'
import {
  getUserTorrentActivities,
  getTorrentActivitiesOverview,
  SeedersPerTorrent,
  TorrentActivityOrderByColumn,
  OrderByDirection,
  type TorrentActivitiesOverview,
  type TorrentActivityAndTitleGroup,
} from '@/services/api-schema'
import { bytesToReadable, formatBp as formatBpShared, formatNumber } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import type { DataTableSortEvent } from 'primevue/datatable'
import ContentContainer from '../ContentContainer.vue'

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
const bpAlias = computed(() => publicArcadiaSettings.bonus_points_alias)

const activities = ref<TorrentActivityAndTitleGroup[]>([])
const loading = ref(true)
const overview = ref<TorrentActivitiesOverview | null>(null)
const overviewLoading = ref(false)

const hoursSeedingPerDay = ref(24)
const seedersPerTorrent = ref<SeedersPerTorrent>(SeedersPerTorrent.Current)

const sortField = ref<TorrentActivityOrderByColumn>(TorrentActivityOrderByColumn.GrabbedAt)
const sortOrder = ref<1 | -1>(-1)
const page = ref(1)
const pageSize = ref(50)
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize.value))

const seedersOptions = [
  { label: t('shop.current_seeders'), value: SeedersPerTorrent.Current },
  { label: '1', value: SeedersPerTorrent._1 },
  { label: '2', value: SeedersPerTorrent._2 },
  { label: '3', value: SeedersPerTorrent._3 },
  { label: '4', value: SeedersPerTorrent._4 },
  { label: '5', value: SeedersPerTorrent._5 },
  { label: '10', value: SeedersPerTorrent._10 },
  { label: '15', value: SeedersPerTorrent._15 },
  { label: '20', value: SeedersPerTorrent._20 },
  { label: '25', value: SeedersPerTorrent._25 },
  { label: '50', value: SeedersPerTorrent._50 },
  { label: '100', value: SeedersPerTorrent._100 },
  { label: '10%', value: SeedersPerTorrent._10Percent },
  { label: '25%', value: SeedersPerTorrent._25Percent },
  { label: '50%', value: SeedersPerTorrent._50Percent },
  { label: '75%', value: SeedersPerTorrent._75Percent },
  { label: '100%', value: SeedersPerTorrent._100Percent },
  { label: '150%', value: SeedersPerTorrent._150Percent },
  { label: '200%', value: SeedersPerTorrent._200Percent },
  { label: '300%', value: SeedersPerTorrent._300Percent },
  { label: '500%', value: SeedersPerTorrent._500Percent },
]

const formatBp = (value: number) => formatBpShared(value, publicArcadiaSettings.bonus_points_decimal_places, true)
// show up to 2 decimal places for simplicity
const formatBpOverview = (value: number) =>
  formatBpShared(value, publicArcadiaSettings.bonus_points_decimal_places, true, Math.min(publicArcadiaSettings.bonus_points_decimal_places, 2))

const formatSeedTime = (seconds: number) => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const fetchActivities = () => {
  loading.value = true
  getUserTorrentActivities({
    include_unseeded_torrents: false,
    page: page.value,
    page_size: pageSize.value,
    order_by_column: sortField.value,
    order_by_direction: sortOrder.value === 1 ? OrderByDirection.Asc : OrderByDirection.Desc,
    hours_seeding_per_day: hoursSeedingPerDay.value,
    seeders_per_torrent: seedersPerTorrent.value,
  })
    .then((data) => {
      activities.value = data.results
      totalItems.value = data.total_items
      pageSize.value = data.page_size
    })
    .finally(() => {
      loading.value = false
    })
}

const fetchOverview = () => {
  overviewLoading.value = true
  getTorrentActivitiesOverview({
    hours_seeding_per_day: hoursSeedingPerDay.value,
    seeders_per_torrent: seedersPerTorrent.value,
  })
    .then((data) => {
      overview.value = data
    })
    .finally(() => {
      overviewLoading.value = false
    })
}

const calculate = () => {
  fetchOverview()
  fetchActivities()
}

const onSort = (event: DataTableSortEvent) => {
  sortField.value = event.sortField as TorrentActivityOrderByColumn
  sortOrder.value = (event.sortOrder as 1 | -1) ?? -1
  page.value = 1
  fetchActivities()
}

const onPageChange = (pagination: { page: number }) => {
  page.value = pagination.page
  fetchActivities()
}

onMounted(() => {
  fetchActivities()
  fetchOverview()
})
</script>

<style scoped>
.torrent-activities {
  margin-top: 15px;
}

.calculator-form {
  display: flex;
  align-items: flex-end;
  gap: 20px;
  margin-bottom: 20px;
}

.overview {
  margin-bottom: 20px;
}

.overview-grid {
  display: flex;
  gap: 15px;
}

.overview-item {
  display: flex;
  flex-direction: column;
}

.overview-item .label {
  font-size: 0.85rem;
  color: var(--p-text-muted-color);
}

.overview-item .value {
  font-size: 1.2rem;
  font-weight: 600;
}

.light-slug :deep(span) {
  color: var(--p-text-muted-color);
}

.empty-message {
  text-align: center;
  padding: 20px;
  color: var(--text-color-secondary);
}
</style>
