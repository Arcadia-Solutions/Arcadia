<template>
  <div v-if="torrentRequestAndAssociatedData" id="title-group-view" class="with-sidebar">
    <div class="main">
      <TitleGroupSlimHeader
        bold
        :titleGroup="torrentRequestAndAssociatedData.title_group"
        :series="torrentRequestAndAssociatedData.series"
        :affiliatedArtists="torrentRequestAndAssociatedData.affiliated_artists.map((artist) => ({ name: artist.artist.name, artist_id: artist.artist.id }))"
        nameLink
        class="slim-header title"
      />
      <div class="actions">
        <div>
          <i v-if="togglingSubscription" class="pi pi-hourglass" />
          <i
            v-else
            v-tooltip.top="t(`torrent_request.${torrentRequestAndAssociatedData.is_subscribed_to_comments ? 'un' : ''}subscribe_to_comments`)"
            @click="toggleCommentSubscription"
            :class="`pi pi-bell${torrentRequestAndAssociatedData.is_subscribed_to_comments ? '-slash' : ''}`"
          />
          <span class="icon-letter">C</span>
          <i v-tooltip.top="t('general.bookmark')" class="pi pi-bookmark" />
        </div>
        <div>
          <!-- <i
            v-if="titleGroupAndAssociatedData.title_group.created_by_id === userStore.id || userStore.class === 'staff'"
            v-tooltip.top="t('general.edit')"
            class="pi pi-pen-to-square"
            @click="editTitleGroupDialogVisible = true"
          /> -->
          <i @click="uploadTorrent" v-tooltip.top="t('torrent.upload_torrent')" class="pi pi-upload" />
          <i
            v-if="canUnfillTorrentRequest"
            @click="unfillTorrentRequestDialogVisible = true"
            v-tooltip.top="t('torrent_request.unfill_request')"
            class="pi pi-undo"
          />
          <i
            v-if="canDeleteTorrentRequest"
            @click="deleteTorrentRequestDialogVisible = true"
            v-tooltip.top="t('torrent_request.delete_request')"
            class="pi pi-trash"
          />
          <!-- <i @click="requestTorrent" v-tooltip.top="t('torrent.request_format')" class="pi pi-shopping-cart" /> -->
        </div>
      </div>
      <Dialog closeOnEscape modal :header="t('torrent_request.delete_request')" v-model:visible="deleteTorrentRequestDialogVisible">
        <DeleteTorrentRequestDialog
          :torrentRequestId="torrentRequestAndAssociatedData.torrent_request.id"
          :deletingWithPermission="hasDeleteTorrentRequestPermission"
          @deleted="torrentRequestDeleted"
        />
      </Dialog>
      <Dialog closeOnEscape modal :header="t('torrent_request.unfill_request')" v-model:visible="unfillTorrentRequestDialogVisible">
        <UnfillTorrentRequestDialog :torrentRequestId="torrentRequestAndAssociatedData.torrent_request.id" @unfilled="torrentRequestUnfilled" />
      </Dialog>
      <TorrentRequestDetails
        :torrentRequest="torrentRequestAndAssociatedData.torrent_request"
        :votes="torrentRequestAndAssociatedData.votes"
        :contentType="torrentRequestAndAssociatedData.title_group.content_type"
        :filledByUser="torrentRequestAndAssociatedData.filled_by_user"
        @voted="voted"
        @filled="filled"
      />
      <TorrentRequestVotesTable class="votes-table" :votes="torrentRequestAndAssociatedData.votes" />
      <TorrentRequestUnfillsTable
        v-if="torrentRequestAndAssociatedData.unfills.length > 0"
        class="unfills-table"
        :unfills="torrentRequestAndAssociatedData.unfills"
      />
      <!-- <ContentContainer :container-title="t('general.screenshots')" class="screenshots" v-if="titleGroupAndAssociatedData.title_group.screenshots.length !== 0">
        <CustomGalleria :images="titleGroupAndAssociatedData.title_group.screenshots" />
      </ContentContainer> -->
      <!-- <Accordion v-if="titleGroupAndAssociatedData.torrent_requests.length != 0" value="0" class="torrent-requests dense-accordion">
        <AccordionPanel value="0">
          <AccordionHeader> {{ t('torrent.requests') }} ({{ titleGroupAndAssociatedData.torrent_requests.length }}) </AccordionHeader>
          <AccordionContent>
            <TorrentRequestsTable
              :torrentRequests="titleGroupAndAssociatedData.torrent_requests"
              :contentType="titleGroupAndAssociatedData.title_group.content_type"
            />
          </AccordionContent>
        </AccordionPanel>
      </Accordion> -->
      <!-- <EmbeddedLinks
        class="embedded-links"
        v-if="Object.keys(titleGroupAndAssociatedData.title_group.trailers).length > 0"
        :links="titleGroupAndAssociatedData.title_group.trailers"
      /> -->
      <ContentContainer class="description" :container-title="t('title_group.description')">
        <BBCodeRenderer :content="torrentRequestAndAssociatedData.title_group.description" />
      </ContentContainer>
      <ContentContainer class="description" :container-title="t('torrent_request.description')">
        <BBCodeRenderer :content="torrentRequestAndAssociatedData.torrent_request.description" />
      </ContentContainer>
      <TorrentRequestComments :comments="torrentRequestAndAssociatedData.comments" @newComment="newComment" @commentDeleted="commentDeleted" />
    </div>
    <div class="sidebar">
      <TitleGroupSidebar
        :title_group="torrentRequestAndAssociatedData.title_group"
        :affiliatedArtists="torrentRequestAndAssociatedData.affiliated_artists"
        :series="torrentRequestAndAssociatedData.series"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Dialog } from 'primevue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import DeleteTorrentRequestDialog from '@/components/torrent_request/DeleteTorrentRequestDialog.vue'
