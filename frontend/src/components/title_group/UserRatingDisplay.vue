<template>
  <span class="user-rating">
    <template v-if="rating !== null">
      <img v-if="icon" :src="icon.icon_url" alt="rating icon" class="icon" />
      <span>{{ Math.round(rating) }}%</span>
    </template>
    <span v-else class="no-rating">{{ t('title_group.no_ratings') }}</span>
    <span v-if="count !== undefined" class="votes">({{ formatNumber(count) }} {{ t('title_group.vote', count) }})</span>
  </span>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatNumber } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const props = defineProps<{
  rating: number | null
  count?: number
}>()

const { t } = useI18n()
const settings = usePublicArcadiaSettingsStore()

const icon = computed(() => {
  const rating = props.rating
  if (rating === null) return undefined
  return settings.rating_icons.filter((i) => i.minimum_rating <= rating).sort((a, b) => b.minimum_rating - a.minimum_rating)[0]
})
</script>
<style scoped>
.user-rating {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.icon {
  height: 1.2em;
}
.no-rating {
  opacity: 0.7;
}
.votes {
  font-size: 0.9em;
}
</style>
