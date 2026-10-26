<template>
  <Form
    class="form"
    v-slot="$form"
    :initialValues="titleGroupForm"
    :resolver
    @submit="sendTitleGroup"
    @keydown.enter="onFormKeydown"
    validateOnSubmit
    validateOnValueUpdate
    validateOnBlur
    validateOnMount
    ref="formRef"
  >
    <FloatLabel style="margin-top: 0">
      <Select
        v-model="titleGroupForm.content_type"
        @update:model-value="(content_type) => (titleGroupStore.content_type = content_type)"
        inputId="content_type"
        name="content_type"
        :options="getSelectableContentTypes()"
        class="select"
        size="small"
      >
        <template #option="slotProps">
          <span>{{ t(`title_group.content_type.${slotProps.option}`) }}</span>
        </template>
        <template #value="slotProps">
          <span v-if="slotProps.value">
            {{ t(`title_group.content_type.${slotProps.value}`) }}
          </span>
        </template>
      </Select>
      <label for="content_type">{{ t('title_group.content_type.content_type') }}</label>
    </FloatLabel>
    <Message v-if="$form.content_type?.invalid" severity="error" size="small" variant="simple">
      {{ $form.content_type.error?.message }}
    </Message>
    <div class="name">
      <FloatLabel>
        <InputText size="small" v-model="titleGroupForm.name" name="name" class="name-input" />
        <label for="name">{{ t('general.name') }}</label>
      </FloatLabel>
      <Message v-if="$form.name?.invalid" severity="error" size="small" variant="simple">
        {{ $form.name.error?.message }}
      </Message>
    </div>
    <div class="name-aliases input-list" style="margin-top: 5px">
      <label>{{ t('general.alias', 2) }}</label>
      <div v-for="(_alias, index) in titleGroupForm.name_aliases" :key="index">
        <InputText size="small" v-model="titleGroupForm.name_aliases[index]" />
        <Button v-if="index == 0" @click="addNameAlias" icon="pi pi-plus" size="small" />
        <Button v-if="index != 0 || titleGroupForm.name_aliases.length > 1" @click="removeNameAlias(index)" icon="pi pi-minus" size="small" />
      </div>
    </div>
    <div v-if="titleGroupForm.content_type !== null">
      <div class="line" v-if="titleGroupForm.content_type == 'software'">
        <FloatLabel>
          <InputNumber size="small" v-model="titleGroupForm.master_group_id" name="master_group_id" :format="false" />
          <label for="master_group_id">{{ t('master_group.master_group_id') }}</label>
        </FloatLabel>
      </div>
      <div class="line">
        <div v-if="selectableCategories[titleGroupForm.content_type].length !== 0">
          <FloatLabel>
            <Select
              v-model="titleGroupForm.category"
              inputId="category"
              :options="selectableCategories[titleGroupForm.content_type]"
              size="small"
              name="category"
              class="select"
            />
            <label for="category">{{ t('general.category') }}</label>
          </FloatLabel>
          <Message v-if="$form.category?.invalid" severity="error" size="small" variant="simple">
            {{ $form.category.error?.message }}
          </Message>
        </div>
      </div>
      <div class="tags" style="width: 100%" v-if="!editMode">
        <TitleGroupTagsInput v-model="titleGroupForm.tags" @keydown.enter.prevent />
        <Message v-if="customFormErrors['tags']" severity="error" size="small" variant="simple">
          {{ customFormErrors['tags'] }}
        </Message>
      </div>
      <div>
        <FloatLabel>
          <BBCodeEditor
            :initialValue="titleGroupForm.description"
            name="description"
            :label="t('general.description')"
            @valueChange="titleGroupForm.description = $event"
          >
            <template #message>
              <Message v-if="$form.description?.invalid" severity="error" size="small" variant="simple">
                {{ $form.description.error?.message }}
              </Message>
            </template>
          </BBCodeEditor>
          <!-- <label for="description">{{ t('general.description') }}</label> -->
        </FloatLabel>
      </div>
      <div class="line">
        <div v-if="titleGroupForm.content_type == 'software'">
          <FloatLabel>
            <Select
              v-model="titleGroupForm.platform"
              inputId="platform"
              :options="getPlatforms().map((platform) => ({ label: t(`title_group.platforms.${platform}`), value: platform }))"
              optionLabel="label"
              optionValue="value"
              class="select"
              size="small"
              name="platform"
              filter
            />
            <label for="platform">{{ t('title_group.platform') }}</label>
          </FloatLabel>
          <Message v-if="$form.platform?.invalid" severity="error" size="small" variant="simple">
            {{ $form.platform.error?.message }}
          </Message>
        </div>
        <div>
          <FloatLabel>
            <Select
              v-model="titleGroupForm.original_language"
              inputId="original_language"
              :options="getLanguages()"
              class="select"
              size="small"
              name="original_language"
              filter
            />
            <label for="original_language">{{ t('general.original_language') }}</label>
          </FloatLabel>
          <Message v-if="$form.original_language?.invalid" severity="error" size="small" variant="simple">
            {{ $form.original_language.error?.message }}
          </Message>
        </div>
        <div>
          <FloatLabel>
            <Select
              v-model="titleGroupForm.country_from"
              inputId="country_from"
              :options="selectableCountries"
              class="select"
              size="small"
              name="country_from"
              filter
            />
            <label for="country_from">{{ t('general.country') }}</label>
          </FloatLabel>
          <Message v-if="$form.country_from?.invalid" severity="error" size="small" variant="simple">
            {{ $form.country_from.error?.message }}
          </Message>
        </div>
      </div>
      <div class="original-release-date">
        <div class="line" style="margin-bottom: 5px">
          <label for="original_release_date" class="block">{{ t('title_group.original_release_date') }}</label>
          <div class="only-year-known" style="margin-left: 10px">
            <Checkbox
              v-model="titleGroupForm.original_release_date_only_year_known"
              style="margin-right: 3px"
              inputId="only_year_known"
              binary
              @change="onOnlyYearKnownChange"
            />
            <label for="only_year_known">{{ t('title_group.only_year_known') }} </label>
          </div>
        </div>
        <DatePicker
          v-if="!titleGroupForm.original_release_date_only_year_known"
          :manual-input="false"
          v-model="original_release_date"
          showButtonBar
          showIcon
          iconDisplay="input"
          inputId="original_release_date"
          size="small"
          dateFormat="yy-mm-dd"
          name="original_release_date"
        />
        <InputNumber v-else v-model="original_release_year" inputId="original_release_year" size="small" :useGrouping="false" name="original_release_date" />
        <Message v-if="$form.original_release_date?.invalid" severity="error" size="small" variant="simple">
          {{ $form.original_release_date.error?.message }}
        </Message>
      </div>
      <div v-if="!props.editMode">
        <label>{{ t('artist.artist', 2) }}</label>
        <EditAffiliatedArtists
          ref="editAffiliatedArtistsRef"
          :contentType="titleGroupForm.content_type"
          :initial-artists-affiliations="initialArtistsAffiliation"
        />
      </div>
      <div class="covers input-list">
        <label>{{ t('general.cover', 2) }}</label>
        <ImageUploader v-if="publicArcadiaSettings.display_image_host_drag_and_drop" @uploaded="(url) => onImageUploaded(titleGroupForm.covers, url)" />
        <div v-for="(_link, index) in titleGroupForm.covers" :key="index">
          <InputText size="small" v-model="titleGroupForm.covers[index]" />
          <Button v-if="index == 0" @click="addCover" icon="pi pi-plus" size="small" />
          <Button v-if="titleGroupForm.covers.length > 1" @click="removeCover(index)" icon="pi pi-minus" size="small" />
          <Message v-if="customFormErrors[`covers[${index}]`]" severity="error" size="small" variant="simple">
            {{ customFormErrors[`covers[${index}]`] }}
          </Message>
        </div>
      </div>
      <div class="screenshots input-list" v-if="titleGroupForm.content_type == 'software'">
        <label>{{ t('general.screenshots') }}</label>
        <ImageUploader v-if="publicArcadiaSettings.display_image_host_drag_and_drop" @uploaded="(url) => onImageUploaded(titleGroupForm.screenshots, url)" />
        <div v-for="(_link, index) in titleGroupForm.screenshots" :key="index">
          <InputText size="small" v-model="titleGroupForm.screenshots[index]" />
          <Button v-if="index == 0" @click="addScreenshot" icon="pi pi-plus" size="small" />
          <Button v-if="index != 0 || titleGroupForm.screenshots.length > 1" @click="removeScreenshot(index)" icon="pi pi-minus" size="small" />
          <Message v-if="customFormErrors[`screenshots[${index}]`]" severity="error" size="small" variant="simple">
            {{ customFormErrors[`screenshots[${index}]`] }}
          </Message>
        </div>
      </div>
      <div class="external-links input-list">
        <label>{{ t('general.external_link', 2) }}</label>
        <div v-for="(_link, index) in titleGroupForm.external_links" :key="index">
          <InputText size="small" v-model="titleGroupForm.external_links[index]" />
          <Button v-if="index == 0" @click="addLink" icon="pi pi-plus" size="small" />
          <Button v-if="index != 0 || titleGroupForm.external_links.length > 1" @click="removeLink(index)" icon="pi pi-minus" size="small" />
          <Message v-if="customFormErrors[`external_links[${index}]`]" severity="error" size="small" variant="simple">
            {{ customFormErrors[`external_links[${index}]`] }}
          </Message>
        </div>
      </div>
      <div class="embedded-links input-list">
        <label>{{ t('title_group.trailer', 2) }} ({{ t('title_group.youtube_or_video_link', 2) }})</label>
        <div v-for="(_link, index) in titleGroupForm.trailers" :key="index">
          <InputText size="small" v-model="titleGroupForm.trailers[index]" />
          <Button v-if="index == 0" @click="addEmbeddedLink" icon="pi pi-plus" size="small" />
          <Button v-if="index != 0 || titleGroupForm.trailers.length > 1" @click="removeEmbeddedLink(index)" icon="pi pi-minus" size="small" />
          <Message v-if="customFormErrors[`trailers[${index}]`]" severity="error" size="small" variant="simple">
            {{ customFormErrors[`trailers[${index}]`] }}
          </Message>
        </div>
      </div>
    </div>
    <div class="flex justify-content-center">
      <Button :label="t('general.confirm')" icon="pi pi-check" type="submit" size="small" class="validate-button" :loading="sendingTitleGroup" />
    </div>
  </Form>
