<template>
  <span v-tooltip.top="formatDate(date)">{{ timeAgo(date) }}</span>
</template>

<script setup lang="ts">
import { formatDate, timeAgo } from '@/services/helpers'

defineProps<{
  date: string
}>()
</script>
