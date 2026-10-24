<template>
  <ContentContainer :containerTitle="announcement.forum_thread_name" :containerTitleLink="`/forum/thread/${announcement.forum_thread_id}`">
    <template #top-right>(<TimeAgo :date="announcement.created_at" />)</template>
    <BBCodeRenderer :content="announcement.content" />
  </ContentContainer>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import ContentContainer from '../ContentContainer.vue'
import BBCodeRenderer from '../community/BBCodeRenderer.vue'
import type { ForumPostAndThreadName } from '@/services/api-schema'

defineProps<{
  announcement: ForumPostAndThreadName
}>()
</script>

<style scoped></style>
