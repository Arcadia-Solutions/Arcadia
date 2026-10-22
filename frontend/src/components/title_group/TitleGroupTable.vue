<template>
  <DataTable
    v-model:expandedRows="expandedRows"
    :value="sortedTorrents"
    :rowGroupMode="groupBy ? 'subheader' : undefined"
    :groupRowsBy="groupBy"
    tableStyle="min-width: 35rem"
    size="small"
    class="title-group-table"
    :showHeaders="false"
  >
    <Column>
      <template #body="slotProps">
        <TitleGroupTableTorrentRow
          v-if="!slotProps.data._empty"
          :torrent="slotProps.data"
          :titleGroup="title_group"
          :editionGroup="getEditionGroupById(slotProps.data.edition_group_id)"
          :preview="preview"
          :sortBy="sortBy"
          :showActionBtns="showActionBtns ?? false"
          :settingTorrentIdStaffChecked="settingTorrentIdStaffChecked"
          @report="reportTorrent"
          @delete="deleteTorrent"
          @edit="editTorrent"
          @toggleStaffChecked="toggleTorrentStaffChecked"
          @editFactors="editTorrentFactors"
          @moveToEditionGroup="openMoveTorrentDialog"
          @toggleRow="toggleRow"
          @download="handleDownload"
        />
        <span v-else class="empty-edition-group-label">{{ t('edition_group.no_torrents') }}</span>
      </template>
    </Column>
    <template #groupheader="slotProps" v-if="groupBy !== undefined">
      <div class="edition-group-header">
        <template v-if="groupBy === 'edition_group_id'">
          {{ getEditionGroupSlugById(slotProps.data.edition_group_id) }}
          <i
            v-if="
              showActionBtns &&
              (user.permissions.includes('edit_edition_group') || getEditionGroupCreatorIdById(slotProps.data.edition_group_id) === userStore.id)
            "
            v-tooltip.top="t('edition_group.edit_edition_group')"
            @click="editEditionGroup(slotProps.data.edition_group_id)"
            class="action pi pi-pen-to-square"
            style="color: white; margin-left: 3px; font-size: 0.8em"
          />
          <i
            v-if="
              showActionBtns &&
              (user.permissions.includes('delete_edition_group') || getEditionGroupCreatorIdById(slotProps.data.edition_group_id) === userStore.id)
            "
            v-tooltip.top="t('edition_group.delete_edition_group')"
            @click="openDeleteEditionGroupDialog(slotProps.data.edition_group_id)"
            class="action pi pi-trash"
            style="color: white; margin-left: 3px; font-size: 0.8em"
          />
        </template>
        <template v-else-if="groupBy === 'video_resolution'">{{ slotProps.data.video_resolution }}</template>
        <template v-else-if="groupBy === 'audio_codec'">{{ slotProps.data.audio_codec }}</template>
      </div>
    </template>
    <template #expansion="slotProps" v-if="!preview">
      <div class="pre-style release-name">{{ slotProps.data.release_name }}</div>
      <div v-if="userStore.show_upload_method" class="pre-style upload-method">
        {{ t('torrent.upload_method', { uploadMethod: slotProps.data.upload_method }) }}
      </div>
      <TorrentReseedRequest :torrent="slotProps.data" />
      <Accordion v-model:value="activeAccordionPanels[slotProps.data.id]" multiple class="dense-accordion">
        <AccordionPanel value="5" v-if="slotProps.data.trumpable">
          <AccordionHeader>{{ t('torrent.trump_reason') }}</AccordionHeader>
          <AccordionContent>
            {{ slotProps.data.trumpable }}
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel value="0" v-if="slotProps.data.reports.length > 0">
          <AccordionHeader>Report information</AccordionHeader>
          <AccordionContent>
            <TorrentReportsList :reports="slotProps.data.reports" @deleted="(reportId) => reportDeleted(slotProps.data, reportId)" />
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel v-if="slotProps.data.description" value="2">
          <AccordionHeader>{{ t('general.description') }}</AccordionHeader>
          <AccordionContent>
            <BBCodeRenderer :content="slotProps.data.description" />
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel value="1" v-if="slotProps.data.mediainfo !== null">
          <AccordionHeader class="aa">
            <div class="header-text">{{ t('torrent.mediainfo') }}</div>
          </AccordionHeader>
          <AccordionContent>
            <MediaInfoPreview :mediainfo="slotProps.data.mediainfo" />
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel v-if="slotProps.data.screenshots && slotProps.data.screenshots.length > 0" value="3">
          <AccordionHeader>{{ t('general.screenshots') }}</AccordionHeader>
          <AccordionContent>
            <div class="screenshots-container">
              <div v-for="(screenshot, index) in slotProps.data.screenshots" :key="index" class="screenshot">
                <ImagePreview class="screenshot-image" :imageLink="screenshot" />
              </div>
            </div>
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel value="4">
          <AccordionHeader>{{ t('torrent.file_list') }}</AccordionHeader>
          <AccordionContent>
            <DataTable :value="slotProps.data.file_list.files" tableStyle="min-width: 50rem">
              <Column field="name" :header="(slotProps.data.file_list.parent_folder ?? '') + '/'"></Column>
              <Column field="size" :header="t('torrent.size')">
                <template #body="slotProps">
                  {{ bytesToReadable(slotProps.data.size) }}
                </template>
              </Column>
            </DataTable>
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel value="6" v-if="userStore.permissions.includes('view_torrent_peers_and_activities')">
          <AccordionHeader>{{ t('torrent.peers') }}</AccordionHeader>
          <AccordionContent>
            <!-- Only load the component when the accordion panel is open -->
            <TorrentPeerTable v-if="activeAccordionPanels[slotProps.data.id]?.includes('6')" :torrentId="slotProps.data.id" />
          </AccordionContent>
        </AccordionPanel>
        <AccordionPanel value="7" v-if="userStore.permissions.includes('view_torrent_peers_and_activities')">
          <AccordionHeader>{{ t('torrent.activities') }}</AccordionHeader>
          <AccordionContent>
            <TorrentActivityTable v-if="activeAccordionPanels[slotProps.data.id]?.includes('7')" :torrentId="slotProps.data.id" />
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
    </template>
  </DataTable>
  <Dialog closeOnEscape modal :header="t('torrent.report_torrent')" v-model:visible="reportTorrentDialogVisible">
    <ReportTorrentDialog :torrentId="torrentIdBeingReported" @reported="torrentReported" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('torrent.delete_torrent')" v-model:visible="deleteTorrentDialogVisible">
    <DeleteTorrentDialog :torrentId="torrentIdBeingDeleted" @deleted="torrentDeleted" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('torrent.edit_torrent')" v-model:visible="editTorrentDialogVisible">
    <CreateOrEditTorrent v-if="torrentBeingEdited !== null" :initialTorrent="torrentBeingEdited" @done="torrentEdited" />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('edition_group.edit_edition_group')" v-model:visible="editEditionGroupDialogVisible">
    <CreateOrEditEditionGroup
      v-if="editionGroupBeingEdited !== null"
      :titleGroup="title_group"
      :initialEditionGroupForm="editionGroupBeingEdited"
      :sendingEditionGroup="sendingEditionGroup"
      @validated="editionGroupEdited"
    />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('edition_group.delete_edition_group')" v-model:visible="deleteEditionGroupDialogVisible">
    <DeleteDialog
      :message="t('edition_group.confirm_delete_edition_group')"
      :action="() => deleteEditionGroup(editionGroupIdBeingDeleted)"
      :successMessage="t('edition_group.edition_group_deleted_success')"
      @deleted="editionGroupDeleted"
    />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('torrent.edit_factors')" v-model:visible="editFactorsDialogVisible">
    <EditTorrentFactorsDialog
      v-if="torrentBeingEditedFactors !== null"
      :torrentId="torrentBeingEditedFactors.id"
      :initialUploadFactor="torrentBeingEditedFactors.upload_factor"
      :initialDownloadFactor="torrentBeingEditedFactors.download_factor"
      @done="torrentFactorsEdited"
    />
  </Dialog>
  <Dialog closeOnEscape modal :header="t('torrent.move_to_edition_group')" v-model:visible="moveTorrentDialogVisible">
    <MoveTorrentToEditionGroupDialog
      v-if="torrentBeingMoved !== null"
      :torrentId="torrentBeingMoved.id"
      :currentEditionGroupId="torrentBeingMoved.edition_group_id"
      :editionGroups="editionGroups"
      @done="torrentMoved"
    />
  </Dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import Accordion from 'primevue/accordion'