</template>
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Form, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import FloatLabel from 'primevue/floatlabel'
import InputText from 'primevue/inputtext'
import Select from 'primevue/select'
import Button from 'primevue/button'
import DatePicker from 'primevue/datepicker'
import Message from 'primevue/message'
import { InputNumber, Checkbox } from 'primevue'
import { useI18n } from 'vue-i18n'
import {
  getSelectableContentTypes,
  getLanguages,
  getPlatforms,
  isValidUrl,
  getYoutubeEmbedLink,
  isReleaseDateRequired,
  formatDateToLocalString,
  parseDateStringToLocal,
  onFormKeydown,
} from '@/services/helpers'
import { useTitleGroupStore } from '@/stores/titleGroup'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import type { VNodeRef } from 'vue'
import EditAffiliatedArtists from '../artist/EditAffiliatedArtists.vue'
import { onMounted } from 'vue'
import { nextTick } from 'vue'
import { pick } from 'lodash-es'
import { showToast } from '@/main'
import type { UserCreatedTitleGroupForm } from './CreateOrSelectTitleGroup.vue'
import TitleGroupTagsInput from '../TitleGroupTagsInput.vue'
import {
  createTitleGroup,
  editTitleGroup,
  TitleGroupCategory,
  type AffiliatedArtistHierarchy,
  type ContentType,
  type EditedTitleGroup,
  type TitleGroup,
  type UserCreatedAffiliatedArtist,
} from '@/services/api-schema'
import BBCodeEditor from '../community/BBCodeEditor.vue'
import ImageUploader from '../ImageUploader.vue'

