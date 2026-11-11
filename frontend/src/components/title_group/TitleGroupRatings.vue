<template>
  <ContentContainer :containerTitle="t('title_group.rating', 2)">
    <div class="ratings-component">
      <div class="rating">
        <div class="logo-and-note">
          <img src="/favicon.ico" alt="site logo" class="logo" />
          <UserRatingDisplay :rating="userRatingAverage" />
        </div>
        <div class="votes">
          <a v-if="userRatingCount > 0" class="vote-count cursor-pointer" @click="openReviews">
            ({{ formatNumber(userRatingCount) }} {{ t('title_group.vote', userRatingCount) }})
          </a>
          <a v-if="userStore.permissions.includes('rate_title_groups')" class="vote-link cursor-pointer" @click="openVoteForm">
            {{ t(ownRating === null ? 'title_group.add_your_vote' : 'title_group.edit_your_rating') }}
          </a>
        </div>
      </div>
      <template v-for="rating in publicRatings" :key="rating.service">
        <div class="rating" v-if="rating.service === 'tmdb'">
          <div class="logo-and-note">
            <img src="/logos/external_links/tmdb.svg" alt="tmdb logo" class="logo" />
            <span>{{ rating.rating * 10 }}%</span>
          </div>
          <span class="votes vote-count">({{ formatNumber(rating.votes) }} {{ t('title_group.vote', 2) }})</span>
        </div>
      </template>
    </div>
    <Dialog closeOnEscape modal :header="t('title_group.rating', 2)" v-model:visible="reviewsVisible" class="reviews-dialog">
      <TitleGroupReviews v-if="reviews" :reviews @deleted="reviewDeleted" />
      <ProgressSpinner v-else />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.your_rating')" v-model:visible="voteFormVisible">
      <TitleGroupVoteForm v-if="reviews" :titleGroupId :ownReview="reviews.find((r) => r.user_id === userStore.id)" @saved="reviewSaved" />
      <ProgressSpinner v-else />
    </Dialog>
  </ContentContainer>
</template>
<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { Dialog } from 'primevue'
import ProgressSpinner from 'primevue/progressspinner'
import { getTitleGroupReviews, type PublicRating, type TitleGroupReviewHierarchy } from '@/services/api-schema'
import ContentContainer from '../ContentContainer.vue'
import { useI18n } from 'vue-i18n'
import { formatNumber } from '@/services/helpers'
import { useUserStore } from '@/stores/user'
import UserRatingDisplay from './UserRatingDisplay.vue'
import TitleGroupReviews from './TitleGroupReviews.vue'
import TitleGroupVoteForm from './TitleGroupVoteForm.vue'

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()

const props = defineProps<{
  titleGroupId: number
  publicRatings: PublicRating[]
  userRatingAverage: number | null
  userRatingCount: number
  ownRating: number | null
}>()

const emit = defineEmits<{
  reviewsChanged: [TitleGroupReviewHierarchy[]]
}>()

const reviews = ref<TitleGroupReviewHierarchy[]>()
const reviewsVisible = ref(false)
const voteFormVisible = ref(false)

const fetchReviews = () => {
  reviews.value = undefined
  return getTitleGroupReviews(props.titleGroupId).then((r) => (reviews.value = r))
}

const openReviews = () => {
  reviewsVisible.value = true
  return fetchReviews()
}

const openVoteForm = () => {
  voteFormVisible.value = true
  fetchReviews()
}

const updateReviews = (newReviews: TitleGroupReviewHierarchy[]) => {
  reviews.value = newReviews
  emit('reviewsChanged', newReviews)
}

const reviewSaved = (review: TitleGroupReviewHierarchy) => {
  const current = reviews.value ?? []
  updateReviews(current.some((r) => r.id === review.id) ? current.map((r) => (r.id === review.id ? review : r)) : [...current, review])
  voteFormVisible.value = false
}

const reviewDeleted = (reviewId: number) => updateReviews((reviews.value ?? []).filter((r) => r.id !== reviewId))

// notifications link to `#review-{id}`, the reviews only exist in the dialog
onMounted(() => {
  if (route.hash.startsWith('#review-')) openReviews().then(() => nextTick(() => document.querySelector(route.hash)?.scrollIntoView()))
})
</script>
<style scoped>
.ratings-component {
  display: flex;
  justify-content: space-around;
}
.rating {
  display: flex;
  align-items: center;
  gap: 10px;
}
.logo-and-note {
  display: flex;
  flex-direction: column;
  align-items: center;
  .logo {
    width: 3em;
    margin-bottom: 5px;
  }
}
.votes {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: 0.9em;
}
.vote-count {
  font-weight: bold;
}
.vote-link {
  font-style: italic;
}
</style>
<style>
.reviews-dialog {
  width: 70vw;
}
</style>