import AccordionPanel from 'primevue/accordionpanel'
import AccordionHeader from 'primevue/accordionheader'
import AccordionContent from 'primevue/accordioncontent'
import ReportTorrentDialog from '../torrent/ReportTorrentDialog.vue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import DeleteTorrentDialog from '../torrent/DeleteTorrentDialog.vue'
import Dialog from 'primevue/dialog'
import { downloadTorrent } from '@/services/api/torrentService'
import { useRoute } from 'vue-router'
import { bytesToReadable, getEditionGroupSlug, isAttributeUsed } from '@/services/helpers'
import { useI18n } from 'vue-i18n'
import CreateOrEditTorrent from '../torrent/CreateOrEditTorrent.vue'
import CreateOrEditEditionGroup from '../edition_group/CreateOrEditEditionGroup.vue'
import { useUserStore } from '@/stores/user'
import { useEditionGroupStore } from '@/stores/editionGroup'
import ImagePreview from '../ImagePreview.vue'
import MediaInfoPreview from '@/components/mediainfo/MediaInfoPreview.vue'
import {
  deleteEditionGroup,
  editEditionGroup as editEditionGroupApi,
  setTorrentStaffChecked,
  type EditionGroupHierarchy,
  type EditionGroupHierarchyLite,
  type EditionGroupInfoLite,
  type TitleGroup,
  type TitleGroupHierarchyLite,
  type TorrentHierarchyLite,
  type Torrent,
  type TorrentReport,
  type UserCreatedEditionGroup,
  AudioBitrateSampling,
  AudioCodec,
  Features,
  VideoCodec,
  VideoResolution,
} from '@/services/api-schema'
import TorrentPeerTable from '../torrent/TorrentPeerTable.vue'
import TorrentActivityTable from '../torrent/TorrentActivityTable.vue'
import EditTorrentFactorsDialog from '../torrent/EditTorrentFactorsDialog.vue'
import MoveTorrentToEditionGroupDialog from '../torrent/MoveTorrentToEditionGroupDialog.vue'
import TorrentReportsList from '../torrent/TorrentReportsList.vue'
import TitleGroupTableTorrentRow from './TitleGroupTableTorrentRow.vue'
import TorrentReseedRequest from './TorrentReseedRequest.vue'

