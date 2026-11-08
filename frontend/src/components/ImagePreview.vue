<template>
  <Image
    :src="imageLink"
    :referrerpolicy="referrerPolicy(imageLink)"
    :pt="{ original: { referrerpolicy: referrerPolicy(imageLink) } }"
    :alt
    preview
    loading="lazy"
  >
    <template #previewicon>
      <i class="pi pi-search"></i>
    </template>
  </Image>
</template>
<script setup lang="ts">
import { Image } from 'primevue'
import { referrerPolicy } from '@/services/helpers'

defineProps<{
  imageLink: string
  alt?: string
}>()
</script>
