<template>
  <LazyAutoComplete
    v-model="name"
    :suggestions="foundArtists"
    @complete="search"
    size="small"
    :placeholder
    optionLabel="name"
    @option-select="artistSelected($event.value)"
    @input="onInput"
    @keydown.enter="onEnter"
  >
    <template #option="slotProps">
      <RouterLink
        v-if="clickableSeriesLink"
        :to="`/artist/${slotProps.option.id}`"
        style="width: 100%"
        v-tooltip.right="
          slotProps.option.aliases?.length > 0 ? `${t('general.alias', slotProps.option.aliases.length)}: ${slotProps.option.aliases.join(', ')}` : null
        "
      >
        {{ slotProps.option.name }}
      </RouterLink>
      <div
        v-else
        v-tooltip.right="
          slotProps.option.aliases?.length > 0 ? `${t('general.alias', slotProps.option.aliases.length)}: ${slotProps.option.aliases.join(', ')}` : null
        "
      >
        {{ slotProps.option.name }}
      </div>
    </template>
  </LazyAutoComplete>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue'
import LazyAutoComplete from '@/components/LazyAutoComplete.vue'
import { searchArtistsLite, type ArtistLite } from '@/services/api-schema'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  placeholder: string
  clearInputOnSelect: boolean
  modelValue: string
  clickableSeriesLink?: boolean
  enterRedirectsToArtistPage?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [string]
  artistSelected: [ArtistLite]
}>()

const { t } = useI18n()
const router = useRouter()

const name = ref('')

watch(
  () => props.modelValue,
  (newValue) => {
    name.value = newValue
  },
  { immediate: true },
)

const foundArtists = ref<ArtistLite[]>([])

const artistSelected = (artist: ArtistLite) => {
  const selectedArtistName = artist.name
  emit('artistSelected', artist)
  if (props.clearInputOnSelect) {
    name.value = ''
    emit('update:modelValue', '')
  } else {
    name.value = selectedArtistName
    emit('update:modelValue', selectedArtistName)
  }
}

const onInput = () => {
  emit('update:modelValue', name.value)
}

const onEnter = () => {
  if (props.enterRedirectsToArtistPage) {
    if (foundArtists.value.length > 0) {
      router.push(`/artist/${foundArtists.value[0].id}`)
      artistSelected(foundArtists.value[0])
    }
  }
}

const search = () => {
  if (name.value !== '') {
    searchArtistsLite(name.value).then((artists) => {
      foundArtists.value = artists
    })
  } else {
    foundArtists.value = []
  }
}
</script>