const props = defineProps<{
  initialTitleGroup?: EditedTitleGroup | UserCreatedTitleGroupForm
  initialArtistsAffiliation?: AffiliatedArtistHierarchy[] | UserCreatedAffiliatedArtist[]
  editMode?: boolean
}>()
const titleGroupStore = ref(useTitleGroupStore())
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const sendingTitleGroup = ref(false)

const titleGroupForm = ref({
  id: 0,
  name: '',
  name_aliases: [''],
  tagline: null,
  description: '',
  original_language: null,
  original_release_date: null as string | null,
  original_release_date_only_year_known: true,
  covers: [''],
  screenshots: [''],
  external_links: [''],
  category: null as null | TitleGroupCategory,
  country_from: '',
  affiliated_artists: [],
  tags: [] as string[],
  master_group_id: null,
  platform: null,
  trailers: [''],
  content_type: null as ContentType | null,
})
const customFormErrors = ref<Record<string, string>>({})
const formRef = ref<VNodeRef | null>(null)
const editAffiliatedArtistsRef = ref<VNodeRef | null>(null)

const original_release_date = computed({
  get() {
    const dateStr = titleGroupForm.value.original_release_date
    return dateStr ? parseDateStringToLocal(dateStr) : null
  },
  set(newValue) {
    titleGroupForm.value.original_release_date = newValue ? formatDateToLocalString(newValue) : null
  },
})