interface Props {
  title_group: TitleGroup | TitleGroupHierarchyLite
  editionGroups: EditionGroupHierarchyLite[] | EditionGroupHierarchy[]
  preview: boolean
  sortBy?: string
  showActionBtns?: boolean
  seriesName?: string
  artistNames?: string[]
}
const { title_group, editionGroups, preview = false, sortBy = 'edition', seriesName, artistNames } = defineProps<Props>()

const { t } = useI18n()

const getSeriesName = (): string | undefined => {
  if ('series' in title_group && title_group.series) return title_group.series.name
  return seriesName
}

const getArtistNames = (): string[] | undefined => {
  if ('affiliated_artists' in title_group) return title_group.affiliated_artists.map((a) => a.name)
  return artistNames
}
const userStore = useUserStore()

const settingTorrentIdStaffChecked = ref<number | null>(null)
const reportTorrentDialogVisible = ref(false)
const deleteTorrentDialogVisible = ref(false)
const editTorrentDialogVisible = ref(false)
const editEditionGroupDialogVisible = ref(false)
const deleteEditionGroupDialogVisible = ref(false)
const editionGroupIdBeingDeleted = ref(0)
const editFactorsDialogVisible = ref(false)
const moveTorrentDialogVisible = ref(false)
const torrentBeingMoved = ref<TorrentHierarchyLite | null>(null)
const torrentBeingEditedFactors = ref<{ id: number; upload_factor: number; download_factor: number } | null>(null)
const torrentBeingEdited = ref<TorrentHierarchyLite | null>(null)
const editionGroupBeingEdited = ref<UserCreatedEditionGroup | null>(null)
const editionGroupIdBeingEdited = ref<number | null>(null)
const sendingEditionGroup = ref(false)
const expandedRows = ref<TorrentHierarchyLite[]>([])
const torrentIdBeingReported = ref(0)
const torrentIdBeingDeleted = ref(0)
const route = useRoute()
const user = useUserStore()
const activeAccordionPanels = ref<Record<number, string[]>>({})

const handleDownload = (torrent: TorrentHierarchyLite) => {
  downloadTorrent(torrent.id, title_group.name, getSeriesName(), getArtistNames())
}

const toggleTorrentStaffChecked = ({ torrent_id, staff_checked }: { torrent_id: number; staff_checked: boolean }) => {
  settingTorrentIdStaffChecked.value = torrent_id
  setTorrentStaffChecked({ torrent_id, staff_checked })
    .then(() => {
      editionGroups.forEach((edition_group) => {
        edition_group.torrents.forEach((torrent) => {
          if (torrent.id === torrent_id) {
            torrent.staff_checked = staff_checked
          }
        })
      })
    })
    .finally(() => {
      settingTorrentIdStaffChecked.value = null
    })
}

