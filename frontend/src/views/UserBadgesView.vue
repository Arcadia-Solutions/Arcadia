<template>
  <div class="user-badges">
    <I18nT v-if="user" keypath="user_badge.badges_of" tag="div" class="title">
      <template #username>
        <UsernameEnriched :user />
      </template>
    </I18nT>
    <div v-for="category in categoriesWithBadges" :key="category.id">
      <div class="category-name">
        {{ category.name }}
        <i
          v-if="canEditBadges"
          v-tooltip.top="t('user_badge.reorder_badges')"
          class="pi pi-pen-to-square cursor-pointer"
          @click="categoryBeingReordered = category"
        />
      </div>
      <div class="badges-grid">
        <ContentContainer v-for="badge in category.badges" :key="badge.id" :containerTitle="badge.name" :class="['badge', { 'not-earned': !badge.awarded_at }]">
          <img :src="badge.image_url" :alt="badge.name" />
          <div class="badge-info">
            <div class="badge-description">{{ badge.description }}</div>
            <div class="badge-earned">
              <template v-if="badge.awarded_at">{{ t('user_badge.earned') }} <TimeAgo :date="badge.awarded_at" /></template>
              <template v-else>{{ t('user_badge.not_earned_yet') }}</template>
            </div>
          </div>
        </ContentContainer>
      </div>
    </div>
    <div v-if="!loading && !categoriesWithBadges.length" class="empty">{{ t('user_badge.no_badges') }}</div>

    <Dialog
      closeOnEscape
      modal
      :header="t('user_badge.reorder_badges_of', { category: categoryBeingReordered?.name })"
      :visible="categoryBeingReordered !== null"
      @update:visible="categoryBeingReordered = null"
    >
      <ReorderUserBadgesDialog
        v-if="categoryBeingReordered"
        :initialBadges="badges.filter(isFullBadge).filter((b) => b.category_id === categoryBeingReordered?.id)"
        @reordered="onReordered"
      />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { I18nT, useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { Dialog } from 'primevue'
import ContentContainer from '@/components/ContentContainer.vue'
import TimeAgo from '@/components/TimeAgo.vue'
import ReorderUserBadgesDialog from '@/components/user/ReorderUserBadgesDialog.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import { useUserStore } from '@/stores/user'
import {
  listUserBadgeCategories,
  listUserBadges,
  listUserEarnedBadges,
  type UserBadge,
  type UserBadgeCategory,
  type UserBadgeListItem,
  type UserLite,
  type UserEarnedBadgeWithDetails,
} from '@/services/api-schema'

type DisplayedBadge = Pick<UserBadge, 'id' | 'name' | 'description' | 'image_url' | 'category_id'> & { awarded_at?: string }

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()

const badges = ref<UserBadgeListItem[]>([])
const earnedBadges = ref<UserEarnedBadgeWithDetails[]>([])
const categories = ref<UserBadgeCategory[]>([])
const user = ref<UserLite>()
const loading = ref(true)
const categoryBeingReordered = ref<UserBadgeCategory | null>(null)

const canEditBadges = computed(() => userStore.permissions.includes('edit_user_badge'))

const isFullBadge = (b: UserBadgeListItem): b is UserBadge => 'name' in b

// every public badge, plus the secret ones this user already earned, in the order of the badge list
// (which also lists the secret badges the viewer cannot see, with their id only)
const displayedBadges = computed<DisplayedBadge[]>(() =>
  badges.value.flatMap((badge): DisplayedBadge[] => {
    const earned = earnedBadges.value.find((e) => e.badge_id === badge.id)
    if (earned) {
      return [
        {
          id: earned.badge_id,
          name: earned.badge_name,
          description: earned.badge_description,
          image_url: earned.badge_image_url,
          category_id: earned.badge_category_id,
          awarded_at: earned.awarded_at,
        },
      ]
    }
    return isFullBadge(badge) && !badge.is_secret ? [badge] : []
  }),
)

const categoriesWithBadges = computed(() =>
  categories.value
    .map((category) => ({ ...category, badges: displayedBadges.value.filter((b) => b.category_id === category.id) }))
    .filter((category) => category.badges.length),
)

const fetchData = () => {
  loading.value = true
  const userId = parseInt(route.params.id.toString())
  Promise.all([listUserBadges(), listUserBadgeCategories(), listUserEarnedBadges(userId)])
    .then(([badgeList, categoryList, userEarnedBadges]) => {
      badges.value = badgeList
      categories.value = categoryList
      earnedBadges.value = userEarnedBadges.earned_badges
      user.value = userEarnedBadges.user
    })
    .finally(() => {
      loading.value = false
    })
}

const onReordered = () => {
  categoryBeingReordered.value = null
  fetchData()
}

watch(() => route.params.id, fetchData, { immediate: true })
</script>

<style scoped>
.user-badges {
  display: flex;
  flex-direction: column;
  gap: 15px;
}
.category-name {
  font-weight: bold;
  font-size: 1.2em;
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.badges-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 10px;
}
.badge :deep(.content-body) {
  display: flex;
  gap: 12px;
}
.badge img {
  width: 64px;
  height: 64px;
  object-fit: contain;
  border-radius: 8px;
}
.badge-description,
.badge-earned {
  font-size: 0.9em;
}
.badge-earned {
  margin-top: 4px;
  opacity: 0.7;
}
.not-earned {
  filter: grayscale(1);
  opacity: 0.5;
}
.empty {
  opacity: 0.7;
  font-style: italic;
}
</style>
