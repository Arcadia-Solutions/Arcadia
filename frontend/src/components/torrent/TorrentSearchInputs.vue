<template>
  <ContentContainer>
    <div id="torrent-search-inputs">
      <div class="line">
        <FloatLabel>
          <InputText class="title-group-name" size="small" v-model="searchForm.title_group_name" name="title_group_name" />
          <label for="title_group_name">{{ t('general.search_terms') }}</label>
        </FloatLabel>
      </div>
      <div class="line">
        <div class="tag-query-wrapper">
          <FloatLabel>
            <Textarea
              size="small"
              v-model="searchForm.title_group_tags"
              name="title_group_tags"
              :rows="2"
              class="tag-query"
              v-tooltip.top="t('general.supports_full_boolean_search')"
              @input="searchTags"
              @keydown="onTagQueryKeydown"
              @blur="foundTags = []"
            />
            <label for="title_group_tags">{{ t('general.tags') }}</label>
          </FloatLabel>
          <Listbox
            v-if="foundTags.length > 0"
            :modelValue="foundTags[highlightedTagIndex]"
            :options="foundTags"
            optionLabel="name"
            class="tag-suggestions"
            @update:modelValue="tagSuggestionSelected"
            @mousedown.prevent
          >
            <template #option="{ option }">
              <span v-tooltip.right="option.synonyms.length > 0 ? `${t('general.synonym', option.synonyms.length)}: ${option.synonyms.join(', ')}` : null">
                {{ option.name }}
              </span>
            </template>
          </Listbox>
        </div>
      </div>
      <div class="line">
        <FloatLabel>
          <InputNumber size="small" v-model="searchForm.torrent_snatched_by_id" name="snatched_by_user_id" />
          <label for="snatched_by_user_id">{{ t('torrent.snatched_by_user_id') }}</label>
        </FloatLabel>
        <FloatLabel>
          <InputNumber size="small" v-model="searchForm.torrent_created_by_id" name="uploaded_by_user_id" />
          <label for="uploaded_by_user_id">{{ t('torrent.uploaded_by_user_id') }}</label>
        </FloatLabel>
        <FloatLabel>
          <MultiSelect
            v-model="searchForm.title_group_content_type"
            :options="contentTypeOptions"
            optionLabel="label"
            optionValue="value"
            size="small"
            input-id="contentTypeSelect"
            display="chip"
            class="scrollable-chips"
          />
          <label for="contentTypeSelect">{{ t('title_group.content_type.content_type') }}</label>
        </FloatLabel>
        <FloatLabel>
          <MultiSelect
            v-model="searchForm.title_group_category"
            :options="getSelectableTitleGroupCategories()"
            size="small"
            input-id="categorySelect"
            display="chip"
            class="scrollable-chips"
          />
          <label for="categorySelect">{{ t('general.category') }}</label>
        </FloatLabel>
        <FloatLabel>
          <MultiSelect
            v-model="searchForm.edition_group_source"
            :options="getSelectableSources()"
            size="small"
            input-id="sourceSelect"
            display="chip"
            class="scrollable-chips"
          />
          <label for="sourceSelect">{{ t('edition_group.source') }}</label>
        </FloatLabel>
        <FloatLabel>
          <MultiSelect
            v-model="searchForm.torrent_video_resolution"
            :options="getSelectableVideoResolutions()"
            size="small"
            input-id="videoResolutionSelect"
            display="chip"
            class="scrollable-chips"
          />
          <label for="videoResolutionSelect">{{ t('torrent.video_resolution') }}</label>
        </FloatLabel>
        <FloatLabel>
          <MultiSelect
            v-model="searchForm.torrent_language"
            :options="getLanguages()"
            size="small"
            input-id="languageSelect"
            display="chip"
            filter
            class="scrollable-chips"
          />
          <label for="languageSelect">{{ t('general.language', 2) }}</label>
        </FloatLabel>
      </div>
      <div class="line">
        <FloatLabel>
          <Dropdown
            v-model="searchForm.order_by_column"
            :options="sortByOptions"
            optionLabel="label"
            optionValue="value"
            size="small"
            input-id="sortByDropdown"
          />
          <label for="sortByDropdown">{{ t('general.sort_by') }}</label>
        </FloatLabel>
        <FloatLabel>
          <Dropdown
            v-model="searchForm.order_by_direction"
            :options="getOrderByDirectionOptions(t)"
            optionLabel="label"
            optionValue="value"
            size="small"
            input-id="orderDropdown"
          />
          <label for="orderDropdown">{{ t('general.order_by') }}</label>
        </FloatLabel>
        <FloatLabel>
          <InputNumber size="small" v-model="searchForm.title_group_minimum_user_rating" inputId="minimumUserRating" :min="0" :max="100" suffix="%" />
          <label for="minimumUserRating">{{ t('title_group.minimum_user_rating') }}</label>
        </FloatLabel>
        <FloatLabel>
          <InputNumber size="small" v-model="searchForm.title_group_maximum_user_rating" inputId="maximumUserRating" :min="0" :max="100" suffix="%" />
          <label for="maximumUserRating">{{ t('title_group.maximum_user_rating') }}</label>
        </FloatLabel>
      </div>
      <div class="line">
        <FloatLabel>
          <Dropdown
            v-model="searchForm.torrent_staff_checked"
            :options="staffOptionChoices"
            optionLabel="label"
            optionValue="value"
            :placeholder="t('general.both')"
            size="small"
            style="width: 10em"
            class="p-inputwrapper-filled"
          />
          <label>{{ t('torrent.staff_checked') }}</label>
        </FloatLabel>
        <FloatLabel>
          <Dropdown
            v-model="searchForm.torrent_reported"
            :options="staffOptionChoices"
            optionLabel="label"
            optionValue="value"
            :placeholder="t('general.both')"
            size="small"
            class="p-inputwrapper-filled"
          />
          <label>{{ t('general.reported') }}</label>
        </FloatLabel>
        <FloatLabel>
          <Dropdown
            v-model="searchForm.torrent_trumpable"
            :options="staffOptionChoices"
            optionLabel="label"
            optionValue="value"
            :placeholder="t('general.both')"
            size="small"
            class="p-inputwrapper-filled"
          />
          <label>{{ t('torrent.trumpable') }}</label>
        </FloatLabel>
      </div>
      <div class="flex align-items-center gap-2" style="margin-top: 5px">
        <Checkbox
          v-model="searchForm.title_group_include_empty_groups"
          binary
          inputId="title_group_include_empty_groups"
          name="title_group_include_empty_groups"
        />
        <label for="title_group_include_empty_groups">{{ t('title_group.include_empty_title_groups') }}</label>
      </div>
      <div class="flex justify-content-center" style="margin-top: 15px" v-if="!displaySearchButton">
        <Button :loading :label="t('general.search')" @click="search" />
      </div>
      <div class="flex justify-content-left" v-if="props.totalResults !== undefined">
        {{ t('general.results', [props.totalResults], props.totalResults) }}
      </div>
    </div>
  </ContentContainer>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import ContentContainer from '../ContentContainer.vue'
