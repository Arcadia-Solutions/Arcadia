<template>
  <Tabs :value="0" size="small">
    <TabList>
      <Tab v-for="(link, i) in links" :key="link" :value="i">{{ `Trailer ${i + 1}` }}</Tab>
    </TabList>
    <TabPanels>
      <TabPanel v-for="(link, i) in links" :key="link" :value="i">
        <iframe
          v-if="isYoutubeLink(link)"
          style="width: 100%; aspect-ratio: 16 / 9; border: 0"
          :src="link"
          allowfullscreen
          referrerpolicy="strict-origin-when-cross-origin"
        />
        <!-- only youtube links are embedded, anything else must be a direct video link -->
        <video v-else-if="isHttpLink(link)" style="width: 100%; aspect-ratio: 16 / 9" :src="link" controls preload="metadata" />
      </TabPanel>
    </TabPanels>
  </Tabs>
</template>

<script setup lang="ts">
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import { isHttpLink, isYoutubeLink } from '@/services/helpers'

defineProps<{
  links: string[]
}>()
</script>
