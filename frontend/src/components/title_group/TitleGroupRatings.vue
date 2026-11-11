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
            {{ t(ownReview === null ? 'title_group.add_your_vote' : 'title_group.edit_your_rating') }}
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
      <ContentContainer class="review-filters-container">
        <div class="review-filters">
          <label>
            {{ t('title_group.minimum_rating') }}
            <InputNumber v-model="minimumRating" :min="0" :max="100" showButtons :inputStyle="{ width: '4em' }" size="small" />
          </label>
          <label>
            {{ t('title_group.maximum_rating') }}
            <InputNumber v-model="maximumRating" :min="0" :max="100" showButtons :inputStyle="{ width: '4em' }" size="small" />
          </label>
          <InputText v-model="content" :placeholder="t('title_group.search_rating_text')" size="small" class="content-search" @keyup.enter="fetchReviews(1)" />
          <Button :label="t('general.search')" size="small" :loading @click="fetchReviews(1)" />
        </div>
      </ContentContainer>
      <PaginatedResults v-if="reviews" :totalItems :pageSize :initialPage="page" :totalPages @changePage="fetchReviews($event.page)">
        <TitleGroupReviews :reviews @deleted="reviewDeleted" />
      </PaginatedResults>
      <ProgressSpinner v-else />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.your_rating')" v-model:visible="voteFormVisible">
      <TitleGroupVoteForm :titleGroupId :ownReview @saved="ownReviewSaved" />
    </Dialog>
  </ContentContainer>
</template>
<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { Button, Dialog, InputNumber, InputText } from 'primevue'
import ProgressSpinner from 'primevue/progressspinner'
import { searchTitleGroupReviews, type PublicRating, type TitleGroupReview, type TitleGroupReviewHierarchy } from '@/services/api-schema'
import ContentContainer from '../ContentContainer.vue'
import PaginatedResults from '../PaginatedResults.vue'
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
  ownReview: TitleGroupReview | null
}>()

const emit = defineEmits<{
  ownReviewSaved: [TitleGroupReview]
  reviewDeleted: [TitleGroupReviewHierarchy]
}>()

const pageSize = 12
const page = ref(1)
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))
const minimumRating = ref<number | null>(null)
const maximumRating = ref<number | null>(null)
const content = ref('')
const reviews = ref<TitleGroupReviewHierarchy[]>()
const loading = ref(false)
const reviewsVisible = ref(false)
const voteFormVisible = ref(false)

const fetchReviews = (newPage: number) => {
  page.value = newPage
  loading.value = true
  return searchTitleGroupReviews({
    title_group_id: props.titleGroupId,
    minimum_rating: minimumRating.value,
    maximum_rating: maximumRating.value,
    content: content.value,
    page: newPage,
    page_size: pageSize,
  })
    .then((paginatedReviews) => {
      reviews.value = paginatedReviews.results
      totalItems.value = paginatedReviews.total_items
    })
    .finally(() => (loading.value = false))
}

const openReviews = () => {
  reviewsVisible.value = true
  return fetchReviews(1)
}

const openVoteForm = () => (voteFormVisible.value = true)

const ownReviewSaved = (review: TitleGroupReview) => {
  emit('ownReviewSaved', review)
  voteFormVisible.value = false
}

const reviewDeleted = (review: TitleGroupReviewHierarchy) => {
  emit('reviewDeleted', review)
  fetchReviews(page.value)
}

// notifications link to `#review-{id}`, the reviews only exist in the dialog, so it is opened on the
// unfiltered first page, which may not contain the review
onMounted(() => {
  if (route.hash.startsWith('#review-')) openReviews().then(() => nextTick(() => document.querySelector(route.hash)?.scrollIntoView()))
})
</script>
<style scoped>
.content-search {
  flex: 1;
  min-width: 15em;
}
.review-filters-container {
  margin-bottom: 10px;
}
.review-filters {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 15px;
  label {
    display: flex;
    align-items: center;
    gap: 5px;
  }
}
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