const reportDeleted = (torrent: TorrentHierarchyLite, reportId: number) => {
  torrent.reports = torrent.reports.filter((r) => r.id !== reportId)
}

const torrentReported = (torrentReport: TorrentReport) => {
  reportTorrentDialogVisible.value = false
  const reportedTorrent = editionGroups
    .flatMap((edition_group) => edition_group.torrents)
    .find((torrent: TorrentHierarchyLite) => torrent.id == torrentReport.reported_torrent_id)
  if (reportedTorrent) {
    if (reportedTorrent.reports) {
      reportedTorrent.reports.push(torrentReport)
    } else {
      reportedTorrent.reports = [torrentReport]
    }
  } else {
    console.error('torrent to report not found !')
  }
}
const torrentDeleted = () => {
  editionGroups.forEach((edition_group) => {
    edition_group.torrents = edition_group.torrents.filter((torrent) => torrent.id !== torrentIdBeingDeleted.value)
  })
  deleteTorrentDialogVisible.value = false
}
const reportTorrent = (id: number) => {
  torrentIdBeingReported.value = id
  reportTorrentDialogVisible.value = true
}
const editTorrent = (torrent: TorrentHierarchyLite) => {
  torrentBeingEdited.value = torrent
  useEditionGroupStore().additional_information = getEditionGroupById(torrent.edition_group_id).additional_information
  useEditionGroupStore().source = getEditionGroupById(torrent.edition_group_id).source
  editTorrentDialogVisible.value = true
}
const editEditionGroup = (editionGroupId: number) => {
  const eg = editionGroups.find((g) => g.id === editionGroupId) as EditionGroupHierarchy
  editionGroupIdBeingEdited.value = editionGroupId
  editionGroupBeingEdited.value = {
    name: eg.name,
    description: eg.description,
    external_links: eg.external_links,
    covers: eg.covers,
    release_date: eg.release_date,
    release_date_only_year_known: eg.release_date_only_year_known,
    title_group_id: eg.title_group_id,
    source: eg.source,
    distributor: eg.distributor,
    additional_information: eg.additional_information,
  }
  editEditionGroupDialogVisible.value = true
}
const editionGroupEdited = async (updatedEditionGroup: UserCreatedEditionGroup) => {
  if (editionGroupIdBeingEdited.value === null) return
  const eg = editionGroups.find((g) => g.id === editionGroupIdBeingEdited.value)
  if (!eg) return
  sendingEditionGroup.value = true
  editEditionGroupApi({
    ...updatedEditionGroup,
    id: editionGroupIdBeingEdited.value,
  })
    .then((result) => {
      Object.assign(eg, result)
      editEditionGroupDialogVisible.value = false
    })
    .finally(() => (sendingEditionGroup.value = false))
}
const openDeleteEditionGroupDialog = (editionGroupId: number) => {
  editionGroupIdBeingDeleted.value = editionGroupId
  deleteEditionGroupDialogVisible.value = true
}
const emit = defineEmits<{
  editionGroupDeleted: [editionGroupId: number]
}>()
const editionGroupDeleted = () => {
  emit('editionGroupDeleted', editionGroupIdBeingDeleted.value)
  deleteEditionGroupDialogVisible.value = false
}
const deleteTorrent = (torrentId: number) => {
  torrentIdBeingDeleted.value = torrentId
  deleteTorrentDialogVisible.value = true
}
const editTorrentFactors = (torrent: TorrentHierarchyLite) => {
  torrentBeingEditedFactors.value = {
    id: torrent.id,
    upload_factor: torrent.upload_factor,
    download_factor: torrent.download_factor,
  }
  editFactorsDialogVisible.value = true
}
const openMoveTorrentDialog = (torrent: TorrentHierarchyLite) => {
  torrentBeingMoved.value = torrent
  moveTorrentDialogVisible.value = true
}
const torrentMoved = (targetEditionGroupId: number) => {
  if (torrentBeingMoved.value === null) return
  const torrentId = torrentBeingMoved.value.id
  editionGroups.forEach((eg) => {
    eg.torrents = eg.torrents.filter((t) => t.id !== torrentId)
  })
  const targetEditionGroup = editionGroups.find((eg) => eg.id === targetEditionGroupId)
  if (targetEditionGroup && torrentBeingMoved.value) {
    torrentBeingMoved.value.edition_group_id = targetEditionGroupId
    targetEditionGroup.torrents.push(torrentBeingMoved.value)
  }
  moveTorrentDialogVisible.value = false
}
const torrentFactorsEdited = (uploadFactor: number, downloadFactor: number) => {
  if (torrentBeingEditedFactors.value === null) return
  editionGroups.forEach((eg) => {
    const torrent = eg.torrents.find((t) => t.id === torrentBeingEditedFactors.value!.id)
    if (torrent) {
      torrent.upload_factor = uploadFactor
      torrent.download_factor = downloadFactor
    }
  })
  editFactorsDialogVisible.value = false
}
const toggleRow = (torrent: TorrentHierarchyLite) => {
  if (!expandedRows.value.some((expandedTorrent) => expandedTorrent.id === torrent.id)) {
    expandedRows.value = [...expandedRows.value, torrent]
  } else {
    expandedRows.value = expandedRows.value.filter((t) => t.id !== torrent.id)
  }
}