import InputText from 'primevue/inputtext'
import FloatLabel from 'primevue/floatlabel'
import Button from 'primevue/button'
import { Checkbox, Dropdown, InputNumber, Listbox, MultiSelect, Textarea } from 'primevue'
import { useRouter } from 'vue-router'
import { searchTitleGroupTagsLite, TorrentSearchOrderByColumn, type TitleGroupTagLite, type TorrentSearch } from '@/services/api-schema'
import { getOrderByDirectionOptions, getSelectableContentTypes, getSelectableVideoResolutions, getLanguages } from '@/services/helpers'
import { Source, TitleGroupCategory } from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const { t } = useI18n()
const router = useRouter()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const contentTypeOptions = computed(() => getSelectableContentTypes().map((ct) => ({ label: t(`title_group.content_type.${ct}`), value: ct })))
const getSelectableSources = () => Object.values(Source)
const getSelectableTitleGroupCategories = () => Object.values(TitleGroupCategory)

const props = defineProps<{
  loading: boolean
  initialForm: TorrentSearch
  displaySearchButton?: boolean
  totalResults?: number
}>()

const staffOptionChoices = ref([
  { label: t('general.yes'), value: true },
  { label: t('general.no'), value: false },
  { label: t('general.both'), value: null },
])

const searchForm = ref<TorrentSearch>({
  title_group_name: '',
  title_group_tags: null,
  title_group_include_empty_groups: false,
  title_group_content_type: [],
  title_group_category: [],
  title_group_minimum_user_rating: 0,
  title_group_maximum_user_rating: 100,
  edition_group_source: [],
  torrent_created_by_id: null,
  torrent_snatched_by_id: null,
  torrent_staff_checked: false,
  torrent_reported: null,
  torrent_trumpable: null,
  torrent_language: [],
  torrent_video_resolution: [],
  page: 1,
  page_size: 4,
  order_by_column: 'torrent_created_at',
  order_by_direction: 'desc',
})
// the tag query supports boolean operators, so the autocompletion only completes the tag under the cursor
const tagSeparators = '&|!() \t\n'
const tagQueryTextarea = ref<HTMLTextAreaElement | null>(null)
const foundTags = ref<TitleGroupTagLite[]>([])
const highlightedTagIndex = ref(0)
let currentTagStart = 0
let currentTagEnd = 0

const searchTags = (event: Event) => {
  if (!(event.target instanceof HTMLTextAreaElement)) {
    return
  }
  tagQueryTextarea.value = event.target
  const query = event.target.value
  const cursorPosition = event.target.selectionStart

  currentTagStart = cursorPosition
  while (currentTagStart > 0 && !tagSeparators.includes(query[currentTagStart - 1])) {
    currentTagStart--
  }
  currentTagEnd = cursorPosition
  while (currentTagEnd < query.length && !tagSeparators.includes(query[currentTagEnd])) {
    currentTagEnd++
  }

  const tagName = query.slice(currentTagStart, currentTagEnd)
  if (tagName === '') {
    foundTags.value = []
    return
  }
  searchTitleGroupTagsLite({ name: tagName, page: 1, page_size: 10 }).then((tags) => {
    foundTags.value = tags.results
    highlightedTagIndex.value = 0
  })
}

