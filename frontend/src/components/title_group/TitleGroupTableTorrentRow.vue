<template>
  <div class="torrent-row">
    <div class="cell cell-staff-check">
      <i
        class="pi pi-verified"
        :style="{ color: torrent.staff_checked ? 'green' : 'grey', 'margin-top': '0.3em' }"
        v-tooltip.top="torrent.staff_checked ? t('torrent.staff_check_present') : t('torrent.staff_check_missing')"
      />
    </div>
    <div class="cell cell-slug">
      <div class="cursor-pointer">
        <RouterLink v-if="preview" :to="`/title-group/${titleGroup.id}?torrentId=${torrent.id}`">
          <TorrentSlug :contentType="titleGroup.content_type" :torrent="torrent" :editionGroup="editionGroup" :sortedBy="sortBy" />
        </RouterLink>
        <a v-else @click="emit('toggleRow', torrent)">
          <TorrentSlug :contentType="titleGroup.content_type" :torrent="torrent" :editionGroup="editionGroup" :sortedBy="sortBy" />
        </a>
      </div>
    </div>
    <div class="cell cell-date">
      <TimeAgo :date="torrent.created_at" /> {{ t('general.by') }}
      <UsernameEnriched :user="torrent.created_by" noStatus />
      <span v-if="torrent.uploaded_as_anonymous && torrent.created_by"> ({{ t('general.anonymous') }})</span>
    </div>
    <div class="cell cell-actions">
      <i
        v-if="userStore.permissions.includes('download_torrent')"
        v-tooltip.top="t('torrent.download')"
        class="action pi pi-download"
        @click="emit('download', torrent)"
      />
      <i v-tooltip.top="t('general.report')" class="action pi pi-flag" @click="emit('report', torrent.id)" />
      <RouterLink :to="`/title-group/${titleGroup.id}?torrentId=${torrent.id}`" style="color: white">
        <i v-tooltip.top="t('torrent.permalink')" class="action pi pi-link" />
      </RouterLink>
      <i
        v-tooltip.top="t('general.delete')"
        class="action pi pi-trash"
        v-if="showActionBtns && (userStore.id === torrent.created_by?.id || userStore.permissions.includes('delete_torrent'))"
        @click="emit('delete', torrent.id)"
      />
      <i
        v-if="showActionBtns && (userStore.id === torrent.created_by?.id || userStore.permissions.includes('edit_torrent'))"
        v-tooltip.top="t('general.edit')"
        @click="emit('edit', torrent)"
        class="action pi pi-pen-to-square"
      />
      <i
        v-if="showActionBtns && userStore.permissions.includes('set_torrent_staff_checked')"
        v-tooltip.top="t(`torrent.${torrent.staff_checked ? 'unset_staff_checked' : 'set_staff_checked'}`)"
        @click="emit('toggleStaffChecked', { torrent_id: torrent.id, staff_checked: !torrent.staff_checked })"
        :class="{
          action: true,
          pi: true,
          'pi-verified': settingTorrentIdStaffChecked !== torrent.id,
          'pi-hourglass': settingTorrentIdStaffChecked === torrent.id,
        }"
        :style="`color: ${torrent.staff_checked ? 'green' : 'white'}`"
      />
      <i
        v-if="showActionBtns && (userStore.id === torrent.created_by?.id || userStore.permissions.includes('move_torrent_to_other_edition_group'))"
        v-tooltip.top="t('torrent.move_to_edition_group')"
        @click="emit('moveToEditionGroup', torrent)"
        class="action pi pi-arrow-right-arrow-left"
      />
      <i
        v-if="showActionBtns && userStore.permissions.includes('edit_torrent_up_down_factors')"
        v-tooltip.top="t('torrent.edit_factors')"
        @click="emit('editFactors', torrent)"
        class="action pi pi-percentage"
      />
    </div>
    <div class="cell cell-size">{{ bytesToReadable(torrent.size) }}</div>
    <div class="cell cell-bp">
      <span v-tooltip.top="publicArcadiaSettings.bonus_points_alias + ' ' + t('torrent.snatch_cost_hint')">
        {{ formatBp(torrent.bonus_points_snatch_cost, publicArcadiaSettings.bonus_points_decimal_places) }}
      </span>
    </div>
    <div class="cell cell-stat">
      <span v-tooltip.top="t('torrent.times_completed', 2)">{{ formatNumber(torrent.times_completed) }}</span>
    </div>
    <div class="cell cell-stat">
      <span style="color: green" v-tooltip.top="t('torrent.seeders')">{{ formatNumber(torrent.seeders) }}</span>
    </div>
    <div class="cell cell-stat">
      <span v-tooltip.top="t('torrent.leecher', 2)">{{ formatNumber(torrent.leechers) }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import TorrentSlug from '../torrent/TorrentSlug.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { RouterLink } from 'vue-router'
import { bytesToReadable, formatBp, formatNumber } from '@/services/helpers'
import { useI18n } from 'vue-i18n'
import { useUserStore } from '@/stores/user'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import type { EditionGroupInfoLite, TitleGroup, TitleGroupHierarchyLite, TorrentHierarchy, TorrentHierarchyLite } from '@/services/api-schema'

interface Props {
  torrent: TorrentHierarchyLite | TorrentHierarchy
  titleGroup: TitleGroup | TitleGroupHierarchyLite
  editionGroup: EditionGroupInfoLite
  preview: boolean
  sortBy: string
  showActionBtns: boolean
  settingTorrentIdStaffChecked: number | null
}
defineProps<Props>()

const emit = defineEmits<{
  report: [torrentId: number]
  delete: [torrentId: number]
  edit: [torrent: TorrentHierarchyLite]
  toggleStaffChecked: [payload: { torrent_id: number; staff_checked: boolean }]
  editFactors: [torrent: TorrentHierarchyLite]
  moveToEditionGroup: [torrent: TorrentHierarchyLite]
  toggleRow: [torrent: TorrentHierarchyLite]
  download: [torrent: TorrentHierarchyLite]
}>()

const { t } = useI18n()
const userStore = useUserStore()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
</script>

<style scoped>
.torrent-row {
  display: grid;
  grid-template-columns: 1.5em 1fr 13em 12em 6em 3em 2em 2em 2em;
  align-items: center;
}

.cell-date,
.cell-actions,
.cell-size,
.cell-bp {
  padding: 0;
}

.cell-bp {
  color: yellow;
}

.cell-stat {
  text-align: center;
}

.action {
  margin-right: 4px;
  cursor: pointer;
}
</style>
