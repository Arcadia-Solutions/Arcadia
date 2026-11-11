<template>
  <ContentContainer :containerTitle="t('title_group.rating', 2)">
    <div class="ratings-component">
      <div class="rating">
        <img src="/favicon.ico" alt="site logo" class="logo" />
        <UserRatingDisplay :rating="userRatingAverage" :count="userRatingCount" />
      </div>
      <template v-for="rating in publicRatings" :key="rating.service">
        <div class="rating" v-if="rating.service === 'tmdb'">
          <img src="/logos/external_links/tmdb.svg" alt="tmdb logo" class="logo" />
          <div class="details">
            <span>{{ rating.rating * 10 }}%</span>
            <span class="votes">({{ formatNumber(rating.votes) }} {{ t('title_group.vote', 2) }})</span>
          </div>
        </div>
      </template>
    </div>
  </ContentContainer>
</template>
<script setup lang="ts">
import type { PublicRating } from '@/services/api-schema'
import ContentContainer from '../ContentContainer.vue'
import { useI18n } from 'vue-i18n'
import { formatNumber } from '@/services/helpers'
import UserRatingDisplay from './UserRatingDisplay.vue'

const { t } = useI18n()

defineProps<{
  publicRatings: PublicRating[]
  userRatingAverage: number | null
  userRatingCount: number
}>()
</script>
<style scoped>
.ratings-component {
  display: flex;
  justify-content: space-around;
}
.rating {
  display: flex;
  align-items: center;
  .logo {
    width: 3em;
    margin-right: 7px;
  }
  .details {
    display: flex;
    flex-direction: column;
    align-items: center;
    .votes {
      font-size: 0.9em;
    }
  }
}
</style>
