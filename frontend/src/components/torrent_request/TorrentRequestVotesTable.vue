<template>
  <DataTable :value="votes" size="small">
    <Column :header="t('torrent_request.voter')">
      <template #body="slotProps">
        <UsernameEnriched :user="slotProps.data.created_by" />
      </template>
    </Column>
    <Column v-if="publicArcadiaSettings.torrent_request_vote_currencies.includes(TorrentRequestVoteCurrency.Upload)" :header="t('user.upload')">
      <template #body="slotProps">
        {{ bytesToReadable(slotProps.data.bounty_upload) }}
      </template>
    </Column>
    <Column
      v-if="publicArcadiaSettings.torrent_request_vote_currencies.includes(TorrentRequestVoteCurrency.BonusPoints)"
      :header="publicArcadiaSettings.bonus_points_alias"
    >
      <template #body="slotProps">
        {{ formatBp(slotProps.data.bounty_bonus_points, publicArcadiaSettings.bonus_points_decimal_places) }}
      </template>
    </Column>
    <Column :header="t('torrent_request.voted_at')">
      <template #body="slotProps">
        <TimeAgo :date="slotProps.data.created_at" />
      </template>
    </Column>
  </DataTable>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { Column, DataTable } from 'primevue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { bytesToReadable, formatBp } from '@/services/helpers'
import { useI18n } from 'vue-i18n'
import { TorrentRequestVoteCurrency, type TorrentRequestVoteHierarchy } from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

defineProps<{
  votes: TorrentRequestVoteHierarchy[]
}>()
</script>
<style scoped></style>