const original_release_year = computed({
  get() {
    const dateStr = titleGroupForm.value.original_release_date
    if (!dateStr) return null
    return parseInt(dateStr.split('-')[0], 10)
  },
  set(newValue: number | null) {
    titleGroupForm.value.original_release_date = newValue ? `${newValue}-01-01` : null
  },
})

const onOnlyYearKnownChange = () => {
  titleGroupForm.value.original_release_date = null
}

const selectableCountries = [
  'UK',
  'USA',
  'France',
  'Argentina',
  'Australia',
  'Austria',
  'Belgium',
  'Brazil',
  'Canada',
  'Chile',
  'China',
  'Colombia',
  'Czech Republic',
  'Denmark',
  'Egypt',
  'Finland',
  'Germany',
  'Greece',
  'Hong Kong',
  'Hungary',
  'India',
  'Indonesia',
  'Ireland',
  'Israel',
  'Italy',
  'Japan',
  'Malaysia',
  'Mexico',
  'Netherlands',
  'New Zealand',
  'Nigeria',
  'Norway',
  'Pakistan',
  'Philippines',
  'Poland',
  'Portugal',
  'Romania',
  'Russia',
  'Saudi Arabia',
  'Singapore',
  'South Africa',
  'South Korea',
  'Spain',
  'Sweden',
  'Switzerland',
  'Taiwan',
  'Thailand',
  'Turkey',
  'Ukraine',
  'Vietnam',
]
const selectableCategories: Record<ContentType, TitleGroupCategory[]> = {
  book: ['Illustrated', 'Periodical', 'Book', 'Article', 'Manual'],
  music: ['Single', 'Album', 'Ep'],
  podcast: [],
  movie: ['FeatureFilm', 'ShortFilm'],
  video: [],
  software: ['Program', 'Game'],
  collection: ['Other'],
  tv_show: [],
  live_performance: [],
}

const { t } = useI18n()

const emit = defineEmits<{
  done: [titleGroup: TitleGroup]
}>()

// type FormErrors = {
//   [key in keyof UserCreatedTitleGroup]: UserCreatedTitleGroup[key] extends Array<unknown>
//     ? { message: string }[][]
//     : { message: string }[]
// }
const resolver = ({ values }: FormResolverOptions) => {
  const errors: Record<string, { message: string }[]> = {}

  if (titleGroupForm.value.content_type === null) {
    errors.content_type = [{ message: t('error.select_content_type') }]
    return { errors }
  }
  if (values.name.length < 1) {
    errors.name = [{ message: t('error.write_more_than_x_chars', [0]) }]
  }
  if (!titleGroupForm.value.category && selectableCategories[titleGroupForm.value.content_type]) {
    errors.category = [{ message: t('error.select_category') }]
  }
  if (titleGroupForm.value.description.length < 10) {
    errors.description = [{ message: t('error.write_more_than_x_chars', [10]) }]
  }
  if (values.platform == '') {
    errors.platform = [{ message: t('error.select_platform') }]
  }
  if (titleGroupForm.value.content_type !== 'music' && values.original_language == '') {
    errors.original_language = [{ message: t('error.select_language') }]
  }
  // if (values.country_from == '') {
  //   errors.country_from = [{ message: t('error.select_country') }]
  // }
  if ((values.original_release_date === null || values.original_release_date == '') && isReleaseDateRequired(titleGroupForm.value.content_type)) {
    errors.original_release_date = [{ message: t('error.select_date') }]
  }
  // affiliated_artists_names.value.forEach((artist_name: string, index: number) => {
  //   if (artist_name === '') {
  //     if (!('affiliated_artists' in errors)) {
  //       errors.affiliated_artists = []
  //     }
  //     errors.affiliated_artists![index] = { message: t('error.invalid_name') }
  //   }
  // })
  // titleGroupForm.value.affiliated_artists.forEach((artist: UserCreatedAffiliatedArtist, index: number) => {
  //   if (artist.roles.length === 0) {
  //     if (!('affiliated_artists' in errors)) {
  //       errors.affiliated_artists = []
  //     }
  //     errors.affiliated_artists![index] = {
  //       message: t('error.artist_must_have_at_least_one_role'),
  //     }
  //   }
  // })
  // values.external_links.forEach((link: string, index: number) => {
  //   if (!isValidUrl(link)) {
  //     if (!('external_links' in errors)) {
  //       errors.external_links = []
  //     }
  //     errors.external_links![index] = { message: t('error.invalid_url') }
  //   }
  // })
  return {
    errors,
  }
}