const editionGroupMap = computed(() => new Map(editionGroups.map((group) => [group.id, group])))

const getEditionGroupById = (editionGroupId: number): EditionGroupInfoLite => {
  return editionGroupMap.value.get(editionGroupId) as EditionGroupInfoLite
}
const getEditionGroupCreatorIdById = (editionGroupId: number): number => {
  return (editionGroupMap.value.get(editionGroupId) as EditionGroupHierarchy).created_by_id
}
const getEditionGroupSlugById = (editionGroupId: number): string => {
  const editionGroup = getEditionGroupById(editionGroupId)
  return editionGroup ? getEditionGroupSlug(editionGroup) : ''
}

onMounted(() => {
  const torrentIdParam = route.query.torrentId?.toString()
  if (torrentIdParam) {
    const torrentId = parseInt(torrentIdParam)
    const matchingTorrent = editionGroups.flatMap((edition_group) => edition_group.torrents).find((torrent) => torrent.id === torrentId)

    if (matchingTorrent) {
      toggleRow(matchingTorrent)
    }
  }
})
// resolutions from lowest to highest, the index in this list is what torrents get compared on
const videoResolutionOrder: string[] = Object.values(VideoResolution)

const getVideoCategory = (torrent: TorrentHierarchyLite): number => {
  if (torrent.video_codec === VideoCodec.Bd50 || torrent.video_codec === VideoCodec.Uhd100) return 3
  if (torrent.extras.length > 0) return 2
  if (torrent.features.includes(Features.Remux)) return 1
  return 0
}

// Encodes, then remuxes, then extras, then full discs (heaviest at the bottom), lowest resolution
// first inside each of those. The kind has to be compared before the resolution: a remux carries
// the resolution of the disc it was made from (NTSC/PAL), which ranks below every encode otherwise.
const compareTorrentsByVideo = (a: TorrentHierarchyLite, b: TorrentHierarchyLite): number => {
  const categoryDiff = getVideoCategory(a) - getVideoCategory(b)
  if (categoryDiff !== 0) return categoryDiff
  const aIndex = videoResolutionOrder.indexOf(a.video_resolution as string)
  const bIndex = videoResolutionOrder.indexOf(b.video_resolution as string)
  if (aIndex !== bIndex) return aIndex - bIndex
  return a.size - b.size
}

// Extras are add-ons to another torrent, they are not a release in their own right
const isExtra = (torrent: TorrentHierarchyLite): boolean => torrent.extras.length > 0

// Music, audiobooks, podcasts and software have no video attributes, their torrents are ranked by audio quality instead
const hasVideoAttributes = computed(() => isAttributeUsed('video_codec', title_group.content_type))

// 24bit lossless, then lossless, then mp3 320 and mp3 V0
const getAudioQualityRank = (torrent: TorrentHierarchyLite): number => {
  if (torrent.audio_bitrate_sampling === AudioBitrateSampling._24bitLossless) return 0
  if (torrent.audio_bitrate_sampling === AudioBitrateSampling.Lossless) return 1
  if (torrent.audio_codec === AudioCodec.Mp3 && torrent.audio_bitrate_sampling === AudioBitrateSampling._320) return 2
  if (torrent.audio_codec === AudioCodec.Mp3 && torrent.audio_bitrate_sampling === AudioBitrateSampling.V0Vbr) return 3
  return 4
}

