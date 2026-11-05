<template>
  <div class="title-group-preview-cover-only">
    <RouterLink :to="`/title-group/${titleGroup.id}`">
      <img
        class="title-group-cover"
        :src="titleGroup.covers[0]"
        :loading="loading"
        :fetchpriority="priority ? 'high' : undefined"
        v-tooltip.top="titleGroup.name"
        alt="Title Group Cover"
      />
    </RouterLink>
    <div class="latest-torrent-info">
      <template v-if="titleGroup.latest_torrent_uploaded_at">
        <TimeAgo :date="titleGroup.latest_torrent_uploaded_at" />
      </template>
      <span v-if="showUploader" style="margin-left: 5px">
        {{ t('general.by') }}
        <UsernameEnriched :user="titleGroup.latest_torrent_uploaded_by" />
      </span>
    </div>
  </div>
</template>
<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import type { TitleGroupLite } from '@/services/api-schema'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

withDefaults(
  defineProps<{
    titleGroup: TitleGroupLite
    showUploader?: boolean
    /** Marks an above-the-fold cover so it loads with a high priority hint (used for the LCP image). */
    priority?: boolean
    /** Native `loading` mode. Above-the-fold rows should use `eager`; large grids default to `lazy`. */
    loading?: 'eager' | 'lazy'
  }>(),
  { loading: 'lazy' },
)
</script>
<style scoped>
.latest-torrent-info {
  text-align: center;
}
</style>
<style>
.title-group-preview-cover-only > a {
  display: block;
}
.title-group-preview-cover-only .title-group-cover {
  display: block;
  width: 100%;
  aspect-ratio: 2 / 3;
  object-fit: contain;
  border-radius: 7px;
}
.title-group-preview-cover-only a:hover {
  background-color: transparent;
}
</style>
