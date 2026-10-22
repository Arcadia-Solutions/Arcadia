<template>
  <!-- TODO: use skeletons while the data is loading -->
  <div v-if="titleGroupAndAssociatedData" id="title-group-view" class="with-sidebar">
    <div
      :class="{
        main: true,
      }"
    >
      <TitleGroupSlimHeader
        bold
        :titleGroup="titleGroupAndAssociatedData.title_group"
        :series="titleGroupAndAssociatedData.series"
        :affiliatedArtists="titleGroupAndAssociatedData.affiliated_artists.map((artist) => ({ artist_id: artist.artist.id, name: artist.artist.name }))"
        class="slim-header title"
      />
      <div class="actions">
        <div>
          <i v-if="togglingTorrentSubscription" class="pi pi-hourglass" />
          <i
            v-else
            v-tooltip.top="t(`title_group.${titleGroupAndAssociatedData.is_subscribed_to_torrents ? 'un' : ''}subscribe_to_torrents`)"
            @click="toggleTorrentSubscribtion"
            :class="`pi pi-bell${titleGroupAndAssociatedData.is_subscribed_to_torrents ? '-slash' : ''}`"
          />
          <span class="icon-letter">T</span>
          <i v-if="togglingCommentSubscription" class="pi pi-hourglass" />
          <i
            v-else
            v-tooltip.top="t(`title_group.${titleGroupAndAssociatedData.is_subscribed_to_comments ? 'un' : ''}subscribe_to_comments`)"
            @click="toggleCommentSubscribtion"
            :class="`pi pi-bell${titleGroupAndAssociatedData.is_subscribed_to_comments ? '-slash' : ''}`"
          />
          <span class="icon-letter">C</span>
          <i v-if="removingBookmark" class="pi pi-hourglass" />
          <i
            v-else-if="titleGroupAndAssociatedData.bookmark_id"
            v-tooltip.top="t('general.unbookmark')"
            class="pi pi-bookmark slashed"
            @click="removeBookmark(titleGroupAndAssociatedData.bookmark_id)"
          />
          <i v-else v-tooltip.top="t('general.bookmark')" class="pi pi-bookmark" @click="bookmarkDialogVisible = true" />
        </div>
        <div>
          <i
            v-if="titleGroupAndAssociatedData.title_group.created_by_id === userStore.id || userStore.permissions.includes('edit_title_group')"
            v-tooltip.top="t('general.edit')"
            class="pi pi-pen-to-square"
            @click="editTitleGroupDialogVisible = true"
          />
          <i
            v-if="userStore.permissions.includes('delete_title_group')"
            v-tooltip.top="t('general.delete')"
            class="pi pi-trash"
            @click="deleteTitleGroupDialogVisible = true"
          />
          <i
            v-if="userStore.permissions.includes('merge_title_group') || userStore.id === titleGroupAndAssociatedData.title_group.created_by_id"
            v-tooltip.top="t('title_group.merge_title_group')"
            class="pi pi-arrow-down-left-and-arrow-up-right-to-center"
            @click="mergeTitleGroupDialogVisible = true"
          />
          <i @click="uploadTorrent" v-tooltip.top="t('torrent.add_format')" class="pi pi-upload" />
          <i @click="requestTorrent" v-tooltip.top="t('torrent.request_format')" class="pi pi-shopping-cart" />
          <i @click="addCollagesDialogVisible = true" v-tooltip.top="t('collage.add_collage_to_entry', 2)" class="pi pi-folder-plus" />
        </div>
        <FloatLabel class="sort-by-select" variant="on">
          <Select v-model="sortBy" inputId="sort_by" :options="selectableSortingOptions" class="select" size="small">
            <template #option="slotProps">
              <span>{{ t(`torrent.${slotProps.option}`) }}</span>
            </template>
            <template #value="slotProps">
              <span>{{ t(`torrent.${slotProps.value}`) }}</span>
            </template>
          </Select>
          <label for="sort_by">{{ t('general.sort_by') }}</label>
        </FloatLabel>
      </div>
      <TitleGroupTable
        :showActionBtns="true"
        :title_group="titleGroupAndAssociatedData.title_group"
        :editionGroups="titleGroupAndAssociatedData.edition_groups"
        :sortBy
        :preview="false"
        showHeaders
        :seriesName="titleGroupAndAssociatedData.series?.name"
        :artistNames="titleGroupAndAssociatedData.affiliated_artists.map((a) => a.artist.name)"
        @editionGroupDeleted="editionGroupDeleted"
      />
      <ContentContainer :container-title="t('general.screenshots')" class="screenshots" v-if="titleGroupAndAssociatedData.title_group.screenshots.length !== 0">
        <CustomGalleria :images="titleGroupAndAssociatedData.title_group.screenshots" />
      </ContentContainer>
      <Accordion v-if="titleGroupAndAssociatedData.collages.length != 0" value="0" class="dense-accordion">
        <AccordionPanel value="0">
          <AccordionHeader> {{ t('collage.collage', 2) }} ({{ titleGroupAndAssociatedData.collages.length }}) </AccordionHeader>
          <AccordionContent>
            <CollagesTable :collages="titleGroupAndAssociatedData.collages" />
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
      <Accordion v-if="titleGroupAndAssociatedData.torrent_requests.length != 0" value="0" class="dense-accordion">
        <AccordionPanel value="0">
          <AccordionHeader> {{ t('torrent.requests') }} ({{ titleGroupAndAssociatedData.torrent_requests.length }}) </AccordionHeader>
          <AccordionContent>
            <TorrentRequestsTable
              :torrentRequests="titleGroupAndAssociatedData.torrent_requests"
              :contentType="titleGroupAndAssociatedData.title_group.content_type"
            />
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
      <EmbeddedLinks
        class="embedded-links"
        v-if="Object.keys(titleGroupAndAssociatedData.title_group.trailers).length > 0"
        :links="titleGroupAndAssociatedData.title_group.trailers"
      />
      <ContentContainer class="description" :container-title="t('title_group.description')">
        <div class="title-group-description">
          <BBCodeRenderer :content="titleGroupAndAssociatedData.title_group.description" />
        </div>
        <div v-for="edition_group in titleGroupAndAssociatedData.edition_groups" :key="edition_group.id">
          <div v-if="edition_group.description" class="edition-description">
            <div class="edition-group-slug">{{ getEditionGroupSlug(edition_group) }}</div>
            <BBCodeRenderer :content="edition_group.description" />
          </div>
        </div>
      </ContentContainer>
      <TitleGroupRatings
        v-if="titleGroupAndAssociatedData.title_group.public_ratings.length > 0"
        :publicRatings="titleGroupAndAssociatedData.title_group.public_ratings"
        class="ratings"
      />
      <ContentContainer
        v-if="userStore.permissions.includes('link_similar_title_group') || titleGroupAndAssociatedData.similar_title_groups.length != 0"
        :container-title="t('title_group.similar_titles')"
        class="similar-titles"
      >
        <template #top-right>
          <i
            v-if="userStore.permissions.includes('link_similar_title_group')"
            class="pi pi-plus"
            v-tooltip.top="t('title_group.link_similar_title')"
            @click="linkSimilarTitleDialogVisible = true"
          />
        </template>
        <div v-if="titleGroupAndAssociatedData.similar_title_groups.length != 0" class="similar-titles-list">
          <div v-for="similar in titleGroupAndAssociatedData.similar_title_groups" :key="similar.id" class="similar-title">
            <RouterLink :to="`/title-group/${similar.id}`" v-tooltip.top="similarTitleTooltip(similar)">
              <img v-if="similar.cover" class="similar-title-cover" :src="similar.cover" :alt="similar.name" />
              <div v-else class="similar-title-cover no-cover" />
            </RouterLink>
            <i
              v-if="userStore.permissions.includes('unlink_similar_title_group')"
              class="pi pi-times-circle unlink-icon"
              v-tooltip.top="t('title_group.unlink_similar_title')"
              @click="unlinkSimilarTitle(similar.id)"
            />
          </div>
        </div>
      </ContentContainer>
      <TitleGroupComments
        :comments="titleGroupAndAssociatedData.title_group_comments"
        :isSubscribedToComments="titleGroupAndAssociatedData.is_subscribed_to_comments"
        @newComment="newComment"
        @commentEdited="commentEdited"
        @commentDeleted="commentDeleted"
        @subscribed="titleGroupAndAssociatedData!.is_subscribed_to_comments = true"
      />
    </div>
    <div class="sidebar">
      <TitleGroupSidebar
        :title_group="titleGroupAndAssociatedData.title_group"
        :inSameMasterGroup="titleGroupAndAssociatedData.in_same_master_group"
        :affiliatedArtists="titleGroupAndAssociatedData.affiliated_artists"
        :affiliatedEntities="titleGroupAndAssociatedData.affiliated_entities"
        :series="titleGroupAndAssociatedData.series"
        v-model:relatedThreads="titleGroupAndAssociatedData.related_threads"
        editAffiliationBtns
        @edit-affiliated-artists-clicked="editAffiliatedArtistsDialogVisible = true"
        @tag-applied="titleGroupAndAssociatedData.title_group.tags.push($event)"
        @tag-removed="titleGroupAndAssociatedData.title_group.tags = titleGroupAndAssociatedData.title_group.tags.filter((tag_name) => $event != tag_name)"
        @series-removed="titleGroupAndAssociatedData.series = { id: 0, name: '' }"
      />
    </div>
    <Dialog modal :header="t('title_group.edit_affiliated_artists')" v-model:visible="editAffiliatedArtistsDialogVisible">
      <EditArtistsModal
        :artists-affiliations="
          titleGroupAndAssociatedData.affiliated_artists.length === 0
            ? [{ artist_id: 0, nickname: null, roles: [], title_group_id: 0 }]
            : titleGroupAndAssociatedData.affiliated_artists
        "
        :content-type="titleGroupAndAssociatedData.title_group.content_type"
        :title-group-id="titleGroupAndAssociatedData.title_group.id"
        @cancelled="editAffiliatedArtistsDialogVisible = false"
        @done="affiliatedArtistsEdited"
      />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.edit_title_group')" v-model:visible="editTitleGroupDialogVisible">
      <CreateOrEditTitleGroup
        class="edit-title-group"
        v-if="titleGroupAndAssociatedData"
        :initialTitleGroup="titleGroupAndAssociatedData.title_group"
        editMode
        @done="titleGroupEdited"
      />
    </Dialog>
    <Dialog modal :header="t('collage.add_collage_to_entry', 2)" v-model:visible="addCollagesDialogVisible">
      <AddCollagesToEntryDialog :titleGroupId="titleGroupAndAssociatedData.title_group.id" @addedEntries="router.go(0)" />
    </Dialog>
    <LinkSimilarTitleGroupDialog
      v-model:visible="linkSimilarTitleDialogVisible"
      :titleGroupId="titleGroupAndAssociatedData.title_group.id"
      :linkedTitleGroupIds="titleGroupAndAssociatedData.similar_title_groups.map((s) => s.id)"
      @linked="similarTitleLinked"
    />
    <Dialog closeOnEscape modal :header="t('title_group.delete_title_group')" v-model:visible="deleteTitleGroupDialogVisible">
      <DeleteDialog
        :message="t('title_group.confirm_delete_title_group')"
        :action="() => deleteTitleGroup(titleGroupAndAssociatedData!.title_group.id)"
        :successMessage="t('title_group.title_group_deleted_success')"
        @deleted="titleGroupDeleted"
      />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.merge_title_group')" v-model:visible="mergeTitleGroupDialogVisible">
      <MergeTitleGroupDialog :titleGroupId="titleGroupAndAssociatedData.title_group.id" :sourceData="titleGroupAndAssociatedData" @merged="titleGroupMerged" />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('general.bookmark')" v-model:visible="bookmarkDialogVisible">
      <BookmarkTitleGroupDialog :titleGroupId="titleGroupAndAssociatedData.title_group.id" @bookmarked="titleGroupBookmarked" />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { config } from '@/config'
