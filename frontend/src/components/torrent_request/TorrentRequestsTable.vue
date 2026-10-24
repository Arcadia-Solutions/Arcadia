<template>
  <DataTable :value="torrentRequests" size="small" :sortField :sortOrder lazy @sort="onSort">
    <Column :header="t('torrent_request.request')">
      <template #body="slotProps">
        <template v-if="displayTitleGroup">
          <RouterLink class="request-link" :to="`/torrent-request/${getTorrentRequest(slotProps.data).torrent_request.id}`">
            <TitleGroupSlimHeader
              :titleGroup="slotProps.data.title_group"
              :series="slotProps.data.series"
              :affiliatedArtists="slotProps.data.affiliated_artists"
            />
          </RouterLink>
          <TorrentRequestSlug
            class="light-slug"
            :torrentRequest="getTorrentRequest(slotProps.data).torrent_request"
            :contentType="contentType ?? slotProps.data.title_group.content_type"
          />
        </template>
        <template v-else>
          <RouterLink class="request-link" :to="`/torrent-request/${getTorrentRequest(slotProps.data).torrent_request.id}`">
            <TorrentRequestSlug
              :torrentRequest="getTorrentRequest(slotProps.data).torrent_request"
              :contentType="contentType ?? slotProps.data.title_group.content_type"
            />
          </RouterLink>
        </template>
      </template>
    </Column>
    <Column
      v-if="publicArcadiaSettings.torrent_request_vote_currencies.includes(TorrentRequestVoteCurrency.Upload)"
      field="upload"
      :header="t('user.upload')"
      :sortable="sortable"
    >
      <template #body="slotProps">{{ bytesToReadable(getTorrentRequest(slotProps.data).bounty.upload) }}</template>
    </Column>
    <Column
      v-if="publicArcadiaSettings.torrent_request_vote_currencies.includes(TorrentRequestVoteCurrency.BonusPoints)"
      field="bonus_points"
      :header="publicArcadiaSettings.bonus_points_alias"
      :sortable="sortable"
    >
      <template #body="slotProps">{{
        formatBp(getTorrentRequest(slotProps.data).bounty.bonus_points, publicArcadiaSettings.bonus_points_decimal_places)
      }}</template>
    </Column>
    <Column field="voters" :header="t('torrent_request.voter', 2)" :sortable="sortable">
      <template #body="slotProps">{{ formatNumber(getTorrentRequest(slotProps.data).user_votes_amount) }}</template>
    </Column>
    <Column field="created_at" :header="t('general.created_at')" :sortable="sortable">
      <template #body="slotProps"><TimeAgo :date="getTorrentRequest(slotProps.data).torrent_request.created_at" /></template>
    </Column>
    <Column :header="t('torrent_request.requested_by')">
      <template #body="slotProps">
        <UsernameEnriched :user="getTorrentRequest(slotProps.data).created_by" />
      </template>
    </Column>
    <Column :header="t('torrent_request.filled_by')">
      <template #body="slotProps">
        <UsernameEnriched v-if="getTorrentRequest(slotProps.data).filled_by" :user="getTorrentRequest(slotProps.data).filled_by" />
      </template>
    </Column>
  </DataTable>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Column, DataTable } from 'primevue'
import TorrentRequestSlug from './TorrentRequestSlug.vue'
import TitleGroupSlimHeader from '../title_group/TitleGroupSlimHeader.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { RouterLink } from 'vue-router'
import { bytesToReadable, formatBp, formatNumber } from '@/services/helpers'
import { useI18n } from 'vue-i18n'
import type { ContentType, TorrentRequestHierarchyLite, TorrentRequestWithTitleGroupLite, TorrentRequestSearchOrderBy } from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { TorrentRequestVoteCurrency } from '@/services/api-schema'
import type { DataTableSortEvent } from 'primevue/datatable'

defineProps<{
  torrentRequests: TorrentRequestHierarchyLite[] | TorrentRequestWithTitleGroupLite[]
  contentType?: ContentType
  displayTitleGroup?: boolean
  sortable?: boolean
  sortField?: TorrentRequestSearchOrderBy
  sortOrder?: number
}>()

// this component accepts 2 types. one of which contains title group information, and the other does not.
const getTorrentRequest = (row: TorrentRequestHierarchyLite | TorrentRequestWithTitleGroupLite): TorrentRequestHierarchyLite => {
  if ('title_group' in row) {
    return row.torrent_request
  }
  return row
}

const emit = defineEmits<{
  sort: [event: DataTableSortEvent]
}>()

const onSort = (event: DataTableSortEvent) => {
  emit('sort', event)
}

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
</script>
<style scoped>
.request-link :deep(a) {
  pointer-events: none;
}

.light-slug :deep(span) {
  font-weight: 300;
}
</style>
