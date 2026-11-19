<template>
  <ContentContainer :containerTitle="t('title_group.rating', 2)">
    <template v-if="userStore.permissions.includes('use_maintenance_tools') && ratableLinks.length > 0" #top-right>
      <i v-tooltip.top="t('title_group.refetch_public_ratings')" class="pi pi-refresh cursor-pointer" @click="openRefetchDialog" />
    </template>
    <div class="ratings-component">
      <div class="rating" v-for="rating in publicRatingsWithWebsite" :key="rating.external_link">
        <a class="logo-and-note" :href="rating.external_link" target="_blank" v-tooltip.top="rating.website.name">
          <img v-if="rating.website.icon" :src="rating.website.icon" :alt="rating.website.name" class="logo" />
          <span v-else>{{ rating.website.name }}</span>
        </a>
        <div class="votes">
          <span class="rating-value">{{ rating.rating.toLocaleString(undefined, { maximumFractionDigits: 1 }) }} {{ rating.website.rating_scale }}</span>
          <span class="vote-count">({{ formatNumber(rating.votes) }} {{ t('title_group.vote', rating.votes) }})</span>
        </div>
      </div>
      <div class="rating">
        <div class="logo-and-note" :class="{ 'cursor-pointer': userRatingCount > 0 }" @click="userRatingCount > 0 && openRatings()">
          <img :src="averageIcon?.icon_url ?? '/favicon.ico'" alt="rating icon" class="logo" />
        </div>
        <div class="votes">
          <UserRatingDisplay :rating="userRatingAverage" hideIcon class="rating-value" />
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
    <Dialog closeOnEscape modal :header="t('title_group.refetch_public_ratings')" v-model:visible="refetchDialogVisible">
      <div class="refetch-links">
        <div v-for="link in ratableLinks" :key="link">
          <Checkbox v-model="linksToRefetch" :value="link" :inputId="link" />
          <label :for="link">{{ publicRatingWebsite(link).name }}: {{ link }}</label>
          <Message v-if="link in refetchErrors" :severity="refetchErrors[link] === null ? 'success' : 'error'" size="small" variant="simple">
            {{ refetchErrors[link] === null ? t('title_group.public_rating_refetched') : t('title_group.public_rating_refetch_failed', [refetchErrors[link]]) }}
          </Message>
        </div>
      </div>
      <div class="wrapper-center">
        <Button :label="t('title_group.refetch')" :loading="refetching" :disabled="linksToRefetch.length === 0" @click="refetch" />
      </div>
    </Dialog>
    <Dialog closeOnEscape modal :header="t('title_group.your_rating')" v-model:visible="voteFormVisible">
      <TitleGroupVoteForm :titleGroupId :ownRating @saved="ownRatingSaved" />
    </Dialog>
  </ContentContainer>
</template>
<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { Button, Checkbox, Dialog, Message } from 'primevue'
import ProgressSpinner from 'primevue/progressspinner'
import {
  OrderByDirection,
  refetchTitleGroupPublicRatings,
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
import { isRatableLink, publicRatingWebsite } from '@/config'
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
  externalLinks: string[]
}>()

const averageIcon = computed(() => getRatingIcon(publicArcadiaSettings.rating_icons, props.userRatingAverage))

const ratableLinks = computed(() => props.externalLinks.filter(isRatableLink))

const publicRatingsWithWebsite = computed(() => props.publicRatings.map((rating) => ({ ...rating, website: publicRatingWebsite(rating.external_link) })))

const emit = defineEmits<{
  ownRatingSaved: [TitleGroupRating]
  ratingDeleted: [TitleGroupRatingHierarchy]
  publicRatingsRefetched: [PublicRating[]]
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
const refetchDialogVisible = ref(false)
const linksToRefetch = ref<string[]>([])
const refetching = ref(false)
// the outcome of the last refetch of each link: the error of its provider, or null when it succeeded
const refetchErrors = ref<Record<string, string | null>>({})

const openRefetchDialog = () => {
  linksToRefetch.value = [...ratableLinks.value]
  refetchErrors.value = {}
  refetchDialogVisible.value = true
}

const refetch = () => {
  refetching.value = true
  refetchTitleGroupPublicRatings({ title_group_id: props.titleGroupId, external_links: linksToRefetch.value })
    .then(({ public_ratings, failures }) => {
      emit('publicRatingsRefetched', public_ratings)
      if (failures.length === 0) {
        refetchDialogVisible.value = false
        return
      }
      refetchErrors.value = Object.fromEntries(linksToRefetch.value.map((link) => [link, null]))
      failures.forEach((failure) => (refetchErrors.value[failure.external_link] = failure.error))
      // only the failed links are left selected, to retry them
      linksToRefetch.value = failures.map((failure) => failure.external_link)
    })
    .finally(() => (refetching.value = false))
}

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
.vote-count,
.rating-value {
  font-weight: bold;
}
.refetch-links {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 15px;
  label {
    margin-left: 7px;
  }
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
