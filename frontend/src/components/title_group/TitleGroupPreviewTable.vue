<template>
  <ContentContainer>
    <div class="title-group-preview-table">
      <!-- TODO : add tags and other potentially useful information -->
      <!-- TODO : clicking on a torrent should redirect to the title_group page
      edit the titlegrouptable component to have a prop that allows this -->
      <ImagePreview class="cover" :imageLink="title_group.covers[0]" />
      <div class="right">
        <div class="title">
          <TitleGroupSlimHeader
            :hideSeriesName
            :titleGroup="title_group"
            :series="title_group.series"
            :affiliatedArtists="title_group.affiliated_artists"
            nameLink
          />
        </div>
        <div class="tags">
          <span
            v-if="title_group.user_rating_average != null"
            v-tooltip.top="`${formatNumber(title_group.user_rating_count)} ${t('title_group.vote', title_group.user_rating_count)}`"
            class="user-rating"
          >
            <span v-if="publicArcadiaSettings.user_rating_label">{{ publicArcadiaSettings.user_rating_label }}:</span>
            <UserRatingDisplay :rating="title_group.user_rating_average" />
          </span>
          <span>
            <template v-for="(tag, index) in title_group.tags" :key="tag">
              <RouterLink :to="{ path: '/torrents', query: { title_group_tags: tag } }" @click="scrollToTop">{{ tag }}</RouterLink>
              <template v-if="index < title_group.tags.length - 1">, </template>
            </template>
          </span>
        </div>
        <TitleGroupTable v-if="title_group.edition_groups.length" :title_group="title_group" :editionGroups="title_group.edition_groups" :preview="true" />
      </div>
      <div class="left">
        <i class="pi pi-trash cursor-pointer" v-if="showDeleteBtn" v-tooltip.top="deleteBtnTooltip" @click="emit('delete', title_group.id)" />
      </div>
    </div>
  </ContentContainer>
</template>
<script setup lang="ts">
import TitleGroupTable from './TitleGroupTable.vue'
import ContentContainer from '../ContentContainer.vue'
import ImagePreview from '../ImagePreview.vue'
import type { TitleGroupHierarchyLite } from '@/services/api-schema'
import TitleGroupSlimHeader from './TitleGroupSlimHeader.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { useI18n } from 'vue-i18n'
import { formatNumber } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

defineProps<{
  title_group: TitleGroupHierarchyLite
  hideSeriesName?: boolean
  showDeleteBtn?: boolean
  deleteBtnTooltip?: string
}>()

const emit = defineEmits<{
  delete: [titleGroupId: number]
}>()

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const scrollToTop = () => window.scrollTo({ top: 0, behavior: 'smooth' })
</script>
<style scoped>
.title-group-preview-table {
  display: flex;
  justify-content: center;
  align-items: start;
}
.right {
  width: 100%;
}
.title {
  margin-top: -5px;
  font-size: 1.4em;
}
.user-rating {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-weight: bold;
  font-style: normal;
}
.tags {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.9em;
  font-weight: 350;
  font-style: italic;
}
</style>
<style>
.title-group-preview-table .cover {
  margin-right: 10px;
}
.title-group-preview-table .cover img {
  border-radius: 7px;
  width: 7em;
}
</style>
