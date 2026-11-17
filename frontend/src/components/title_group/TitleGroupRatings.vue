<template>
  <ContentContainer :containerTitle="t('title_group.rating', 2)">
    <div class="ratings-component">
      <div class="rating" v-for="rating in publicRatingsWithWebsite" :key="rating.external_link">
        <a class="logo-and-note" :href="rating.external_link" target="_blank" v-tooltip.top="rating.website.name">
          <img v-if="rating.website.icon" :src="rating.website.icon" :alt="rating.website.name" class="logo" />
          <span v-else>{{ rating.website.name }}</span>
          <span>{{ rating.rating.toLocaleString(undefined, { maximumFractionDigits: 1 }) }}</span>
        </a>
        <span class="votes vote-count">({{ formatNumber(rating.votes) }} {{ t('title_group.vote', rating.votes) }})</span>
      </div>
      <div class="rating">
        <div class="logo-and-note" :class="{ 'cursor-pointer': userRatingCount > 0 }" @click="userRatingCount > 0 && openRatings()">
          <img :src="averageIcon?.icon_url ?? '/favicon.ico'" alt="rating icon" class="logo" />
          <UserRatingDisplay :rating="userRatingAverage" hideIcon />
        </div>
        <div class="votes">
          <a v-if="userRatingCount > 0" class="vote-count cursor-pointer" @click="openRatings">
            ({{ formatNumber(userRatingCount) }} {{ t('title_group.vote', userRatingCount) }})
          </a>
          <i
            v-if="userStore.permissions.includes('rate_title_groups')"
            v-tooltip.top="t(ownRating === null ? 'title_group.add_your_rating' : 'title_group.edit_your_rating')"
            :class="['vote-link', 'cursor-pointer', 'pi', ownRating === null ? 'pi-plus' : 'pi-pen-to-square']"
            @click="openVoteForm"
          />
        </div>
      </div>
    </div>
    <Dialog closeOnEscape modal :header="t('title_group.rating', 2)" v-model:visible="ratingsVisible" class="ratings-dialog">
      <TitleGroupRatingFilters v-model="filters" :loading @search="fetchRatings(1)" />
      <PaginatedResults v-if="ratings" :totalItems :pageSize :initialPage="page" :totalPages @changePage="fetchRatings($event.page)">
        <TitleGroupRatingList :ratings @deleted="ratingDeleted" />
      </PaginatedResults>
      <ProgressSpinner v-else />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.your_rating')" v-model:visible="voteFormVisible">
      <TitleGroupVoteForm :titleGroupId :ownRating @saved="ownRatingSaved" />
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
  searchTitleGroupRatings,
  TitleGroupRatingSearchOrderByColumn,
  type PublicRating,
  type TitleGroupRating,
  type TitleGroupRatingHierarchy,
} from '@/services/api-schema'
import ContentContainer from '../ContentContainer.vue'
import PaginatedResults from '../PaginatedResults.vue'
import { useI18n } from 'vue-i18n'
import { formatNumber, getRatingIcon } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { publicRatingWebsites, type PublicRatingWebsite } from '@/config'
import { useUserStore } from '@/stores/user'
import UserRatingDisplay from './UserRatingDisplay.vue'
import TitleGroupRatingList from './TitleGroupRatingList.vue'
import TitleGroupVoteForm from './TitleGroupVoteForm.vue'
import TitleGroupRatingFilters, { type TitleGroupRatingFilters as Filters } from './TitleGroupRatingFilters.vue'

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const props = defineProps<{
  titleGroupId: number
  publicRatings: PublicRating[]
  userRatingAverage: number | null
  userRatingCount: number
  ownRating: TitleGroupRating | null
}>()

const averageIcon = computed(() => getRatingIcon(publicArcadiaSettings.rating_icons, props.userRatingAverage))

// TMDB ratings are collected by arcadia itself, the other websites come from config.yml
const ratingWebsites: PublicRatingWebsite[] = [{ name: 'TMDB', domain: 'themoviedb.org', icon: '/logos/external_links/tmdb.svg' }, ...publicRatingWebsites]

// a link matching no website (e.g. its domain was removed from config.yml) is displayed with its host name
const ratingWebsite = (link: string): Pick<PublicRatingWebsite, 'name'> & Partial<PublicRatingWebsite> => {
  const host = new URL(link).hostname
  return ratingWebsites.find(({ domain }) => host === domain || host.endsWith(`.${domain}`)) ?? { name: host }
}

const publicRatingsWithWebsite = computed(() => props.publicRatings.map((rating) => ({ ...rating, website: ratingWebsite(rating.external_link) })))

const emit = defineEmits<{
  ownRatingSaved: [TitleGroupRating]
  ratingDeleted: [TitleGroupRatingHierarchy]
}>()

const pageSize = 25
const page = ref(1)
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))
const filters = ref<Filters>({
  minimum_rating: 0,
  maximum_rating: 100,
  content: '',
  order_by_column: TitleGroupRatingSearchOrderByColumn.CreatedAt,
  order_by_direction: OrderByDirection.Desc,
})
const ratings = ref<TitleGroupRatingHierarchy[]>()
const loading = ref(false)
const ratingsVisible = ref(false)
const voteFormVisible = ref(false)

const fetchRatings = (newPage: number) => {
  page.value = newPage
  loading.value = true
  return searchTitleGroupRatings({
    title_group_id: props.titleGroupId,
    ...filters.value,
    page: newPage,
    page_size: pageSize,
  })
    .then((paginatedRatings) => {
      ratings.value = paginatedRatings.results
      totalItems.value = paginatedRatings.total_items
    })
    .finally(() => (loading.value = false))
}

const openRatings = () => {
  ratingsVisible.value = true
  return fetchRatings(1)
}

const openVoteForm = () => (voteFormVisible.value = true)

const ownRatingSaved = (rating: TitleGroupRating) => {
  emit('ownRatingSaved', rating)
  voteFormVisible.value = false
}

const ratingDeleted = (rating: TitleGroupRatingHierarchy) => {
  emit('ratingDeleted', rating)
  fetchRatings(page.value)
}

// notifications link to `#rating-{id}` of a new rating, which is on the first page of the default sort
onMounted(() => {
  if (route.hash.startsWith('#rating-')) openRatings().then(() => nextTick(() => document.querySelector(route.hash)?.scrollIntoView()))
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
.ratings-dialog {
  width: 70vw;
}
</style>