// some fields aren't properly handled by primevue's form validator
// so we use a custom validator
const validateCustomFields = () => {
  customFormErrors.value = {}
  if (titleGroupForm.value.tags.length < publicArcadiaSettings.min_amount_tags_title_group && !props.editMode) {
    customFormErrors.value['tags'] = t('error.enter_at_least_x_tags', [publicArcadiaSettings.min_amount_tags_title_group])
  }
  const nonEmptyCovers = titleGroupForm.value.covers.filter((link) => link.trim() !== '')
  if (nonEmptyCovers.length === 0) {
    customFormErrors.value['covers[0]'] = t('error.enter_at_least_x_covers', [1])
  } else {
    titleGroupForm.value.covers.forEach((link, index) => {
      if (link.trim() !== '' && !isValidUrl(link)) {
        customFormErrors.value[`covers[${index}]`] = t('error.invalid_url')
      }
    })
  }
  if (titleGroupForm.value.content_type === 'software') {
    titleGroupForm.value.screenshots.forEach((link, index) => {
      if (link.trim() !== '' && !isValidUrl(link)) {
        customFormErrors.value[`screenshots[${index}]`] = t('error.invalid_url')
      }
    })
  }
  return Object.keys(customFormErrors.value).length === 0
}

const sendTitleGroup = async ({ valid }: FormSubmitEvent) => {
  const arrayFieldsValid = validateCustomFields()
  if (!valid || !arrayFieldsValid) return
  sendingTitleGroup.value = true
  const savedCovers = [...titleGroupForm.value.covers]
  const savedScreenshots = [...titleGroupForm.value.screenshots]
  const savedExternalLinks = [...titleGroupForm.value.external_links]
  const savedTrailers = [...titleGroupForm.value.trailers]
  const savedNameAliases = [...titleGroupForm.value.name_aliases]
  titleGroupForm.value.covers = titleGroupForm.value.covers.filter((cover) => cover.trim() !== '')
  titleGroupForm.value.screenshots = titleGroupForm.value.screenshots.filter((screenshot) => screenshot.trim() !== '')
  titleGroupForm.value.external_links = titleGroupForm.value.external_links.filter((link) => link.trim() !== '')
  titleGroupForm.value.trailers = titleGroupForm.value.trailers.filter((link) => link.trim() !== '')
  titleGroupForm.value.name_aliases = titleGroupForm.value.name_aliases.filter((alias) => alias.trim() !== '')
  // convert youtube links to embed links and remove tracking query parameters, video links are stored as is
  titleGroupForm.value.trailers = titleGroupForm.value.trailers.map((link) => getYoutubeEmbedLink(link) ?? link)
  const restoreInputLists = () => {
    titleGroupForm.value.covers = savedCovers
    titleGroupForm.value.screenshots = savedScreenshots
    titleGroupForm.value.external_links = savedExternalLinks
    titleGroupForm.value.trailers = savedTrailers
    titleGroupForm.value.name_aliases = savedNameAliases
  }
  if (props.editMode && props.initialTitleGroup) {
    titleGroupForm.value.id = props.initialTitleGroup.id
    editTitleGroup(titleGroupForm.value as EditedTitleGroup)
      .then((data) => {
        showToast('', t('title_group.title_group_edited_success'), 'success', 3000, true, 'tr')
        emit('done', data)
      })
      .catch(() => {
        restoreInputLists()
      })
      .finally(() => {
        sendingTitleGroup.value = false
      })
  } else {
    try {
      // create artists that need to be created
      await editAffiliatedArtistsRef.value.createInexistingArtists()
    } catch {
      restoreInputLists()
      sendingTitleGroup.value = false
      return
    }
    titleGroupForm.value.affiliated_artists = editAffiliatedArtistsRef.value.affiliated_artists
    const formattedTitleGroupForm = JSON.parse(JSON.stringify(titleGroupForm.value))
    createTitleGroup(formattedTitleGroupForm)
      .then((data) => {
        emit('done', data)
      })
      .catch(() => {
        restoreInputLists()
      })
      .finally(() => {
        sendingTitleGroup.value = false
      })
  }
}