import { defineAsyncComponent, ref, watch } from 'vue'
import { useUserStore } from '@/stores/user'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import TitleGroupComments from '@/components/title_group/TitleGroupComments.vue'
import TitleGroupSidebar from '@/components/title_group/TitleGroupSidebar.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import TitleGroupTable from '@/components/title_group/TitleGroupTable.vue'
import TorrentRequestsTable from '@/components/torrent_request/TorrentRequestsTable.vue'
import Accordion from 'primevue/accordion'
import AccordionPanel from 'primevue/accordionpanel'
import AccordionHeader from 'primevue/accordionheader'
import AccordionContent from 'primevue/accordioncontent'
import TitleGroupSlimHeader from '@/components/title_group/TitleGroupSlimHeader.vue'
import { useTitleGroupStore } from '@/stores/titleGroup'
import TitleGroupRatings from '@/components/title_group/TitleGroupRatings.vue'
import FloatLabel from 'primevue/floatlabel'
import Select from 'primevue/select'
const CustomGalleria = defineAsyncComponent(() => import('@/components/CustomGalleria.vue'))
import { useRoute, useRouter } from 'vue-router'
import { getEditionGroupSlug, isAttributeUsed } from '@/services/helpers'
import { useI18n } from 'vue-i18n'
import { showToast } from '@/main'
const EditArtistsModal = defineAsyncComponent(() => import('@/components/artist/EditArtistsModal.vue'))
import { Dialog } from 'primevue'
const EmbeddedLinks = defineAsyncComponent(() => import('@/components/title_group/EmbeddedLinks.vue'))
const CreateOrEditTitleGroup = defineAsyncComponent(() => import('@/components/title_group/CreateOrEditTitleGroup.vue'))
import { useEditionGroupStore } from '@/stores/editionGroup'
import { onBeforeRouteLeave } from 'vue-router'
const AddCollagesToEntryDialog = defineAsyncComponent(() => import('@/components/collage/AddCollagesToEntryDialog.vue'))
const CollagesTable = defineAsyncComponent(() => import('@/components/collage/CollagesTable.vue'))
const LinkSimilarTitleGroupDialog = defineAsyncComponent(() => import('@/components/title_group/LinkSimilarTitleGroupDialog.vue'))
const DeleteDialog = defineAsyncComponent(() => import('@/components/DeleteDialog.vue'))
const MergeTitleGroupDialog = defineAsyncComponent(() => import('@/components/title_group/MergeTitleGroupDialog.vue'))
const BookmarkTitleGroupDialog = defineAsyncComponent(() => import('@/components/title_group/BookmarkTitleGroupDialog.vue'))
import {
  createTitleGroupCommentsSubscription,
  createTitleGroupTorrentsSubscription,
  deleteTitleGroup,
  getTitleGroup,
  removeTitleGroupBookmark,
  removeTitleGroupCommentsSubscription,
  removeTitleGroupTorrentsSubscription,
  unlinkSimilarTitleGroups,
  type AffiliatedArtistHierarchy,
  type SimilarTitleGroupLite,
  type EditedTitleGroupComment,
  type TitleGroup,
  type TitleGroupAndAssociatedData,
  type TitleGroupBookmark,
  type TitleGroupCommentHierarchy,
} from '@/services/api-schema'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()

