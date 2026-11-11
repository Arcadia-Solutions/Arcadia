<template>
  <div>
    <div class="page-title">{{ t('community.title_group_reviews') }}</div>
    <PaginatedResults :totalItems :pageSize :initialPage="page" :totalPages @changePage="changePage($event.page)">
      <ContentContainer v-for="review in reviews" :key="review.id" class="review">
        <div class="review-header">
          <RouterLink :to="`/title-group/${review.title_group_id}`">{{ review.title_group_name }}</RouterLink>
          <UserRatingDisplay :rating="review.rating" />
          <TimeAgo :date="review.created_at" />
        </div>
        <BBCodeRenderer v-if="review.review" :content="review.review" />
      </ContentContainer>
    </PaginatedResults>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import PaginatedResults from '@/components/PaginatedResults.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import TimeAgo from '@/components/TimeAgo.vue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import UserRatingDisplay from '@/components/title_group/UserRatingDisplay.vue'
import { searchUserTitleGroupReviews, type TitleGroupReviewWithTitleGroup } from '@/services/api-schema'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const pageSize = 10
const page = ref(1)
const reviews = ref<TitleGroupReviewWithTitleGroup[]>([])
const totalItems = ref(0)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize))

const fetchReviewsFromUrl = () => {
  page.value = route.query.page ? parseInt(route.query.page.toString()) : 1
  searchUserTitleGroupReviews({
    user_id: parseInt(route.query.created_by_id?.toString() ?? ''),
    page: page.value,
    page_size: pageSize,
  }).then((paginatedReviews) => {
    reviews.value = paginatedReviews.results
    totalItems.value = paginatedReviews.total_items
  })
}

const changePage = (newPage: number) => {
  router.push({ query: { ...route.query, page: newPage.toString() } })
}

onMounted(fetchReviewsFromUrl)
watch(() => route.query, fetchReviewsFromUrl, { deep: true })
</script>

<style scoped>
.page-title {
  font-size: 1.2em;
  margin-bottom: 10px;
}
.review {
  margin-bottom: 15px;
}
.review-header {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 5px;
}
</style>
