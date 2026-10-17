<template>
  <div class="title-group-preview-cover-only">
    <RouterLink :to="`/title-group/${titleGroup.id}`">
      <img
        class="title-group-cover"
        :src="coverSrc"
        :srcset="coverSrcset"
        :sizes="coverSrcset ? sizes : undefined"
        :loading="loading"
        :fetchpriority="priority ? 'high' : undefined"
        v-tooltip.top="titleGroup.name"
        alt="Title Group Cover"
      />
    </RouterLink>
    <div class="latest-torrent-info">
      <template v-if="titleGroup.latest_torrent_uploaded_at">
        {{ timeAgo(titleGroup.latest_torrent_uploaded_at) }}
      </template>
      <span v-if="showUploader" style="margin-left: 5px">
        {{ t('general.by') }}
        <UsernameEnriched :user="titleGroup.latest_torrent_uploaded_by" />
      </span>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import type { TitleGroupLite } from '@/services/api-schema'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import { useI18n } from 'vue-i18n'
import { timeAgo, tmdbImageAtWidth, tmdbImageSrcset } from '@/services/helpers'

const { t } = useI18n()

const props = withDefaults(
  defineProps<{
    titleGroup: TitleGroupLite
    showUploader?: boolean
    /** Marks an above-the-fold cover so it loads with a high priority hint (used for the LCP image). */
    priority?: boolean
    /** Native `loading` mode. Above-the-fold rows should use `eager`; large grids default to `lazy`. */
    loading?: 'eager' | 'lazy'
    /** `sizes` for the responsive srcset. Defaults to the width of the preview grids. */
    sizes?: string
  }>(),
  { loading: 'lazy', sizes: '(max-width: 767px) 18vw, 15vw' },
)

// The preview grids render covers at most a few hundred pixels wide, so never request the full w500
// original. This is a no-op for non-TMDB (user supplied) cover URLs.
const coverSrc = computed(() => tmdbImageAtWidth(props.titleGroup.covers[0], 342))
const coverSrcset = computed(() => tmdbImageSrcset(props.titleGroup.covers[0], [185, 342, 500]))
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
  object-fit: cover;
  border-radius: 7px;
}
.title-group-preview-cover-only a:hover {
  background-color: transparent;
}
</style>