const editAffiliatedArtistsDialogVisible = ref(false)
const addCollagesDialogVisible = ref(false)
const linkSimilarTitleDialogVisible = ref(false)
const deleteTitleGroupDialogVisible = ref(false)
const mergeTitleGroupDialogVisible = ref(false)
const bookmarkDialogVisible = ref(false)
const userStore = useUserStore()
const titleGroupStore = useTitleGroupStore()
const editTitleGroupDialogVisible = ref(false)

// TODO: add by extras
const selectableSortingOptions = ['edition', 'size', 'seeders', 'times_completed', 'created_at']

const titleGroupAndAssociatedData = ref<TitleGroupAndAssociatedData>()
const sortBy = ref('edition')
const togglingTorrentSubscription = ref(false)
const togglingCommentSubscription = ref(false)
const removingBookmark = ref(false)
const siteName = config.site_name

const commentEdited = (editedComment: EditedTitleGroupComment, commentId: number) => {
  if (!titleGroupAndAssociatedData.value) return
  const index = titleGroupAndAssociatedData.value.title_group_comments.findIndex((comment) => comment.id === commentId)
  if (index !== -1) {
    titleGroupAndAssociatedData.value.title_group_comments[index] = { ...titleGroupAndAssociatedData.value.title_group_comments[index], ...editedComment }
    showToast('', t('title_group.comment_edited_success'), 'success', 2000)
  }
}