import UnfillTorrentRequestDialog from '@/components/torrent_request/UnfillTorrentRequestDialog.vue'
import TorrentRequestUnfillsTable from '@/components/torrent_request/TorrentRequestUnfillsTable.vue'
import TitleGroupSidebar from '@/components/title_group/TitleGroupSidebar.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import TorrentRequestVotesTable from '@/components/torrent_request/TorrentRequestVotesTable.vue'
import TitleGroupSlimHeader from '@/components/title_group/TitleGroupSlimHeader.vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { showToast } from '@/main'
import TorrentRequestDetails from '@/components/torrent_request/TorrentRequestDetails.vue'
import TorrentRequestComments from '@/components/torrent_request/TorrentRequestComments.vue'
import {
  createTorrentRequestCommentsSubscription,
  getTitleGroupInfoLite,
  getTorrentRequest,
  removeTorrentRequestCommentsSubscription,
  type TorrentRequestAndAssociatedData,
  type TorrentRequestCommentHierarchy,
  type TorrentRequestVoteHierarchy,
} from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { useTitleGroupStore } from '@/stores/titleGroup'

const router = useRouter()
const route = useRoute()
const userStore = useUserStore()
const titleGroupStore = useTitleGroupStore()
const { t } = useI18n()

const torrentRequestAndAssociatedData = ref<TorrentRequestAndAssociatedData>()
const togglingSubscription = ref(false)
const deleteTorrentRequestDialogVisible = ref(false)
const unfillTorrentRequestDialogVisible = ref(false)

const hasDeleteTorrentRequestPermission = computed(() => userStore.permissions.includes('delete_torrent_request'))
const hasVotesFromOtherUsers = computed(() => torrentRequestAndAssociatedData.value?.votes.some((vote) => vote.created_by_id !== userStore.id) ?? false)
// a filled request can never be deleted, and the author can only delete their own request as long as nobody else spent a bounty on it
const canDeleteTorrentRequest = computed(
  () =>
    !torrentRequestAndAssociatedData.value?.torrent_request.filled_at &&
    (hasDeleteTorrentRequestPermission.value ||
      (torrentRequestAndAssociatedData.value?.torrent_request.created_by_id === userStore.id && !hasVotesFromOtherUsers.value)),
)

const canUnfillTorrentRequest = computed(
  () =>
    !!torrentRequestAndAssociatedData.value?.torrent_request.filled_at &&
    (torrentRequestAndAssociatedData.value.torrent_request.created_by_id === userStore.id || userStore.permissions.includes('manage_unfilled_requests')),
)

const fetchTorrentRequest = async () => {
  torrentRequestAndAssociatedData.value = await getTorrentRequest(parseInt(route.params.id.toString()))

  /*
    For series, the title group name just holds the season name (i.e. 'Season 1')
    so we want to show the series name itself in the document title as well.
  */
  // document.title = titleGroupAndAssociatedData.value.series.name
  //   ? `${titleGroupAndAssociatedData.value.title_group.name} (${titleGroupAndAssociatedData.value.series.name}) - ${siteName}`
  //   : `${titleGroupAndAssociatedData.value.title_group.name} - ${siteName}`
}

