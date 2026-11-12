<template>
  <ContentContainer :id="`rating-${rating.id}`" class="rating-card">
    <div class="header">
      <img class="avatar" :src="rating.created_by.avatar ?? '/default_user_avatar.png'" :alt="rating.created_by.username + '\'s avatar'" />
      <div class="author">
        <UsernameEnriched :user="rating.created_by" />
        <TimeAgo :date="rating.created_at" class="date" />
        <UserRatingDisplay :rating="rating.rating" class="rating" />
      </div>
      <i v-if="userStore.permissions.includes('delete_title_group_rating')" v-tooltip.top="t('general.delete')" class="pi pi-trash" @click="emit('delete')" />
    </div>
    <BBCodeRenderer v-if="rating.comment" :content="rating.comment" class="content" />
  </ContentContainer>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import BBCodeRenderer from '../community/BBCodeRenderer.vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'
import TimeAgo from '../TimeAgo.vue'
import ContentContainer from '../ContentContainer.vue'
import UserRatingDisplay from './UserRatingDisplay.vue'
import { useUserStore } from '@/stores/user'
import type { TitleGroupRatingHierarchy } from '@/services/api-schema'

defineProps<{
  rating: TitleGroupRatingHierarchy
}>()

const emit = defineEmits<{
  delete: []
}>()

const { t } = useI18n()
const userStore = useUserStore()
</script>

<style scoped>
.rating-card {
  min-width: 0;
}
.header {
  display: flex;
  align-items: center;
  gap: 12px;
}
.avatar {
  width: 5em;
  height: 5em;
  border-radius: 7px;
  object-fit: cover;
}
.author {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
  min-width: 0;
}
.date {
  font-size: 0.85em;
  opacity: 0.7;
}
.rating {
  font-weight: bold;
  font-size: 1.2em;
}
.pi-trash {
  align-self: flex-start;
  cursor: pointer;
}
.content {
  margin-top: 10px;
  overflow-wrap: anywhere;
}
</style>