const fetchTitleGroup = async () => {
  titleGroupAndAssociatedData.value = await getTitleGroup(parseInt(route.params.id.toString()))

  // add audio_codec to sorting options
  const audioCodecInSortingOptions = selectableSortingOptions.includes('audio_codec')
  const contentTypeShouldHaveAudioCodec = isAttributeUsed('audio_codec', titleGroupAndAssociatedData.value.title_group.content_type)
  if (contentTypeShouldHaveAudioCodec && !audioCodecInSortingOptions) selectableSortingOptions.unshift('audio_codec')
  else if (!contentTypeShouldHaveAudioCodec && audioCodecInSortingOptions) selectableSortingOptions.splice(selectableSortingOptions.indexOf('audio_codec'), 1)

  // add video_resolution to sorting options
  const resolutionInSortingOptions = selectableSortingOptions.includes('video_resolution')
  const contentTypeShouldHaveResolution = isAttributeUsed('video_resolution', titleGroupAndAssociatedData.value.title_group.content_type)
  if (contentTypeShouldHaveResolution && !resolutionInSortingOptions) selectableSortingOptions.unshift('video_resolution')
  else if (!contentTypeShouldHaveResolution && resolutionInSortingOptions) selectableSortingOptions.splice(selectableSortingOptions.indexOf('resolution'), 1)
  /*
    For series, the title group name just holds the season name (i.e. 'Season 1')
    so we want to show the series name itself in the document title as well.
  */
  document.title = titleGroupAndAssociatedData.value.series
    ? `${titleGroupAndAssociatedData.value.title_group.name} (${titleGroupAndAssociatedData.value.series.name}) - ${siteName}`
    : `${titleGroupAndAssociatedData.value.title_group.name} - ${siteName}`

  populateTitleGroupStore()
}