const uploadTorrent = () => {
  if (torrentRequestAndAssociatedData.value) {
    const tg = torrentRequestAndAssociatedData.value.title_group
    getTitleGroupInfoLite(tg.id).then((titleGroupsLite) => {
      const titleGroupLite = titleGroupsLite[0]
      if (titleGroupLite) {
        titleGroupStore.id = titleGroupLite.id
        titleGroupStore.name = titleGroupLite.name
        titleGroupStore.content_type = titleGroupLite.content_type
        titleGroupStore.covers = titleGroupLite.covers
        titleGroupStore.original_release_date = titleGroupLite.original_release_date
        titleGroupStore.original_release_date_only_year_known = titleGroupLite.original_release_date_only_year_known
        titleGroupStore.edition_groups = titleGroupLite.edition_groups
      }
      router.push({ path: '/upload' })
    })
  }
}

const toggleCommentSubscription = () => {
  if (torrentRequestAndAssociatedData.value) {
    togglingSubscription.value = true
    const id = parseInt(route.params.id.toString())
    const action = torrentRequestAndAssociatedData.value.is_subscribed_to_comments
      ? removeTorrentRequestCommentsSubscription(id)
      : createTorrentRequestCommentsSubscription(id)
    action
      .then(() => {
        torrentRequestAndAssociatedData.value!.is_subscribed_to_comments = !torrentRequestAndAssociatedData.value!.is_subscribed_to_comments
        showToast(
          'Success',
          t(`torrent_request.${torrentRequestAndAssociatedData.value!.is_subscribed_to_comments ? 'subscription_successful' : 'unsubscription_successful'}`),
          'success',
          3000,
        )
      })
      .finally(() => {
        togglingSubscription.value = false
      })
  }
}

const torrentRequestDeleted = () => {
  deleteTorrentRequestDialogVisible.value = false
  showToast('', t('torrent_request.deletion_successful'), 'success', 3000, true, 'tr')
  router.push({ path: `/title-group/${torrentRequestAndAssociatedData.value!.title_group.id}` })
}

const torrentRequestUnfilled = () => {
  unfillTorrentRequestDialogVisible.value = false
  showToast('', t('torrent_request.unfill_successful'), 'success', 3000, true, 'tr')
  fetchTorrentRequest()
}

const voted = (vote: TorrentRequestVoteHierarchy) => {
  if (torrentRequestAndAssociatedData.value) {
    torrentRequestAndAssociatedData.value.votes.push(vote)
  }
}

const newComment = (comment: TorrentRequestCommentHierarchy) => {
  torrentRequestAndAssociatedData.value?.comments.push(comment)
}

const commentDeleted = (commentId: number) => {
  if (!torrentRequestAndAssociatedData.value) return
  torrentRequestAndAssociatedData.value.comments = torrentRequestAndAssociatedData.value.comments.filter((comment) => comment.id !== commentId)
}

const filled = (torrentId: number, fillerIsUploader: boolean) => {
  if (torrentRequestAndAssociatedData.value) {
    torrentRequestAndAssociatedData.value.torrent_request.filled_at = new Date().toISOString()
    torrentRequestAndAssociatedData.value.torrent_request.filled_by_torrent_id = torrentId
    torrentRequestAndAssociatedData.value.filled_by_user = {
      id: userStore.id,
      username: userStore.username,
      warned: userStore.warned,
      banned: userStore.banned,
    }
    const totalBountyUpload =
      torrentRequestAndAssociatedData.value.votes.reduce((accumulator, currentObject) => {
        return accumulator + currentObject.bounty_upload
      }, 0) / (fillerIsUploader ? 1 : 2)
    userStore.uploaded += totalBountyUpload
    userStore.real_uploaded += totalBountyUpload
    userStore.bonus_points +=
      torrentRequestAndAssociatedData.value.votes.reduce((accumulator, currentObject) => {
        return accumulator + currentObject.bounty_bonus_points
      }, 0) / (fillerIsUploader ? 1 : 2)
  }
}

watch(() => route.params.id, fetchTorrentRequest, { immediate: true })
</script>

<style scoped>
.main {
  width: 75%;
}
.sidebar {
  width: 25%;
}
.actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 5px;
}
.actions i {
  margin: 0px 0.5em;
  color: white;
  cursor: pointer;
}
.icon-letter {
  margin-left: -8px;
  font-size: 0.8em;
}
.votes-table {
  margin-top: 20px;
}
.unfills-table {
  margin-top: 20px;
}
.embedded-links {
  margin-top: 20px;
}
.description {
  margin-top: 20px;
}
.title-group-description {
  margin-top: 10px;
  margin-bottom: 25px;
}
.ratings {
  margin-top: 20px;
}
.comments {
  margin-top: 20px;
}
.edit-title-group {
  width: 60vw;
}
</style>
<style>
#title-group-view {
  .p-tabpanel {
    line-height: 0 !important;
  }
  .p-tabpanels {
    padding: 0 !important;
  }
}
</style>