// Best quality first, then heaviest first, extras at the bottom
const compareTorrentsByAudio = (a: TorrentHierarchyLite, b: TorrentHierarchyLite): number => {
  const qualityDiff = getAudioQualityRank(a) - getAudioQualityRank(b)
  if (qualityDiff !== 0) return qualityDiff
  const sizeDiff = b.size - a.size
  if (sizeDiff !== 0) return sizeDiff
  return Number(isExtra(a)) - Number(isExtra(b))
}

// Extras are add-ons to another torrent, they are not a release in their own right, which the
// video category already takes care of by ranking them above full discs
const compareTorrentsByEdition = (a: TorrentHierarchyLite, b: TorrentHierarchyLite): number =>
  hasVideoAttributes.value ? compareTorrentsByVideo(a, b) : compareTorrentsByAudio(a, b)

// amounts and dates read best with the highest/newest first, enums keep their natural order
const descendingSortFields = new Set(['size', 'seeders', 'leechers', 'times_completed', 'created_at'])

const groupBy = computed(() => {
  switch (sortBy) {
    case 'edition':
      return 'edition_group_id'
    case 'video_resolution':
      return 'video_resolution'
    case 'audio_codec':
      return 'audio_codec'
    default:
      return undefined
  }
})
const sortedTorrents = computed(() => {
  // `sortBy` is a user facing option, `groupBy` is the field it actually maps to
  const sortField = groupBy.value ?? sortBy
  const flatTorrents = editionGroups.flatMap((edition_group: EditionGroupHierarchyLite) => edition_group.torrents)

  // Add placeholder rows for empty edition groups so their group header still appears
  if (sortBy === 'edition') {
    const emptyEditionGroups = editionGroups.filter((eg) => eg.torrents.length === 0)
    for (const eg of emptyEditionGroups) {
      flatTorrents.push({ edition_group_id: eg.id, _empty: true } as TorrentHierarchyLite & { _empty: boolean })
    }
  }

  const orderedEnums: Record<string, string[]> = {
    video_resolution: videoResolutionOrder,
    audio_codec: ['flac', 'true-hd', 'aac', 'ac3', 'dts', 'mp3', 'opus', 'mp2', 'pcm', 'dsd', 'cook', 'wma'],
  }

  const enumOrder = orderedEnums[sortField]
  const direction = descendingSortFields.has(sortField) ? -1 : 1
  const compareBySortField = (a: TorrentHierarchyLite, b: TorrentHierarchyLite): number => {
    const aVal = a[sortField as keyof TorrentHierarchyLite]
    const bVal = b[sortField as keyof TorrentHierarchyLite]
    if (enumOrder) return enumOrder.indexOf(aVal as string) - enumOrder.indexOf(bVal as string)
    if (aVal == null && bVal == null) return 0
    if (aVal == null) return 1
    if (bVal == null) return -1
    return aVal < bVal ? -direction : aVal > bVal ? direction : 0
  }

  return flatTorrents.sort((a, b) => {
    const primaryDiff = compareBySortField(a, b)
    if (primaryDiff !== 0) return primaryDiff
    // Ties have to be broken in a way that keeps the rows of a group adjacent, otherwise PrimeVue
    // renders one group header per run of equal rows instead of one per group.
    if (sortField === 'edition_group_id') return compareTorrentsByEdition(a, b)
    if (sortField === 'video_resolution') return compareTorrentsByVideo(a, b)
    if (sortField === 'audio_codec' && !hasVideoAttributes.value) return compareTorrentsByAudio(a, b)
    return 0
  })
})
const torrentEdited = (editedTorrent: Torrent) => {
  editionGroups.forEach((eg) => {
    const index = eg.torrents.findIndex((t) => t.id === editedTorrent.id)
    if (index !== -1) {
      eg.torrents[index] = { ...eg.torrents[index], ...editedTorrent }
    }
  })
  editTorrentDialogVisible.value = false
}
</script>
<style scoped>
.action {
  margin-right: 4px;
  cursor: pointer;
}
.edition-group-header {
  color: var(--color-primary);
}
.empty-edition-group-label {
  color: grey;
  font-style: italic;
}
.release-name,
.upload-method {
  margin-bottom: 10px;
  margin-left: 7px;
}
.screenshots-container {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 10px;
}

.screenshot {
  max-width: 200px;
}

.screenshot-image {
  width: 100%;
  height: auto;
  border-radius: 4px;
}
</style>
<style>
.title-group-table {
  .p-datatable-header-cell {
    padding: 7px 0 !important;
  }
  .p-accordionheader.aa {
    align-items: baseline;
  }
}
</style>