onBeforeRouteLeave((to, from, next) => {
  if (to.name !== 'UploadTorrent' && to.name !== 'NewTorrentRequest') {
    titleGroupStore.$reset()
    useEditionGroupStore().$reset()
  }
  // we must call `next()` to continue navigation
  next()
})

const populateTitleGroupStore = () => {
  if (titleGroupAndAssociatedData.value) {
    titleGroupStore.id = titleGroupAndAssociatedData.value.title_group.id
    titleGroupStore.original_release_date = titleGroupAndAssociatedData.value.title_group.original_release_date
    titleGroupStore.original_release_date_only_year_known = titleGroupAndAssociatedData.value.title_group.original_release_date_only_year_known
    titleGroupStore.name = titleGroupAndAssociatedData.value.title_group.name
    titleGroupStore.edition_groups = titleGroupAndAssociatedData.value.edition_groups
    titleGroupStore.content_type = titleGroupAndAssociatedData.value.title_group.content_type
  }
}

const uploadTorrent = async () => {
  router.push({ path: '/upload' })
}

const requestTorrent = () => {
  router.push({ path: '/new-torrent-request' })
}

const toggleTorrentSubscribtion = async () => {
  if (titleGroupAndAssociatedData.value) {
    togglingTorrentSubscription.value = true
    if (titleGroupAndAssociatedData.value.is_subscribed_to_torrents) {
      await removeTitleGroupTorrentsSubscription(parseInt(route.params.id.toString()))
    } else {
      await createTitleGroupTorrentsSubscription(parseInt(route.params.id.toString()))
    }
    titleGroupAndAssociatedData.value.is_subscribed_to_torrents = !titleGroupAndAssociatedData.value.is_subscribed_to_torrents
    showToast(
      'Success',
      t(`title_group.${titleGroupAndAssociatedData.value.is_subscribed_to_torrents ? 'subscription_successful' : 'unsubscription_successful'}`),
      'success',
      3000,
    )
    togglingTorrentSubscription.value = false
  }
}
const toggleCommentSubscribtion = async () => {
  if (titleGroupAndAssociatedData.value) {
    togglingCommentSubscription.value = true
    if (titleGroupAndAssociatedData.value.is_subscribed_to_comments) {
      await removeTitleGroupCommentsSubscription(parseInt(route.params.id.toString()))
    } else {
      await createTitleGroupCommentsSubscription(parseInt(route.params.id.toString()))
    }
    titleGroupAndAssociatedData.value.is_subscribed_to_comments = !titleGroupAndAssociatedData.value.is_subscribed_to_comments
    showToast(
      'Success',
      t(`title_group.${titleGroupAndAssociatedData.value.is_subscribed_to_comments ? 'subscription_successful' : 'unsubscription_successful'}`),
      'success',
      3000,
    )
    togglingCommentSubscription.value = false
  }
}

const newComment = (comment: TitleGroupCommentHierarchy) => {
  titleGroupAndAssociatedData.value?.title_group_comments.push(comment)
}

const commentDeleted = (commentId: number) => {
  if (!titleGroupAndAssociatedData.value) return
  titleGroupAndAssociatedData.value.title_group_comments = titleGroupAndAssociatedData.value.title_group_comments.filter((comment) => comment.id !== commentId)
}

const affiliatedArtistsEdited = (newAffiliatedArtists: AffiliatedArtistHierarchy[], removedAffiliatedArtistsIds: number[]) => {
  if (titleGroupAndAssociatedData.value) {
    titleGroupAndAssociatedData.value.affiliated_artists = titleGroupAndAssociatedData.value.affiliated_artists.filter((aa: AffiliatedArtistHierarchy) => {
      // removedAffiliatedArtistsIds.indexOf(aa.id) === -1
      // return aa
      return !removedAffiliatedArtistsIds.includes(aa.id)
    })
    titleGroupAndAssociatedData.value.affiliated_artists = titleGroupAndAssociatedData.value.affiliated_artists.concat(newAffiliatedArtists)
  }
  editAffiliatedArtistsDialogVisible.value = false
}

