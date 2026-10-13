<template>
  <ContentContainer :containerTitle :containerTitleLink>
    <template #top-right>
      <slot name="top-right"></slot>
    </template>
    <!-- While loading, reserve the height of one cover row (covers are 17% wide with a 2:3
         aspect ratio, so the placeholder scales with the viewport) so this block does not push
         the content below it down once the data arrives. -->
    <div class="last-uploads" v-if="loading" aria-hidden="true">
      <div class="title-group">
        <div class="cover-placeholder" />
        <div class="info-placeholder">&nbsp;</div>
      </div>
    </div>
    <div class="last-uploads" v-else-if="titleGroups">
      <TitleGroupPreviewCoverOnly
        v-for="titleGroup in titleGroups"
        :key="titleGroup.id"
        :titleGroup="titleGroup"
        :showUploader
        :loading="priority ? 'eager' : 'lazy'"
        :priority="priority"
        class="title-group"
      />
    </div>
    <div v-else>
      {{ t(`user.no_${props.type}_explanation`) }}
    </div>
  </ContentContainer>
</template>

<script setup lang="ts">
import type { TitleGroupLite } from '@/services/api-schema'
import ContentContainer from '../ContentContainer.vue'
import TitleGroupPreviewCoverOnly from '../title_group/TitleGroupPreviewCoverOnly.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  titleGroups: TitleGroupLite[]
  containerTitleLink: string
  containerTitle: string
  type?: 'snatches' | 'uploads'
  showUploader?: boolean
  loading?: boolean
  /** Above-the-fold hero row: load covers eagerly with a high priority hint (used for the LCP image). */
  priority?: boolean
}>()
</script>

<style>
.last-uploads {
  display: flex;
  justify-content: space-around;
  align-items: center;
  overflow-y: scroll;
  .title-group {
    margin: 0 5px;
  }
}
.title-group {
  width: 17%;
}
.cover-placeholder {
  width: 100%;
  /* Reserve a 2:3 box. `padding-top` is a percentage of the containing block width, so the
     reserved height always matches the real cover (which is width: 100% + aspect-ratio 2/3)
     and scales with the viewport. */
  padding-top: 150%;
  border-radius: 7px;
  background-color: var(--color-background-primary);
}
/* The real cover row keeps a constant height: reserve the same 3 lines of info text the real
   items use (timeAgo + "by <uploader>"), so the block never grows once the data arrives. */
.info-placeholder,
.last-uploads .latest-torrent-info {
  text-align: center;
  min-height: 4.8em;
}
.info-placeholder {
  visibility: hidden;
}
</style>
