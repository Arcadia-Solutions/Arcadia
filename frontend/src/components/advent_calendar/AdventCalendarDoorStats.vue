<template>
  <Dialog closeOnEscape modal :header="t('advent_calendar.door_stats')" :visible="doorId !== null" @update:visible="doorId = null">
    <div class="door-stats">
      <div class="opening-count">{{ t('advent_calendar.opening_count', openingCount) }}</div>
      <div v-if="rewardTotals" class="reward-totals">
        {{ t('advent_calendar.rewards_granted') }}: {{ formatRewards(rewardTotals).join(', ') || t('advent_calendar.no_rewards') }}
      </div>
      <InputText v-model="username" size="small" :placeholder="t('user.username')" @keyup.enter="fetchStats(1)" />
      <PaginatedResults :totalItems :pageSize :initialPage="page" :totalPages @changePage="fetchStats($event.page)">
        <DataTable :value="openers" :loading size="small" lazy :sortField :sortOrder @sort="onSort">
          <template #empty>{{ t('advent_calendar.no_openers') }}</template>
          <Column style="width: 50px">
            <template #body="slotProps">
              <img :src="slotProps.data.user.avatar ?? '/default_user_avatar.png'" :alt="slotProps.data.user.username + '\'s avatar'" class="avatar" />
            </template>
          </Column>
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.Username" :header="t('user.username')" sortable>
            <template #body="slotProps">
              <UsernameEnriched :user="slotProps.data.user" />
            </template>
          </Column>
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.OpenedAt" :header="t('advent_calendar.opened_at')" sortable>
            <template #body="slotProps">
              <TimeAgo :date="slotProps.data.opened_at" />
            </template>
          </Column>
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.Upload" :header="t('advent_calendar.reward_upload')" sortable>
            <template #body="slotProps">{{ bytesToReadable(slotProps.data.upload) }}</template>
          </Column>
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.DownloadRemoved" :header="t('advent_calendar.reward_download_removed')" sortable>
            <template #body="slotProps">{{ bytesToReadable(slotProps.data.download_removed) }}</template>
          </Column>
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.BonusPoints" :header="publicArcadiaSettings.bonus_points_alias" sortable>
            <template #body="slotProps">{{ formatBp(slotProps.data.bonus_points, publicArcadiaSettings.bonus_points_decimal_places) }}</template>
          </Column>
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.FreeleechTokens" :header="t('advent_calendar.reward_freeleech_tokens')" sortable />
          <Column :field="AdventCalendarDoorOpenerOrderByColumn.Invitations" :header="t('advent_calendar.reward_invitations')" sortable />
        </DataTable>
      </PaginatedResults>
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Column, DataTable, Dialog, InputText, type DataTableSortEvent } from 'primevue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import TimeAgo from '@/components/TimeAgo.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import { useAdventCalendarRewards } from '@/composables/useAdventCalendarRewards'
import { bytesToReadable, formatBp } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import {
  AdventCalendarDoorOpenerOrderByColumn,
  getAdventCalendarDoorStats,
  OrderByDirection,
  type AdventCalendarDoorOpener,
  type AdventCalendarDoorRewardTotals,
} from '@/services/api-schema'

const doorId = defineModel<number | null>('doorId', { required: true })

const { t } = useI18n()
const formatRewards = useAdventCalendarRewards()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const pageSize = 100
const page = ref(1)
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))
const openingCount = ref(0)
const rewardTotals = ref<AdventCalendarDoorRewardTotals | null>(null)
const openers = ref<AdventCalendarDoorOpener[]>([])
const username = ref('')
const sortField = ref<AdventCalendarDoorOpenerOrderByColumn>(AdventCalendarDoorOpenerOrderByColumn.OpenedAt)
const sortOrder = ref<1 | -1>(-1)
const loading = ref(false)

const fetchStats = (newPage: number) => {
  if (doorId.value === null) return
  page.value = newPage
  loading.value = true
  getAdventCalendarDoorStats({
    door_id: doorId.value,
    username: username.value,
    order_by_column: sortField.value,
    order_by_direction: sortOrder.value === 1 ? OrderByDirection.Asc : OrderByDirection.Desc,
    page: newPage,
    page_size: pageSize,
  })
    .then((stats) => {
      openingCount.value = stats.opening_count
      rewardTotals.value = stats.reward_totals
      openers.value = stats.openers.results
      totalItems.value = stats.openers.total_items
    })
    .finally(() => (loading.value = false))
}

const onSort = (event: DataTableSortEvent) => {
  sortField.value = event.sortField as AdventCalendarDoorOpenerOrderByColumn
  sortOrder.value = event.sortOrder === 1 ? 1 : -1
  fetchStats(1)
}

// the component stays mounted between doors, so the previous door's results and filter are cleared
watch(
  doorId,
  () => {
    username.value = ''
    openers.value = []
    rewardTotals.value = null
    fetchStats(1)
  },
  { immediate: true },
)
</script>

<style scoped>
.door-stats {
  width: 75em;
  max-width: 100%;
}
.opening-count {
  font-weight: bold;
}
.reward-totals {
  margin: 5px 0 10px;
}
.avatar {
  width: 50px;
  border-radius: 7px;
}
</style>