const titleGroupEdited = (updatedTitleGroup: TitleGroup) => {
  if (titleGroupAndAssociatedData.value) {
    titleGroupAndAssociatedData.value.title_group = { ...titleGroupAndAssociatedData.value.title_group, ...updatedTitleGroup }
  }
  editTitleGroupDialogVisible.value = false
}

const editionGroupDeleted = (editionGroupId: number) => {
  if (titleGroupAndAssociatedData.value) {
    titleGroupAndAssociatedData.value.edition_groups = titleGroupAndAssociatedData.value.edition_groups.filter((eg) => eg.id !== editionGroupId)
  }
}

const titleGroupDeleted = () => {
  deleteTitleGroupDialogVisible.value = false
  router.push({ path: '/' })
}

const titleGroupBookmarked = (bookmark: TitleGroupBookmark) => {
  if (titleGroupAndAssociatedData.value) {
    titleGroupAndAssociatedData.value.bookmark_id = bookmark.id
  }
  bookmarkDialogVisible.value = false
  showToast('', t('title_group.bookmarked_successfully'), 'success', 2000)
}

const removeBookmark = (bookmarkId: number) => {
  removingBookmark.value = true
  removeTitleGroupBookmark(bookmarkId)
    .then(() => {
      if (titleGroupAndAssociatedData.value) {
        titleGroupAndAssociatedData.value.bookmark_id = null
      }
      showToast('', t('title_group.unbookmarked_successfully'), 'success', 2000)
    })
    .finally(() => {
      removingBookmark.value = false
    })
}

const similarTitleTooltip = (similar: SimilarTitleGroupLite) => {
  let tooltip = similar.name
  if (similar.original_release_date) {
    tooltip += ` (${similar.original_release_date.substring(0, 4)})`
  }
  if (similar.note) {
    tooltip += `\n\n${t('collage.note')}: ${similar.note}`
  }
  return tooltip
}

const similarTitleLinked = (similar: SimilarTitleGroupLite) => {
  titleGroupAndAssociatedData.value?.similar_title_groups.push(similar)
}

const unlinkSimilarTitle = (similarId: number) => {
  if (!titleGroupAndAssociatedData.value) return
  const titleGroupId = titleGroupAndAssociatedData.value.title_group.id
  unlinkSimilarTitleGroups({ group_1: titleGroupId, group_2: similarId, note: null }).then(() => {
    if (!titleGroupAndAssociatedData.value) return
    titleGroupAndAssociatedData.value.similar_title_groups = titleGroupAndAssociatedData.value.similar_title_groups.filter((s) => s.id !== similarId)
    showToast('Success', t('title_group.similar_title_unlinked'), 'success', 4000)
  })
}

const titleGroupMerged = (targetId: number) => {
  mergeTitleGroupDialogVisible.value = false
  router.push({ path: `/title-group/${targetId}` })
}

watch(() => route.params.id, fetchTitleGroup, { immediate: true })
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
.actions i.slashed {
  position: relative;
}
.actions i.slashed::after {
  content: '';
  position: absolute;
  top: 50%;
  left: -15%;
  width: 130%;
  height: 2px;
  background-color: white;
  transform: rotate(-45deg);
}
.icon-letter {
  margin-left: -8px;
  font-size: 0.8em;
}
.screenshots {
  margin-top: 20px;
}
.dense-accordion {
  margin-top: 20px;
}
.similar-titles {
  margin: 20px 0;
}
.similar-titles .pi-plus {
  cursor: pointer;
  color: white;
}
.similar-titles-list {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 15px;
}
.similar-title {
  position: relative;
  width: 100px;
}
.similar-title-cover {
  width: 100px;
  height: auto;
  display: block;
  border-radius: 7px;
}
.similar-title-cover.no-cover {
  height: 140px;
  background-color: var(--color-background-secondary);
}
.unlink-icon {
  position: absolute;
  top: 3px;
  right: 3px;
  cursor: pointer;
  color: white;
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
.edition-description {
  margin-top: 15px;
}
.edition-description .edition-group-slug {
  color: var(--color-primary);
  margin-bottom: 5px;
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
