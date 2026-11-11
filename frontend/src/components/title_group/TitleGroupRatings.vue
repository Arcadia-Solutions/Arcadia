<template>
  <ContentContainer :containerTitle="t('title_group.rating', 2)">
    <div class="ratings-component">
      <div class="rating">
        <div class="logo-and-note" :class="{ 'cursor-pointer': userRatingCount > 0 }" @click="userRatingCount > 0 && openReviews()">
          <img :src="averageIcon?.icon_url ?? '/favicon.ico'" alt="rating icon" class="logo" />
          <UserRatingDisplay :rating="userRatingAverage" hideIcon />
        </div>
        <div class="votes">
          <a v-if="userRatingCount > 0" class="vote-count cursor-pointer" @click="openReviews">
            ({{ formatNumber(userRatingCount) }} {{ t('title_group.vote', userRatingCount) }})
          </a>
          <i
            v-if="userStore.permissions.includes('rate_title_groups')"
            v-tooltip.top="t(ownReview === null ? 'title_group.add_your_rating' : 'title_group.edit_your_rating')"
            :class="['vote-link', 'cursor-pointer', 'pi', ownReview === null ? 'pi-plus' : 'pi-pen-to-square']"
            @click="openVoteForm"
          />
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
      <TitleGroupReviewFilters v-model="filters" :loading @search="fetchReviews(1)" />
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
import { Dialog } from 'primevue'
import ProgressSpinner from 'primevue/progressspinner'
import {
  OrderByDirection,
  searchTitleGroupReviews,
  TitleGroupReviewSearchOrderByColumn,
  type PublicRating,
  type TitleGroupReview,
  type TitleGroupReviewHierarchy,
} from '@/services/api-schema'
import ContentContainer from '../ContentContainer.vue'
import PaginatedResults from '../PaginatedResults.vue'
import { useI18n } from 'vue-i18n'
import { formatNumber, getRatingIcon } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { useUserStore } from '@/stores/user'
import UserRatingDisplay from './UserRatingDisplay.vue'
import TitleGroupReviews from './TitleGroupReviews.vue'
import TitleGroupVoteForm from './TitleGroupVoteForm.vue'
import TitleGroupReviewFilters, { type TitleGroupReviewFilters as Filters } from './TitleGroupReviewFilters.vue'

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const props = defineProps<{
  titleGroupId: number
  publicRatings: PublicRating[]
  userRatingAverage: number | null
  userRatingCount: number
  ownReview: TitleGroupReview | null
}>()

const averageIcon = computed(() => getRatingIcon(publicArcadiaSettings.rating_icons, props.userRatingAverage))

const emit = defineEmits<{
  ownReviewSaved: [TitleGroupReview]
  reviewDeleted: [TitleGroupReviewHierarchy]
}>()

const pageSize = 12
const page = ref(1)
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))
const filters = ref<Filters>({
  minimum_rating: null,
  maximum_rating: null,
  content: '',
  order_by_column: TitleGroupReviewSearchOrderByColumn.CreatedAt,
  order_by_direction: OrderByDirection.Desc,
})
const reviews = ref<TitleGroupReviewHierarchy[]>()
const loading = ref(false)
const reviewsVisible = ref(false)
const voteFormVisible = ref(false)

const fetchReviews = (newPage: number) => {
  page.value = newPage
  loading.value = true
  return searchTitleGroupReviews({
    title_group_id: props.titleGroupId,
    ...filters.value,
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

// notifications link to `#review-{id}` of a new review, which is on the first page of the default sort
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
  margin-top: 5px;
  font-size: 0.8em;
}
a.vote-count,
.vote-link {
  transition: color 0.2s;
}
a.vote-count:hover,
.vote-link:hover {
  color: var(--color-primary);
}
</style>
<style>
.reviews-dialog {
  width: 70vw;
}
</style>