const addNameAlias = () => {
  titleGroupForm.value.name_aliases.push('')
}
const removeNameAlias = (index: number) => {
  titleGroupForm.value.name_aliases.splice(index, 1)
}
const addLink = () => {
  titleGroupForm.value.external_links.push('')
}
const removeLink = (index: number) => {
  titleGroupForm.value.external_links.splice(index, 1)
}
const addCover = () => {
  titleGroupForm.value.covers.push('')
}
const removeCover = (index: number) => {
  titleGroupForm.value.covers.splice(index, 1)
}
const addScreenshot = () => {
  titleGroupForm.value.screenshots.push('')
}
const removeScreenshot = (index: number) => {
  titleGroupForm.value.screenshots.splice(index, 1)
}
const addEmbeddedLink = () => {
  titleGroupForm.value.trailers.push('')
}
const removeEmbeddedLink = (index: number) => {
  titleGroupForm.value.trailers.splice(index, 1)
}
const onImageUploaded = (urls: string[], url: string) => {
  if (urls.length === 1 && urls[0] === '') {
    urls[0] = url
  } else {
    urls.push(url)
  }
}

// const updateTitleGroupForm = (form: Partial<UserCreatedTitleGroupForm>) => {
//   if (form.affiliated_artists && form.affiliated_artists.length === 0) {
//     form.affiliated_artists = titleGroupForm.value.affiliated_artists
//   }
//   titleGroupForm.value = {
//     ...titleGroupForm.value,
//     ...form,
//   }
//   if (titleGroupForm.value.tags.length > 0) {
//     tagsString.value = titleGroupForm.value.tags.join(',')
//   }
//   try {
//     // some fields fail because they are not in the primevueform, but they are in titleGroupForm
//     formRef.value?.setValues(titleGroupForm.value)
//   } catch {}
// }

onMounted(async () => {
  if (props.initialTitleGroup) {
    Object.assign(titleGroupForm.value, pick(JSON.parse(JSON.stringify(props.initialTitleGroup)), Object.keys(titleGroupForm.value)))
    if (titleGroupForm.value.external_links.length === 0) {
      titleGroupForm.value.external_links.push('')
    }
    if (titleGroupForm.value.covers.length === 0) {
      titleGroupForm.value.covers.push('')
    }
    if (titleGroupForm.value.trailers.length === 0) {
      titleGroupForm.value.trailers.push('')
    }
    if (titleGroupForm.value.name_aliases.length === 0) {
      titleGroupForm.value.name_aliases.push('')
    }
    Object.keys(titleGroupForm.value).forEach((key) => {
      try {
        formRef.value?.setFieldValue(key, titleGroupForm.value[key as keyof typeof titleGroupForm.value])
      } catch {
        // some fields fail because they are not in the primevueform, but they are in titleGroupForm
      }
    })
  }
})
watch(
  // some fields of the form are only displayed when content_type is selected
  // validating the form on mount will only have effect on the few fields displayed, and not refreshed automatically
  // when the new fields are added, this watch fixes this
  () => titleGroupForm.value.content_type,
  async (newValue) => {
    if (newValue !== null) {
      // Set default category for book content type, since this is the most commonly chosen
      if (newValue === 'book' && !titleGroupForm.value.category) {
        titleGroupForm.value.category = 'Book'
      }
      await nextTick()
      formRef.value?.validate()
    }
  },
  { immediate: true },
)
</script>
<style scoped>
.description {
  width: 100%;
  height: 10em;
}
.name {
  width: 50%;
  .name-input {
    width: 100%;
  }
}
.select {
  width: 200px;
}
.original-release-date {
  margin-top: 30px;
  margin-bottom: 20px;
}
.input-list {
  margin-bottom: 15px;
}
.input-list input {
  &:not(.artist) {
    width: 400px;
  }
}
.validate-button {
  margin-top: 20px;
}
</style>