// the listbox deselects (emits null) when the already highlighted suggestion is clicked
const tagSuggestionSelected = (tag: TitleGroupTagLite | null) => {
  selectTag(tag ?? foundTags.value[highlightedTagIndex.value])
}

const selectTag = (tag: TitleGroupTagLite) => {
  const query = searchForm.value.title_group_tags ?? ''
  const suffix = query.slice(currentTagEnd)
  searchForm.value.title_group_tags = query.slice(0, currentTagStart) + tag.name + (suffix.startsWith(' ') ? '' : ' ') + suffix
  foundTags.value = []
  // place the cursor after the space following the completed tag
  const cursorPosition = currentTagStart + tag.name.length + 1
  nextTick(() => {
    tagQueryTextarea.value?.focus()
    tagQueryTextarea.value?.setSelectionRange(cursorPosition, cursorPosition)
  })
}

const onTagQueryKeydown = (event: KeyboardEvent) => {
  if (foundTags.value.length === 0) {
    return
  }
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    highlightedTagIndex.value = (highlightedTagIndex.value + 1) % foundTags.value.length
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    highlightedTagIndex.value = (highlightedTagIndex.value - 1 + foundTags.value.length) % foundTags.value.length
  } else if (event.key === 'Enter') {
    event.preventDefault()
    selectTag(foundTags.value[highlightedTagIndex.value])
  } else if (event.key === 'Escape') {
    foundTags.value = []
  }
}

const sortByOptions = computed(() => {
  const options: { label: string; value: TorrentSearchOrderByColumn }[] = [
    { label: t('torrent.created_at'), value: TorrentSearchOrderByColumn.TorrentCreatedAt },
    { label: t('torrent.size'), value: TorrentSearchOrderByColumn.TorrentSize },
    { label: t('title_group.original_release_date'), value: TorrentSearchOrderByColumn.TitleGroupOriginalReleaseDate },
    { label: t('torrent.snatched'), value: TorrentSearchOrderByColumn.TorrentSnatched },
    { label: t('torrent.seeders'), value: TorrentSearchOrderByColumn.TorrentSeeders },
    { label: t('torrent.leecher', 2), value: TorrentSearchOrderByColumn.TorrentLeechers },
    { label: t('title_group.user_rating'), value: TorrentSearchOrderByColumn.TitleGroupUserRatingAverage },
    {
      label: `${publicArcadiaSettings.bonus_points_alias} ${t('torrent.snatch_cost')}`,
      value: TorrentSearchOrderByColumn.TorrentBonusPointsSnatchCost,
    },
  ]
  if (searchForm.value.torrent_snatched_by_id != null) {
    options.push({ label: t('torrent.snatched_at'), value: TorrentSearchOrderByColumn.TorrentSnatchedAt })
  }
  return options
})

const changePage = (page: number) => {
  searchForm.value.page = page
  search()
}
const search = () => {
  router.push({
    query: Object.fromEntries(Object.entries(searchForm.value).map(([k, v]) => [k, typeof v === 'boolean' ? String(v) : v])),
  })
  // a search will be triggered by the query changes through a watcher
}
defineExpose({
  searchForm,
  changePage,
})

watch(
  () => props.initialForm,
  (newForm) => {
    searchForm.value = newForm
  },
  { immediate: true },
)

watch(
  () => searchForm.value,
  (newVal, oldVal) => {
    // ignore if only `page` changed
    if (newVal.page === oldVal.page) {
      searchForm.value.page = 1
    }
  },
  { deep: true },
)

watch(
  () => searchForm.value.torrent_snatched_by_id,
  (newVal) => {
    if (newVal == null && searchForm.value.order_by_column === TorrentSearchOrderByColumn.TorrentSnatchedAt) {
      searchForm.value.order_by_column = TorrentSearchOrderByColumn.TorrentCreatedAt
    }
  },
  { immediate: true },
)
</script>

<style>
.title-group-name {
  width: 40em;
}
.tag-query {
  width: 30em;
}
.tag-query-wrapper {
  position: relative;
}
.tag-suggestions {
  position: absolute;
  z-index: 1000;
}
.dropdown {
  display: flex;
  align-items: center;
  margin-right: 10px;
  label {
    margin-right: 5px;
  }
}
.staff-options {
  display: flex;
}
.scrollable-chips {
  min-width: 12em;
  max-width: 20em;
}
.scrollable-chips .p-multiselect-label {
  display: flex;
  flex-wrap: nowrap;
  overflow-x: auto;
  gap: 0.25rem;
}
</style>
